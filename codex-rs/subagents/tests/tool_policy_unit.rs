//! Comprehensive unit tests for tool policy enforcement and allowlist validation.

use codex_subagents::spec::SubagentSpec;
use codex_subagents::builder::SubagentBuilder;
use codex_subagents::error::{SubagentError, SubagentResult};
use std::collections::HashSet;
use std::path::PathBuf;

/// Mock tool policy validator to test enforcement logic
#[derive(Debug, Clone)]
pub struct ToolPolicyValidator {
    global_allowlist: HashSet<String>,
    denied_tools: HashSet<String>,
    require_explicit_allowlist: bool,
}

impl ToolPolicyValidator {
    pub fn new() -> Self {
        Self {
            global_allowlist: [
                "read", "write", "edit", "bash", "grep", "analysis",
                "git", "cargo", "npm", "python", "node"
            ].iter().map(|s| s.to_string()).collect(),
            denied_tools: ["rm", "sudo", "dd", "format", "fdisk"].iter().map(|s| s.to_string()).collect(),
            require_explicit_allowlist: true,
        }
    }

    pub fn with_global_allowlist(mut self, tools: Vec<String>) -> Self {
        self.global_allowlist = tools.into_iter().collect();
        self
    }

    pub fn with_denied_tools(mut self, tools: Vec<String>) -> Self {
        self.denied_tools = tools.into_iter().collect();
        self
    }

    pub fn require_explicit_allowlist(mut self, require: bool) -> Self {
        self.require_explicit_allowlist = require;
        self
    }

    /// Validate a subagent's tool policy according to security requirements
    pub fn validate_subagent_policy(&self, spec: &SubagentSpec) -> SubagentResult<ValidationReport> {
        let mut report = ValidationReport::new();
        let allowed_tools = spec.tools();

        // Check if agent has explicit allowlist when required
        if self.require_explicit_allowlist && allowed_tools.is_empty() {
            report.add_warning(format!(
                "Subagent '{}' has no tools specified in allowlist. This may limit functionality.",
                spec.name()
            ));
        }

        // Validate each tool in the allowlist
        for tool in allowed_tools {
            self.validate_tool(&mut report, tool, spec.name())?;
        }

        // Check for security violations
        if report.has_security_violations() {
            return Err(SubagentError::InvalidSpec(format!(
                "Subagent '{}' has security violations in tool allowlist",
                spec.name()
            )));
        }

        Ok(report)
    }

    /// Validate an individual tool call against policy
    pub fn validate_tool_call(&self, tool_name: &str, subagent_name: &str, context: &ToolCallContext) -> SubagentResult<()> {
        // Check if tool is explicitly denied
        if self.denied_tools.contains(tool_name) {
            return Err(SubagentError::InvalidSpec(format!(
                "Tool '{}' is globally denied and cannot be used by subagent '{}'",
                tool_name, subagent_name
            )));
        }

        // Check if tool is in global allowlist
        if !self.global_allowlist.contains(tool_name) {
            return Err(SubagentError::InvalidSpec(format!(
                "Tool '{}' is not in global allowlist for subagent '{}'",
                tool_name, subagent_name
            )));
        }

        // Validate tool call parameters if provided
        if let Some(params) = &context.parameters {
            self.validate_tool_parameters(tool_name, params, subagent_name)?;
        }

        Ok(())
    }

    fn validate_tool(&self, report: &mut ValidationReport, tool: &str, agent_name: &str) -> SubagentResult<()> {
        // Check for explicitly denied tools
        if self.denied_tools.contains(tool) {
            report.add_security_violation(format!(
                "Tool '{}' is explicitly denied for security reasons",
                tool
            ));
            return Ok(()); // Continue validation to report all issues
        }

        // Check against global allowlist
        if !self.global_allowlist.contains(tool) {
            report.add_warning(format!(
                "Tool '{}' is not in global allowlist - may be rejected at runtime",
                tool
            ));
        } else {
            report.add_approved_tool(tool.to_string());
        }

        Ok(())
    }

    fn validate_tool_parameters(&self, tool_name: &str, parameters: &ToolParameters, subagent_name: &str) -> SubagentResult<()> {
        match tool_name {
            "bash" => self.validate_bash_parameters(parameters, subagent_name),
            "write" | "edit" => self.validate_file_parameters(parameters, subagent_name),
            "git" => self.validate_git_parameters(parameters, subagent_name),
            _ => Ok(()), // No specific validation for other tools
        }
    }

    fn validate_bash_parameters(&self, params: &ToolParameters, subagent_name: &str) -> SubagentResult<()> {
        if let Some(command) = params.get_string("command") {
            // Check for dangerous commands
            let dangerous_patterns = ["rm -rf", "sudo", "dd if=", "mkfs", "> /dev/"];
            for pattern in &dangerous_patterns {
                if command.contains(pattern) {
                    return Err(SubagentError::InvalidSpec(format!(
                        "Bash command contains dangerous pattern '{}' for subagent '{}'",
                        pattern, subagent_name
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_file_parameters(&self, params: &ToolParameters, subagent_name: &str) -> SubagentResult<()> {
        if let Some(file_path) = params.get_string("file_path") {
            let path = PathBuf::from(file_path);

            // Check for attempts to write to sensitive system files
            let sensitive_paths = ["/etc/", "/bin/", "/usr/bin/", "/sbin/", "/usr/sbin/"];
            for sensitive in &sensitive_paths {
                if file_path.starts_with(sensitive) {
                    return Err(SubagentError::InvalidSpec(format!(
                        "Attempt to write to sensitive system path '{}' by subagent '{}'",
                        file_path, subagent_name
                    )));
                }
            }

            // Check for attempts to escape working directory
            if file_path.contains("../") || file_path.starts_with("/") {
                return Err(SubagentError::InvalidSpec(format!(
                    "File path '{}' attempts to escape working directory for subagent '{}'",
                    file_path, subagent_name
                )));
            }
        }
        Ok(())
    }

    fn validate_git_parameters(&self, params: &ToolParameters, subagent_name: &str) -> SubagentResult<()> {
        if let Some(command) = params.get_string("command") {
            // Check for dangerous git operations
            let dangerous_git_ops = ["push --force", "reset --hard", "clean -fd", "gc --prune=now"];
            for op in &dangerous_git_ops {
                if command.contains(op) {
                    return Err(SubagentError::InvalidSpec(format!(
                        "Git command contains dangerous operation '{}' for subagent '{}'",
                        op, subagent_name
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Tool call context for validation
#[derive(Debug, Clone)]
pub struct ToolCallContext {
    pub parameters: Option<ToolParameters>,
    pub execution_context: ExecutionContext,
}

#[derive(Debug, Clone)]
pub struct ToolParameters {
    params: std::collections::HashMap<String, serde_json::Value>,
}

impl ToolParameters {
    pub fn new() -> Self {
        Self {
            params: std::collections::HashMap::new(),
        }
    }

    pub fn with_string(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        self
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.params.get(key)?.as_str().map(|s| s.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub sandbox_enabled: bool,
    pub working_directory: PathBuf,
    pub environment_vars: std::collections::HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            sandbox_enabled: true,
            working_directory: PathBuf::from("."),
            environment_vars: std::collections::HashMap::new(),
        }
    }

    pub fn with_sandbox(mut self, enabled: bool) -> Self {
        self.sandbox_enabled = enabled;
        self
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_directory = dir;
        self
    }
}

/// Validation report for tool policy checks
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub warnings: Vec<String>,
    pub security_violations: Vec<String>,
    pub approved_tools: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            security_violations: Vec::new(),
            approved_tools: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_security_violation(&mut self, violation: String) {
        self.security_violations.push(violation);
    }

    pub fn add_approved_tool(&mut self, tool: String) {
        self.approved_tools.push(tool);
    }

    pub fn has_security_violations(&self) -> bool {
        !self.security_violations.is_empty()
    }

    pub fn is_safe(&self) -> bool {
        self.security_violations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_test_spec(name: &str, tools: Vec<&str>) -> SubagentSpec {
        SubagentBuilder::new(name)
            .instructions("Test instructions")
            .tools(tools.into_iter().map(|s| s.to_string()).collect())
            .build()
            .unwrap()
    }

    #[test]
    fn test_validator_allows_safe_tools() {
        let validator = ToolPolicyValidator::new();
        let spec = create_test_spec("test-agent", vec!["read", "write", "analysis"]);

        let report = validator.validate_subagent_policy(&spec).unwrap();
        assert!(report.is_safe());
        assert_eq!(report.approved_tools.len(), 3);
        assert!(report.approved_tools.contains(&"read".to_string()));
        assert!(report.approved_tools.contains(&"write".to_string()));
        assert!(report.approved_tools.contains(&"analysis".to_string()));
    }

    #[test]
    fn test_validator_rejects_denied_tools() {
        let validator = ToolPolicyValidator::new();
        let spec = create_test_spec("dangerous-agent", vec!["read", "rm", "write"]);

        let result = validator.validate_subagent_policy(&spec);
        assert!(result.is_err());

        // Test individual tool validation
        let context = ToolCallContext {
            parameters: None,
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("rm", "dangerous-agent", &context);
        assert!(matches!(result, Err(SubagentError::InvalidSpec(msg)) if msg.contains("globally denied")));
    }

    #[test]
    fn test_validator_warns_about_unknown_tools() {
        let validator = ToolPolicyValidator::new();
        let spec = create_test_spec("experimental-agent", vec!["read", "unknown-tool", "write"]);

        let report = validator.validate_subagent_policy(&spec).unwrap();
        assert!(report.is_safe()); // Unknown tools are warnings, not security violations
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("unknown-tool"));
        assert!(report.warnings[0].contains("not in global allowlist"));
    }

    #[test]
    fn test_validator_warns_about_empty_allowlist() {
        let validator = ToolPolicyValidator::new().require_explicit_allowlist(true);
        let spec = create_test_spec("no-tools-agent", vec![]);

        let report = validator.validate_subagent_policy(&spec).unwrap();
        assert!(report.is_safe());
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("no tools specified"));
        assert!(report.warnings[0].contains("may limit functionality"));
    }

    #[test]
    fn test_validator_allows_empty_allowlist_when_not_required() {
        let validator = ToolPolicyValidator::new().require_explicit_allowlist(false);
        let spec = create_test_spec("flexible-agent", vec![]);

        let report = validator.validate_subagent_policy(&spec).unwrap();
        assert!(report.is_safe());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_tool_call_validation_with_parameters() {
        let validator = ToolPolicyValidator::new();
        let context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("command", "ls -la")),
            execution_context: ExecutionContext::new(),
        };

        // Safe bash command should pass
        let result = validator.validate_tool_call("bash", "test-agent", &context);
        assert!(result.is_ok());

        // Dangerous bash command should fail
        let dangerous_context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("command", "rm -rf /")),
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("bash", "test-agent", &dangerous_context);
        assert!(matches!(result, Err(SubagentError::InvalidSpec(msg)) if msg.contains("dangerous pattern")));
    }

    #[test]
    fn test_file_operation_validation() {
        let validator = ToolPolicyValidator::new();

        // Safe file path should pass
        let safe_context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("file_path", "src/main.rs")),
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("write", "test-agent", &safe_context);
        assert!(result.is_ok());

        // System file path should fail
        let system_context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("file_path", "/etc/passwd")),
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("write", "test-agent", &system_context);
        assert!(matches!(result, Err(SubagentError::InvalidSpec(msg)) if msg.contains("sensitive system path")));

        // Directory escape should fail
        let escape_context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("file_path", "../../../etc/passwd")),
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("write", "test-agent", &escape_context);
        assert!(matches!(result, Err(SubagentError::InvalidSpec(msg)) if msg.contains("escape working directory")));
    }

    #[test]
    fn test_git_operation_validation() {
        let validator = ToolPolicyValidator::new();

        // Safe git command should pass
        let safe_context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("command", "git status")),
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("git", "test-agent", &safe_context);
        assert!(result.is_ok());

        // Dangerous git command should fail
        let dangerous_context = ToolCallContext {
            parameters: Some(ToolParameters::new().with_string("command", "git push --force origin main")),
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("git", "test-agent", &dangerous_context);
        assert!(matches!(result, Err(SubagentError::InvalidSpec(msg)) if msg.contains("dangerous operation")));
    }

    #[test]
    fn test_custom_allowlist_configuration() {
        let validator = ToolPolicyValidator::new()
            .with_global_allowlist(vec!["read".to_string(), "custom-tool".to_string()])
            .with_denied_tools(vec!["write".to_string()]);

        // Custom allowed tool should pass
        let spec = create_test_spec("custom-agent", vec!["read", "custom-tool"]);
        let report = validator.validate_subagent_policy(&spec).unwrap();
        assert!(report.is_safe());
        assert_eq!(report.approved_tools.len(), 2);

        // Custom denied tool should fail
        let spec = create_test_spec("denied-agent", vec!["write"]);
        let result = validator.validate_subagent_policy(&spec);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_report_functionality() {
        let mut report = ValidationReport::new();

        assert!(report.is_safe());
        assert!(!report.has_security_violations());

        report.add_warning("Test warning".to_string());
        assert!(report.is_safe());
        assert_eq!(report.warnings.len(), 1);

        report.add_approved_tool("read".to_string());
        assert_eq!(report.approved_tools.len(), 1);

        report.add_security_violation("Test violation".to_string());
        assert!(!report.is_safe());
        assert!(report.has_security_violations());
        assert_eq!(report.security_violations.len(), 1);
    }

    #[test]
    fn test_execution_context_configuration() {
        let context = ExecutionContext::new()
            .with_sandbox(false)
            .with_working_dir(PathBuf::from("/custom/dir"));

        assert!(!context.sandbox_enabled);
        assert_eq!(context.working_directory, PathBuf::from("/custom/dir"));
    }

    #[test]
    fn test_tool_parameters_manipulation() {
        let params = ToolParameters::new()
            .with_string("command", "test command")
            .with_string("file_path", "test.txt");

        assert_eq!(params.get_string("command"), Some("test command".to_string()));
        assert_eq!(params.get_string("file_path"), Some("test.txt".to_string()));
        assert_eq!(params.get_string("nonexistent"), None);
    }

    #[test]
    fn test_comprehensive_security_scenarios() {
        let validator = ToolPolicyValidator::new();

        // Test multiple security violations in one spec
        let spec = create_test_spec("multi-violation-agent", vec!["read", "rm", "sudo", "write"]);
        let result = validator.validate_subagent_policy(&spec);
        assert!(result.is_err());

        // Test edge case: empty tool name
        let context = ToolCallContext {
            parameters: None,
            execution_context: ExecutionContext::new(),
        };
        let result = validator.validate_tool_call("", "test-agent", &context);
        assert!(result.is_err()); // Should fail because empty tool is not in allowlist

        // Test case sensitivity
        let result = validator.validate_tool_call("READ", "test-agent", &context);
        assert!(result.is_err()); // Should fail because case doesn't match
    }

    #[test]
    fn test_integration_with_subagent_spec() {
        // Test that our validator works with real SubagentSpec instances
        let spec = SubagentBuilder::new("integration-test")
            .instructions("Integration test instructions")
            .description("Test agent for integration testing")
            .tools(vec!["read".to_string(), "write".to_string(), "analysis".to_string()])
            .source_path(PathBuf::from("test.md"))
            .build()
            .unwrap();

        let validator = ToolPolicyValidator::new();
        let report = validator.validate_subagent_policy(&spec).unwrap();

        assert!(report.is_safe());
        assert_eq!(report.approved_tools.len(), 3);
        assert_eq!(spec.name(), "integration-test");
        assert_eq!(spec.tools().len(), 3);
    }
}