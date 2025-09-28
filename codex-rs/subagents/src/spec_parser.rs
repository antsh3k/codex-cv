use crate::AgentSource;
use crate::DiagnosticLevel;
use crate::Subagent;
use crate::SubagentBuilder;
use crate::SubagentSpec;
use crate::TaskContext;
use crate::TaskContextError;
use crate::TypedSubagent;
use crate::pipeline::AcceptanceCriterion;
use crate::pipeline::Requirement;
use crate::pipeline::RequirementsSpec;
use anyhow::Result;
use anyhow::anyhow;
use once_cell::sync::Lazy;
use regex_lite::Regex;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;

static REQUIREMENT_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^-\s*\[(?P<id>[A-Z0-9_-]+)\]\s*(?P<summary>.+)$").expect("valid regex")
});

static ACCEPTANCE_LINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<id>AC-[0-9]{3,})?\s*:?\s*(?P<text>.+)$").expect("valid regex"));

static FILE_HINT_LINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^file[s]?\s*:\s*(?P<files>.+)$").expect("valid regex"));

const SPEC_PARSER_PROMPT: &str = r#"
You are the specification parser subagent. Read the user's request and produce a concise
requirements document. Ensure every requirement has an identifier (format `REQ-###`),
a summary, and at least one acceptance criterion. Capture any referenced files when
explicitly mentioned (e.g. `files: src/lib.rs`).

Return JSON following this schema:
- title: short title for the effort
- overview: one paragraph summary
- requirements: array of requirement objects with `id`, `summary`,
  `acceptance_criteria` (array of strings), and optional `file_hints` (array of strings).
"#;

#[derive(Default)]
pub struct SpecParserSubagent;

#[derive(Debug, Clone)]
pub struct SpecParserSeed {
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecParserOutput {
    pub spec: RequirementsSpec,
}

impl SpecParserSeed {
    pub fn new(markdown: impl Into<String>) -> Self {
        Self {
            markdown: markdown.into(),
        }
    }
}

impl SpecParserSubagent {
    pub fn subagent_spec() -> SubagentSpec {
        SubagentBuilder::new("spec-parser")
            .description(Some(
                "Parses natural language briefs into structured requirements".to_string(),
            ))
            .model(Some("gpt-5-codex".to_string()))
            .keywords(["requirements", "spec", "analysis"])
            .instructions(SPEC_PARSER_PROMPT)
            .source(AgentSource::Builtin)
            .build()
            .expect("valid spec parser definition")
    }

    fn load_seed(ctx: &TaskContext) -> Result<SpecParserSeed, anyhow::Error> {
        ctx.get_typed::<SpecParserSeed>()?
            .ok_or_else(|| anyhow!("SpecParserSeed not present in task context"))
    }
}

impl Subagent for SpecParserSubagent {
    fn spec(&self) -> Cow<'_, SubagentSpec> {
        Cow::Owned(Self::subagent_spec())
    }
}

impl TypedSubagent for SpecParserSubagent {
    type Input = SpecParserSeed;
    type Output = SpecParserOutput;

    fn prepare(&self, ctx: &TaskContext) -> Result<Self::Input> {
        Self::load_seed(ctx)
    }

    fn execute(&self, ctx: &mut TaskContext, input: Self::Input) -> Result<Self::Output> {
        let parsed = parse_spec_markdown(&input.markdown).map_err(|err| {
            let _ = ctx.push_diagnostic(
                DiagnosticLevel::Error,
                format!("Spec parsing failed: {err}"),
            );
            err
        })?;
        ctx.push_diagnostic(
            DiagnosticLevel::Info,
            format!("Parsed {} requirements", parsed.requirements.len()),
        )?;
        Ok(SpecParserOutput { spec: parsed })
    }

    fn finalize(&self, ctx: &mut TaskContext, output: Self::Output) -> Result<()> {
        ctx.insert_typed(output.spec.clone())?;
        ctx.set_scratchpad(
            "subagents.spec_parser.output",
            serde_json::to_value(&output).map_err(TaskContextError::Serialization)?,
        )?;
        Ok(())
    }
}

fn parse_spec_markdown(markdown: &str) -> Result<RequirementsSpec> {
    let mut title = String::new();
    let mut overview_lines = Vec::new();
    let mut requirements = Vec::new();
    let mut in_requirements_section = false;
    let mut current: Option<RequirementBuilder> = None;
    let mut seen_ids = HashSet::new();

    for raw_line in markdown.lines() {
        let line = raw_line.trim_end();
        if line.starts_with('#') {
            if line.starts_with("##") {
                let heading = line.trim_start_matches('#').trim();
                if heading.eq_ignore_ascii_case("requirements") {
                    if let Some(builder) = current.take() {
                        requirements.push(builder.finish()?);
                    }
                    in_requirements_section = true;
                    continue;
                }
            } else if title.is_empty() {
                title = line.trim_start_matches('#').trim().to_string();
                continue;
            }
        }

        if !in_requirements_section {
            if !line.trim().is_empty() {
                overview_lines.push(line.trim().to_string());
            }
            continue;
        }

        let trimmed = line.trim();

        if let Some(captures) = REQUIREMENT_LINE.captures(trimmed) {
            if let Some(builder) = current.take() {
                requirements.push(builder.finish()?);
            }
            let id = captures
                .name("id")
                .expect("id capture")
                .as_str()
                .to_string();
            validate_requirement_id(&id)?;
            if !seen_ids.insert(id.clone()) {
                return Err(anyhow!("Duplicate requirement id {id}"));
            }
            let summary = captures
                .name("summary")
                .expect("summary capture")
                .as_str()
                .trim()
                .to_string();
            current = Some(RequirementBuilder::new(id, summary));
            continue;
        }

        if let Some(builder) = current.as_mut() {
            if !trimmed.starts_with('-') {
                continue;
            }
            let inner = trimmed.trim_start_matches('-').trim();

            if let Some(file_caps) = FILE_HINT_LINE.captures(inner) {
                let files = file_caps.name("files").expect("files").as_str();
                let hints = files
                    .split(',')
                    .map(|f| f.trim().to_string())
                    .filter(|f| !f.is_empty());
                for hint in hints {
                    builder.file_hints.push(hint);
                }
                continue;
            }

            if let Some(ac_caps) = ACCEPTANCE_LINE.captures(inner) {
                let id = ac_caps.name("id").map(|m| m.as_str().to_string());
                let text = ac_caps
                    .name("text")
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();
                if text.is_empty() {
                    return Err(anyhow!("Acceptance criterion missing text"));
                }
                builder
                    .acceptance_criteria
                    .push(AcceptanceCriterion::new(id, text));
                continue;
            }

            if !inner.is_empty() {
                builder
                    .acceptance_criteria
                    .push(AcceptanceCriterion::new(None, inner.to_string()));
            }
        }
    }

    if let Some(builder) = current.take() {
        requirements.push(builder.finish()?);
    }

    if requirements.is_empty() {
        return Err(anyhow!("No requirements found"));
    }

    let overview = if overview_lines.is_empty() {
        "No overview provided".to_string()
    } else {
        overview_lines.join(" ")
    };

    Ok(RequirementsSpec::new(
        if title.is_empty() { "Untitled" } else { &title },
        overview,
        requirements,
    ))
}

fn validate_requirement_id(id: &str) -> Result<()> {
    if !id.starts_with("REQ-") {
        return Err(anyhow!("Requirement id {id} must start with REQ-"));
    }
    let suffix = &id[4..];
    if suffix.is_empty() || suffix.chars().any(|c| !c.is_ascii_digit()) {
        return Err(anyhow!("Requirement id {id} must end in digits"));
    }
    Ok(())
}

struct RequirementBuilder {
    id: String,
    summary: String,
    acceptance_criteria: Vec<AcceptanceCriterion>,
    file_hints: Vec<String>,
}

impl RequirementBuilder {
    fn new(id: String, summary: String) -> Self {
        Self {
            id,
            summary,
            acceptance_criteria: Vec::new(),
            file_hints: Vec::new(),
        }
    }

    fn finish(self) -> Result<Requirement> {
        if self.acceptance_criteria.is_empty() {
            return Err(anyhow!(
                "Requirement {} missing acceptance criteria",
                self.id
            ));
        }
        Ok(Requirement::new(
            self.id,
            self.summary,
            self.acceptance_criteria,
            self.file_hints,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskContext;

    const SAMPLE: &str = r#"
# Feature Rollout

The product team needs an authenticated export feature.

## Requirements
- [REQ-001] Add export command
  - AC-001: CLI exposes `codex export`
  - AC-002: command requires auth token
  - files: cli/src/commands/export.rs, core/src/export.rs
- [REQ-002] Log export telemetry
  - AC-003: record success and failure events
  - file: core/src/telemetry.rs
"#;

    #[test]
    fn parses_markdown_into_structured_spec() {
        let mut ctx = TaskContext::new();
        ctx.insert_typed(SpecParserSeed::new(SAMPLE.to_string()))
            .unwrap();
        let agent = SpecParserSubagent;
        let input = agent.prepare(&ctx).unwrap();
        let output = agent.execute(&mut ctx, input).unwrap();
        agent.finalize(&mut ctx, output.clone()).unwrap();

        let snapshot = serde_json::to_string_pretty(&output).unwrap();
        insta::assert_snapshot!("spec_parser_basic", snapshot);
    }

    #[test]
    fn rejects_missing_acceptance_criteria() {
        let markdown = "## Requirements\n- [REQ-123] Missing criteria";
        let err = parse_spec_markdown(markdown).unwrap_err();
        assert!(err.to_string().contains("missing acceptance"));
    }
}
