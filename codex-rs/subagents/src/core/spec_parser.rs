//! Spec Parser subagent for converting natural language requirements into structured specifications.

use crate::error::{SubagentError, SubagentResult};
use crate::pipeline::{RequirementsSpec, AcceptanceCriterion, Priority};
use crate::spec::SubagentSpec;
use crate::task_context::TaskContext;
use crate::traits::{Subagent, TypedSubagent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Input for the Spec Parser subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecParserRequest {
    /// Raw requirements text or user story
    pub requirements_text: String,
    /// Optional context about the existing codebase
    pub codebase_context: Option<String>,
    /// Relevant file paths for context
    pub related_files: Vec<PathBuf>,
    /// Additional metadata to include
    pub metadata: HashMap<String, String>,
}

/// Output from the Spec Parser subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecParserResponse {
    /// Structured requirements specification
    pub requirements: RequirementsSpec,
    /// Confidence level in the parsing results
    pub confidence: f32,
    /// Any warnings or notes from the parsing process
    pub parsing_notes: Vec<String>,
}

/// Spec Parser subagent implementation
pub struct SpecParserSubagent {
    spec: SubagentSpec,
    /// Template for structured prompt generation
    prompt_template: SpecParserPromptTemplate,
}

impl SpecParserSubagent {
    /// Create a new Spec Parser subagent
    pub fn new(spec: SubagentSpec) -> Self {
        Self {
            spec,
            prompt_template: SpecParserPromptTemplate::default(),
        }
    }

    /// Create with custom prompt template
    pub fn with_template(spec: SubagentSpec, template: SpecParserPromptTemplate) -> Self {
        Self {
            spec,
            prompt_template: template,
        }
    }

    /// Generate structured prompt for requirements parsing
    fn generate_prompt(&self, request: &SpecParserRequest) -> String {
        self.prompt_template.render(request)
    }

    /// Parse and validate the LLM response into a structured specification
    fn parse_response(&self, response_text: &str, request: &SpecParserRequest) -> SubagentResult<SpecParserResponse> {
        // This would typically involve:
        // 1. JSON/YAML parsing of structured LLM output
        // 2. Schema validation against RequirementsSpec
        // 3. Confidence scoring based on completeness
        // 4. Generation of parsing notes and warnings

        // For now, implement a basic parser that creates a specification
        let requirements_id = format!("req-{}", uuid::Uuid::new_v4());

        // Extract title from first line or generate one
        let title = self.extract_title(&request.requirements_text);

        // Parse acceptance criteria from requirements text
        let acceptance_criteria = self.parse_acceptance_criteria(&request.requirements_text);

        // Create structured requirements
        let mut requirements = RequirementsSpec::new(
            requirements_id,
            title,
            request.requirements_text.clone(),
        );

        for criterion in acceptance_criteria {
            requirements.add_criterion(criterion);
        }

        requirements.related_files = request.related_files.clone();
        requirements.metadata = request.metadata.clone();

        // Calculate confidence based on completeness
        let confidence = self.calculate_confidence(&requirements, &request.requirements_text);

        // Generate parsing notes
        let parsing_notes = self.generate_parsing_notes(&requirements);

        Ok(SpecParserResponse {
            requirements,
            confidence,
            parsing_notes,
        })
    }

    /// Extract title from requirements text
    fn extract_title(&self, text: &str) -> String {
        // Look for common title patterns
        let lines: Vec<&str> = text.lines().collect();

        if let Some(first_line) = lines.first() {
            let cleaned = first_line.trim();

            // Check for markdown headers
            if cleaned.starts_with('#') {
                return cleaned.trim_start_matches('#').trim().to_string();
            }

            // Check for "Title:" pattern
            if cleaned.to_lowercase().starts_with("title:") {
                return cleaned[6..].trim().to_string();
            }

            // Use first sentence if it's reasonable length
            if cleaned.len() <= 100 && !cleaned.is_empty() {
                return cleaned.to_string();
            }
        }

        // Fallback: generate title from content
        "Parsed Requirements".to_string()
    }

    /// Parse acceptance criteria from requirements text
    fn parse_acceptance_criteria(&self, text: &str) -> Vec<AcceptanceCriterion> {
        let mut criteria = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Look for bullet points, numbered lists, or "Given/When/Then" patterns
            if self.is_acceptance_criterion_line(trimmed) {
                let criterion_text = self.clean_criterion_text(trimmed);
                if !criterion_text.is_empty() {
                    let criterion = AcceptanceCriterion {
                        id: format!("ac-{}", index + 1),
                        description: criterion_text,
                        priority: self.infer_priority(trimmed),
                        testable: self.is_testable(trimmed),
                        test_scenario: self.extract_test_scenario(trimmed),
                    };
                    criteria.push(criterion);
                }
            }
        }

        // If no criteria found, create one from the overall requirement
        if criteria.is_empty() {
            criteria.push(AcceptanceCriterion {
                id: "ac-1".to_string(),
                description: "Requirements must be implemented as specified".to_string(),
                priority: Priority::High,
                testable: false,
                test_scenario: None,
            });
        }

        criteria
    }

    /// Check if a line represents an acceptance criterion
    fn is_acceptance_criterion_line(&self, line: &str) -> bool {
        let lower = line.to_lowercase();

        // Bullet points
        if line.starts_with('-') || line.starts_with('*') || line.starts_with('+') {
            return true;
        }

        // Numbered lists
        if line.chars().next().map_or(false, |c| c.is_ascii_digit()) && line.contains('.') {
            return true;
        }

        // BDD patterns
        if lower.starts_with("given ") || lower.starts_with("when ") || lower.starts_with("then ") {
            return true;
        }

        // Acceptance criteria markers
        if lower.contains("must ") || lower.contains("should ") || lower.contains("shall ") {
            return true;
        }

        false
    }

    /// Clean acceptance criterion text
    fn clean_criterion_text(&self, line: &str) -> String {
        let mut cleaned = line.trim();

        // Remove bullet points and numbers
        if cleaned.starts_with('-') || cleaned.starts_with('*') || cleaned.starts_with('+') {
            cleaned = &cleaned[1..].trim();
        }

        // Remove numbered list markers
        if let Some(dot_pos) = cleaned.find('.') {
            if cleaned[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
                cleaned = &cleaned[dot_pos + 1..].trim();
            }
        }

        cleaned.to_string()
    }

    /// Infer priority from criterion text
    fn infer_priority(&self, text: &str) -> Priority {
        let lower = text.to_lowercase();

        if lower.contains("must") || lower.contains("critical") || lower.contains("required") {
            Priority::High
        } else if lower.contains("should") || lower.contains("important") {
            Priority::Medium
        } else {
            Priority::Low
        }
    }

    /// Check if criterion is testable
    fn is_testable(&self, text: &str) -> bool {
        let lower = text.to_lowercase();

        // Look for testable keywords
        lower.contains("test") || lower.contains("verify") || lower.contains("validate") ||
        lower.contains("given") || lower.contains("when") || lower.contains("then") ||
        lower.contains("should") || lower.contains("function") || lower.contains("feature")
    }

    /// Extract test scenario if present
    fn extract_test_scenario(&self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();

        if lower.contains("given") && (lower.contains("when") || lower.contains("then")) {
            Some(text.to_string())
        } else {
            None
        }
    }

    /// Calculate confidence score based on specification completeness
    fn calculate_confidence(&self, requirements: &RequirementsSpec, original_text: &str) -> f32 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Title completeness (10%)
        max_score += 0.1;
        if !requirements.title.is_empty() && requirements.title != "Parsed Requirements" {
            score += 0.1;
        }

        // Description completeness (20%)
        max_score += 0.2;
        if requirements.description.len() > 20 {
            score += 0.2;
        }

        // Acceptance criteria presence (40%)
        max_score += 0.4;
        if !requirements.acceptance_criteria.is_empty() {
            score += 0.2;

            // Bonus for multiple criteria
            if requirements.acceptance_criteria.len() > 1 {
                score += 0.1;
            }

            // Bonus for testable criteria
            let testable_count = requirements.acceptance_criteria.iter()
                .filter(|c| c.testable)
                .count();
            if testable_count > 0 {
                score += 0.1;
            }
        }

        // Content analysis (30%)
        max_score += 0.3;
        let word_count = original_text.split_whitespace().count();
        if word_count > 10 {
            score += 0.1;
        }
        if word_count > 50 {
            score += 0.1;
        }
        if original_text.lines().count() > 3 {
            score += 0.1;
        }

        (score / max_score).min(1.0)
    }

    /// Generate parsing notes and warnings
    fn generate_parsing_notes(&self, requirements: &RequirementsSpec) -> Vec<String> {
        let mut notes = Vec::new();

        if requirements.acceptance_criteria.is_empty() {
            notes.push("No explicit acceptance criteria found in requirements text".to_string());
        }

        if requirements.acceptance_criteria.len() == 1 {
            notes.push("Only one acceptance criterion identified; consider adding more specific criteria".to_string());
        }

        let testable_count = requirements.acceptance_criteria.iter()
            .filter(|c| c.testable)
            .count();

        if testable_count == 0 {
            notes.push("No testable acceptance criteria identified".to_string());
        }

        let high_priority_count = requirements.acceptance_criteria.iter()
            .filter(|c| c.priority == Priority::High)
            .count();

        if high_priority_count == 0 {
            notes.push("No high-priority acceptance criteria identified".to_string());
        }

        if requirements.related_files.is_empty() {
            notes.push("No related files specified; consider adding relevant file contexts".to_string());
        }

        notes
    }
}

impl Subagent for SpecParserSubagent {
    fn spec(&self) -> &SubagentSpec {
        &self.spec
    }

    fn spec_mut(&mut self) -> &mut SubagentSpec {
        &mut self.spec
    }
}

impl TypedSubagent for SpecParserSubagent {
    type Request = SpecParserRequest;
    type Response = SpecParserResponse;

    fn run(
        &mut self,
        ctx: &mut TaskContext,
        request: Self::Request,
    ) -> SubagentResult<Self::Response> {
        ctx.info("Starting requirements specification parsing");

        // Generate structured prompt
        let prompt = self.generate_prompt(&request);
        ctx.debug(&format!("Generated prompt ({}chars)", prompt.len()));

        // TODO: In a full implementation, this would:
        // 1. Send the prompt to the configured LLM model
        // 2. Parse the structured response (JSON/YAML)
        // 3. Validate against schema
        // 4. Return parsed results

        // For now, simulate LLM response parsing
        let simulated_response = format!(
            "Parsed requirements:\nTitle: {}\nCriteria: {}",
            self.extract_title(&request.requirements_text),
            request.requirements_text.lines().count()
        );

        // Parse the simulated response
        let response = self.parse_response(&simulated_response, &request)?;

        ctx.info(&format!(
            "Parsed {} acceptance criteria with {:.1}% confidence",
            response.requirements.acceptance_criteria.len(),
            response.confidence * 100.0
        ));

        for note in &response.parsing_notes {
            ctx.warning(note);
        }

        Ok(response)
    }
}

/// Template system for generating structured prompts
#[derive(Debug, Clone)]
pub struct SpecParserPromptTemplate {
    /// Base system prompt for requirements parsing
    pub system_prompt: String,
    /// Template for the user prompt
    pub user_prompt_template: String,
    /// Expected output schema description
    pub output_schema: String,
}

impl SpecParserPromptTemplate {
    /// Render the complete prompt for a request
    pub fn render(&self, request: &SpecParserRequest) -> String {
        let user_prompt = self.user_prompt_template
            .replace("{requirements_text}", &request.requirements_text)
            .replace("{codebase_context}",
                request.codebase_context.as_deref().unwrap_or("No additional context provided"))
            .replace("{related_files}",
                &request.related_files.iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "));

        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            self.system_prompt,
            self.output_schema,
            user_prompt,
            "Please provide your response in the specified JSON format."
        )
    }
}

impl Default for SpecParserPromptTemplate {
    fn default() -> Self {
        Self {
            system_prompt: r#"You are a requirements analysis expert specializing in converting natural language requirements into structured, testable specifications. Your role is to parse user requirements and extract:

1. Clear, actionable acceptance criteria
2. Priority levels for each criterion
3. Testability indicators
4. Structured metadata

Focus on precision, completeness, and testability in your analysis."#.to_string(),

            user_prompt_template: r#"Please analyze the following requirements and convert them into a structured specification:

**Requirements Text:**
{requirements_text}

**Codebase Context:**
{codebase_context}

**Related Files:**
{related_files}

Extract clear acceptance criteria, assign appropriate priorities (High/Medium/Low), and identify which criteria are testable. Provide a comprehensive analysis that maintains the original intent while adding structure."#.to_string(),

            output_schema: r#"**Expected Output Format (JSON):**
```json
{
  "title": "Brief, descriptive title for the requirements",
  "description": "Detailed description of the requirements",
  "acceptance_criteria": [
    {
      "id": "ac-1",
      "description": "Clear, actionable criterion description",
      "priority": "High|Medium|Low",
      "testable": true|false,
      "test_scenario": "Optional BDD-style test scenario"
    }
  ],
  "constraints": ["Any technical or business constraints"],
  "metadata": {
    "complexity": "Low|Medium|High",
    "estimated_effort": "description"
  }
}
```"#.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::SubagentSpec;

    fn create_test_spec() -> SubagentSpec {
        SubagentSpec::builder()
            .name("spec-parser".to_string())
            .description("Test spec parser".to_string())
            .instructions("Parse requirements".to_string())
            .build()
            .unwrap()
    }

    #[test]
    fn test_spec_parser_creation() {
        let spec = create_test_spec();
        let parser = SpecParserSubagent::new(spec);
        assert_eq!(parser.name(), "spec-parser");
    }

    #[test]
    fn test_title_extraction() {
        let spec = create_test_spec();
        let parser = SpecParserSubagent::new(spec);

        // Test markdown header
        let title = parser.extract_title("# User Authentication Feature\nDetailed description...");
        assert_eq!(title, "User Authentication Feature");

        // Test title: pattern
        let title = parser.extract_title("Title: Login System\nMore details...");
        assert_eq!(title, "Login System");

        // Test first line
        let title = parser.extract_title("Build a search feature\nWith filters and sorting");
        assert_eq!(title, "Build a search feature");
    }

    #[test]
    fn test_acceptance_criteria_parsing() {
        let spec = create_test_spec();
        let parser = SpecParserSubagent::new(spec);

        let text = r#"
User Login Feature

The system must support user authentication.

- Users must be able to log in with email and password
- Users should receive error messages for invalid credentials
- Given a valid user, when they log in, then they should be redirected to dashboard
"#;

        let criteria = parser.parse_acceptance_criteria(text);
        assert_eq!(criteria.len(), 3);

        assert_eq!(criteria[0].description, "Users must be able to log in with email and password");
        assert_eq!(criteria[0].priority, Priority::High);

        assert_eq!(criteria[1].description, "Users should receive error messages for invalid credentials");
        assert_eq!(criteria[1].priority, Priority::Medium);

        assert!(criteria[2].testable);
        assert!(criteria[2].test_scenario.is_some());
    }

    #[test]
    fn test_confidence_calculation() {
        let spec = create_test_spec();
        let parser = SpecParserSubagent::new(spec);

        let mut requirements = RequirementsSpec::new(
            "req-1".to_string(),
            "Good Title".to_string(),
            "Detailed description with sufficient content to demonstrate completeness".to_string(),
        );

        let criterion = AcceptanceCriterion {
            id: "ac-1".to_string(),
            description: "System must work correctly".to_string(),
            priority: Priority::High,
            testable: true,
            test_scenario: Some("Given input when action then result".to_string()),
        };
        requirements.add_criterion(criterion);

        let confidence = parser.calculate_confidence(&requirements, "This is a long enough requirements text with multiple words and sufficient detail to test confidence calculation");
        assert!(confidence > 0.8, "Confidence should be high for complete specification");
    }

    #[test]
    fn test_prompt_template_rendering() {
        let template = SpecParserPromptTemplate::default();
        let request = SpecParserRequest {
            requirements_text: "Build a login system".to_string(),
            codebase_context: Some("Existing auth module".to_string()),
            related_files: vec![PathBuf::from("src/auth.rs")],
            metadata: HashMap::new(),
        };

        let prompt = template.render(&request);
        assert!(prompt.contains("Build a login system"));
        assert!(prompt.contains("Existing auth module"));
        assert!(prompt.contains("src/auth.rs"));
    }
}