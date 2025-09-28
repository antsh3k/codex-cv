use std::path::PathBuf;

use crate::error::SubagentError;
use crate::error::SubagentResult;
use crate::spec::SubagentSpec;

/// Builder for constructing [`SubagentSpec`] values programmatically.
#[derive(Debug, Clone)]
pub struct SubagentBuilder {
    name: String,
    description: Option<String>,
    model: Option<String>,
    tools: Vec<String>,
    keywords: Vec<String>,
    instructions: Option<String>,
    source_path: Option<PathBuf>,
}

impl SubagentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            model: None,
            tools: Vec::new(),
            keywords: Vec::new(),
            instructions: None,
            source_path: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn tools<I, S>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    pub fn source_path(mut self, source_path: impl Into<PathBuf>) -> Self {
        self.source_path = Some(source_path.into());
        self
    }

    pub fn build(self) -> SubagentResult<SubagentSpec> {
        let instructions = self
            .instructions
            .ok_or_else(|| SubagentError::InvalidSpec("instructions body missing".to_string()))?;

        if self.name.trim().is_empty() {
            return Err(SubagentError::InvalidSpec(
                "name cannot be empty".to_string(),
            ));
        }

        let tools = dedup(self.tools);
        let keywords = dedup(self.keywords);

        Ok(SubagentSpec::new(self.name, instructions)
            .with_description(self.description)
            .with_model(self.model)
            .with_tools(tools)
            .with_keywords(keywords)
            .with_source_path(self.source_path))
    }
}

fn dedup(values: Vec<String>) -> Vec<String> {
    let mut unique = indexmap::IndexSet::with_capacity(values.len());
    for value in values {
        if !value.trim().is_empty() {
            unique.insert(value);
        }
    }
    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn builder_requires_instructions() {
        let err = SubagentBuilder::new("demo").build().unwrap_err();
        assert!(
            matches!(err, SubagentError::InvalidSpec(message) if message.contains("instructions"))
        );
    }

    #[test]
    fn builder_deduplicates_lists() {
        let spec = SubagentBuilder::new("demo")
            .instructions("body")
            .tools(["git", "git", ""]) // empties dropped
            .keywords(["lint", "lint", "docs"])
            .build()
            .unwrap();

        assert_eq!(spec.tools(), &["git".to_string()]);
        assert_eq!(spec.keywords(), &["lint".to_string(), "docs".to_string()]);
    }
}
