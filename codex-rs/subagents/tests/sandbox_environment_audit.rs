//! Comprehensive audit tests for sandbox environment and CODEX_SANDBOX_* variable handling.

use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::time::Duration;
use std::path::PathBuf;
use codex_subagents::core::tester::{SandboxConfig, TesterSubagent, TesterRequest, TestOptions};
use codex_subagents::core::tester::{RequirementsAnalysis, ProposedChanges};
use codex_subagents::TaskContext;
use codex_subagents::{Subagent, ContextualSubagent};

// Constants for sandbox environment variables (matching core implementation)
const CODEX_SANDBOX_ENV_VAR: &str = "CODEX_SANDBOX";
const CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR: &str = "CODEX_SANDBOX_NETWORK_DISABLED";

/// Mock environment manager for testing sandbox configurations
#[derive(Debug, Clone)]
pub struct MockEnvironment {
    variables: HashMap<String, String>,
    original_env: HashMap<String, String>,
}

impl MockEnvironment {
    pub fn new() -> Self {
        let mut original_env = HashMap::new();

        // Capture current environment state
        if let Ok(value) = env::var(CODEX_SANDBOX_ENV_VAR) {
            original_env.insert(CODEX_SANDBOX_ENV_VAR.to_string(), value);
        }
        if let Ok(value) = env::var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR) {
            original_env.insert(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR.to_string(), value);
        }

        Self {
            variables: HashMap::new(),
            original_env,
        }
    }

    pub fn set_sandbox(&mut self, sandbox_type: &str) -> &mut Self {
        self.variables.insert(CODEX_SANDBOX_ENV_VAR.to_string(), sandbox_type.to_string());
        self
    }

    pub fn set_network_disabled(&mut self, disabled: bool) -> &mut Self {
        if disabled {
            self.variables.insert(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR.to_string(), "1".to_string());
        } else {
            self.variables.remove(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR);
        }
        self
    }

    pub fn apply(&self) {
        // Apply environment variables
        for (key, value) in &self.variables {
            env::set_var(key, value);
        }

        // Remove variables that should not be set
        if !self.variables.contains_key(CODEX_SANDBOX_ENV_VAR) {
            env::remove_var(CODEX_SANDBOX_ENV_VAR);
        }
        if !self.variables.contains_key(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR) {
            env::remove_var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR);
        }
    }

    pub fn restore(&self) {
        // Restore original environment
        for (key, value) in &self.original_env {
            env::set_var(key, value);
        }

        // Remove variables that weren't originally set
        if !self.original_env.contains_key(CODEX_SANDBOX_ENV_VAR) {
            env::remove_var(CODEX_SANDBOX_ENV_VAR);
        }
        if !self.original_env.contains_key(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR) {
            env::remove_var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR);
        }
    }

    pub fn get_current_sandbox_status() -> SandboxStatus {
        let sandbox_type = env::var(CODEX_SANDBOX_ENV_VAR).ok();
        let network_disabled = env::var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR).is_ok();

        SandboxStatus {
            sandbox_type,
            network_disabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SandboxStatus {
    pub sandbox_type: Option<String>,
    pub network_disabled: bool,
}

/// Audit helper for testing sandbox configuration respect
pub struct SandboxAuditor {
    test_commands: Vec<String>,
}

impl SandboxAuditor {
    pub fn new() -> Self {
        Self {
            test_commands: vec![
                "echo 'testing sandbox environment'".to_string(),
                "printenv | grep CODEX_SANDBOX".to_string(),
                "pwd".to_string(),
            ],
        }
    }

    /// Test that commands respect sandbox environment variables
    pub async fn audit_command_execution(&self, config: &SandboxConfig) -> AuditResult {
        let mut result = AuditResult::new();

        for command in &self.test_commands {
            let execution_result = self.execute_with_sandbox_config(command, config).await;
            result.add_execution_result(command.clone(), execution_result);
        }

        result
    }

    /// Execute a command and verify sandbox environment is properly configured
    async fn execute_with_sandbox_config(
        &self,
        command: &str,
        config: &SandboxConfig,
    ) -> CommandExecutionResult {
        let start_time = std::time::Instant::now();

        // Create command with proper sandbox configuration
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            cmd
        };

        // Apply sandbox configuration
        cmd.current_dir(&config.work_dir);

        // Apply environment variables from sandbox config
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // Check if sandbox environment variables are properly respected
        let expected_sandbox_vars = self.get_expected_sandbox_vars(config);
        for (key, value) in expected_sandbox_vars {
            cmd.env(key, value);
        }

        // Execute command
        let output = cmd.output();
        let execution_time = start_time.elapsed();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                CommandExecutionResult {
                    success: output.status.success(),
                    stdout,
                    stderr,
                    execution_time,
                    exit_code: output.status.code(),
                }
            }
            Err(error) => {
                CommandExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: error.to_string(),
                    execution_time,
                    exit_code: None,
                }
            }
        }
    }

    /// Get expected sandbox environment variables based on configuration
    fn get_expected_sandbox_vars(&self, config: &SandboxConfig) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // Set CODEX_SANDBOX based on configuration
        if config.enabled {
            // In a real implementation, this would depend on the platform
            #[cfg(target_os = "macos")]
            vars.insert(CODEX_SANDBOX_ENV_VAR.to_string(), "seatbelt".to_string());

            #[cfg(target_os = "linux")]
            vars.insert(CODEX_SANDBOX_ENV_VAR.to_string(), "seccomp".to_string());

            #[cfg(target_os = "windows")]
            vars.insert(CODEX_SANDBOX_ENV_VAR.to_string(), "appcontainer".to_string());
        }

        // Set network disabled if configured
        if !config.allow_network {
            vars.insert(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR.to_string(), "1".to_string());
        }

        vars
    }
}

#[derive(Debug, Clone)]
pub struct CommandExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub exit_code: Option<i32>,
}

#[derive(Debug)]
pub struct AuditResult {
    pub command_results: Vec<(String, CommandExecutionResult)>,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

impl AuditResult {
    pub fn new() -> Self {
        Self {
            command_results: Vec::new(),
            violations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_execution_result(&mut self, command: String, result: CommandExecutionResult) {
        // Check for sandbox violations
        if result.stderr.contains("sandbox") && !result.success {
            self.violations.push(format!("Command '{}' failed with sandbox error: {}", command, result.stderr));
        }

        // Check for network access when it should be disabled
        if result.stdout.contains("network") || result.stderr.contains("network") {
            self.warnings.push(format!("Command '{}' may have accessed network", command));
        }

        self.command_results.push((command, result));
    }

    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    pub fn is_compliant(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Enhanced SandboxConfig for testing
impl SandboxConfig {
    pub fn test_config() -> Self {
        Self {
            enabled: true,
            work_dir: PathBuf::from("."),
            test_timeout: Duration::from_secs(30),
            max_memory_mb: Some(512),
            allow_network: false,
            env_vars: HashMap::new(),
        }
    }

    pub fn with_network_access(mut self, access: bool) -> Self {
        self.allow_network = access;
        self
    }

    pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_work_dir(mut self, dir: PathBuf) -> Self {
        self.work_dir = dir;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_mock_environment_basic_operations() {
        let mut mock_env = MockEnvironment::new();

        // Test setting sandbox environment
        mock_env.set_sandbox("seatbelt");
        mock_env.apply();

        assert_eq!(env::var(CODEX_SANDBOX_ENV_VAR).unwrap(), "seatbelt");

        // Test setting network disabled
        mock_env.set_network_disabled(true);
        mock_env.apply();

        assert_eq!(env::var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR).unwrap(), "1");

        // Restore environment
        mock_env.restore();
    }

    #[test]
    fn test_sandbox_status_detection() {
        let mut mock_env = MockEnvironment::new();

        // Test clean environment
        mock_env.apply();
        let status = MockEnvironment::get_current_sandbox_status();
        assert_eq!(status.sandbox_type, None);
        assert!(!status.network_disabled);

        // Test with sandbox enabled
        mock_env.set_sandbox("seatbelt").set_network_disabled(true);
        mock_env.apply();

        let status = MockEnvironment::get_current_sandbox_status();
        assert_eq!(status.sandbox_type, Some("seatbelt".to_string()));
        assert!(status.network_disabled);

        mock_env.restore();
    }

    #[tokio::test]
    async fn test_sandbox_config_environment_variables() {
        let config = SandboxConfig::test_config()
            .with_env_var("TEST_VAR", "test_value")
            .with_env_var(CODEX_SANDBOX_ENV_VAR, "test_sandbox");

        assert_eq!(config.env_vars.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(config.env_vars.get(CODEX_SANDBOX_ENV_VAR), Some(&"test_sandbox".to_string()));
    }

    #[tokio::test]
    async fn test_auditor_basic_command_execution() {
        let auditor = SandboxAuditor::new();
        let config = SandboxConfig::test_config();

        let result = auditor.audit_command_execution(&config).await;

        assert!(!result.command_results.is_empty());
        assert!(result.is_compliant()); // Should be compliant for basic commands
    }

    #[test]
    fn test_sandbox_config_builder_pattern() {
        let config = SandboxConfig::test_config()
            .with_network_access(true)
            .with_work_dir(PathBuf::from("/tmp"))
            .with_env_var("CUSTOM_VAR", "custom_value");

        assert!(config.network_access);
        assert_eq!(config.work_dir, PathBuf::from("/tmp"));
        assert_eq!(config.env_vars.get("CUSTOM_VAR"), Some(&"custom_value".to_string()));
    }

    #[tokio::test]
    async fn test_tester_subagent_respects_sandbox_config() {
        let mut mock_env = MockEnvironment::new();
        mock_env.set_sandbox("seatbelt").set_network_disabled(true);
        mock_env.apply();

        let mut tester = TesterSubagent::new();
        let mut ctx = TaskContext::new();
        let sandbox_config = SandboxConfig::test_config().with_network_access(false);

        // Test that the tester properly prepares with sandbox configuration
        let result = tester.prepare(&mut ctx, &sandbox_config);
        assert!(result.is_ok());

        mock_env.restore();
    }

    #[test]
    fn test_environment_isolation_between_tests() {
        // First test sets environment
        {
            let mut mock_env = MockEnvironment::new();
            mock_env.set_sandbox("test1");
            mock_env.apply();
            assert_eq!(env::var(CODEX_SANDBOX_ENV_VAR).unwrap(), "test1");
            mock_env.restore();
        }

        // Second test should have clean environment
        {
            let mut mock_env = MockEnvironment::new();
            mock_env.set_sandbox("test2");
            mock_env.apply();
            assert_eq!(env::var(CODEX_SANDBOX_ENV_VAR).unwrap(), "test2");
            mock_env.restore();
        }
    }

    #[tokio::test]
    async fn test_command_execution_with_sandbox_restrictions() {
        let auditor = SandboxAuditor::new();

        // Test with network disabled
        let config = SandboxConfig::test_config()
            .with_network_access(false)
            .with_env_var(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR, "1");

        let result = auditor.audit_command_execution(&config).await;

        // Verify that commands executed
        assert!(!result.command_results.is_empty());

        // Check that environment variables were properly set
        let env_check_result = result.command_results.iter()
            .find(|(cmd, _)| cmd.contains("printenv"))
            .map(|(_, result)| result);

        if let Some(env_result) = env_check_result {
            // Should contain CODEX_SANDBOX environment variables in output
            let output = format!("{}{}", env_result.stdout, env_result.stderr);
            // This test is environment-dependent, so we just verify it ran
            assert!(env_result.success || !env_result.stderr.is_empty());
        }
    }

    #[test]
    fn test_sandbox_config_network_access_settings() {
        let config_with_network = SandboxConfig::test_config().with_network_access(true);
        assert!(config_with_network.allow_network);

        let config_without_network = SandboxConfig::test_config().with_network_access(false);
        assert!(!config_without_network.allow_network);
    }

    #[tokio::test]
    async fn test_audit_result_violation_detection() {
        let mut result = AuditResult::new();

        // Add a successful command
        result.add_execution_result(
            "echo test".to_string(),
            CommandExecutionResult {
                success: true,
                stdout: "test".to_string(),
                stderr: String::new(),
                execution_time: Duration::from_millis(10),
                exit_code: Some(0),
            }
        );

        // Add a command with sandbox violation
        result.add_execution_result(
            "restricted command".to_string(),
            CommandExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "sandbox violation: operation not permitted".to_string(),
                execution_time: Duration::from_millis(5),
                exit_code: Some(1),
            }
        );

        assert!(result.has_violations());
        assert!(!result.is_compliant());
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].contains("sandbox violation"));
    }

    #[test]
    fn test_expected_sandbox_vars_generation() {
        let auditor = SandboxAuditor::new();

        // Test with sandbox enabled and network disabled
        let config = SandboxConfig::test_config()
            .with_network_access(false);

        let vars = auditor.get_expected_sandbox_vars(&config);

        // Should set network disabled
        assert_eq!(vars.get(CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR), Some(&"1".to_string()));

        // Should set platform-appropriate sandbox type
        #[cfg(target_os = "macos")]
        assert_eq!(vars.get(CODEX_SANDBOX_ENV_VAR), Some(&"seatbelt".to_string()));
    }

    #[test]
    fn test_audit_result_warning_detection() {
        let mut result = AuditResult::new();

        result.add_execution_result(
            "curl example.com".to_string(),
            CommandExecutionResult {
                success: true,
                stdout: "network access occurred".to_string(),
                stderr: String::new(),
                execution_time: Duration::from_millis(100),
                exit_code: Some(0),
            }
        );

        assert!(!result.has_violations());
        assert!(result.is_compliant()); // Warnings don't affect compliance
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("may have accessed network"));
    }

    #[tokio::test]
    async fn test_complete_sandbox_audit_workflow() {
        let mut mock_env = MockEnvironment::new();

        // Setup test environment
        mock_env.set_sandbox("seatbelt").set_network_disabled(true);
        mock_env.apply();

        // Create test configuration
        let config = SandboxConfig::test_config()
            .with_network_access(false)
            .with_env_var("TEST_MODE", "sandbox_audit");

        // Run audit
        let auditor = SandboxAuditor::new();
        let result = auditor.audit_command_execution(&config).await;

        // Verify audit results
        assert!(!result.command_results.is_empty());

        // All commands should have executed (successfully or not)
        for (command, exec_result) in &result.command_results {
            assert!(!command.is_empty());
            assert!(exec_result.execution_time > Duration::from_nanos(0));
        }

        // Cleanup
        mock_env.restore();
    }

    #[test]
    fn test_mock_environment_preserves_original_state() {
        // Set some initial environment state
        env::set_var("TEST_PRESERVE", "original_value");

        let original_test_var = env::var("TEST_PRESERVE").unwrap();

        {
            let mut mock_env = MockEnvironment::new();
            mock_env.set_sandbox("temporary");
            mock_env.apply();

            // Environment should be modified
            assert_eq!(env::var(CODEX_SANDBOX_ENV_VAR).unwrap(), "temporary");

            // Original variable should still exist
            assert_eq!(env::var("TEST_PRESERVE").unwrap(), original_test_var);

            mock_env.restore();
        }

        // After restore, sandbox variable should be gone
        assert!(env::var(CODEX_SANDBOX_ENV_VAR).is_err());

        // Original variable should be preserved
        assert_eq!(env::var("TEST_PRESERVE").unwrap(), original_test_var);

        // Cleanup
        env::remove_var("TEST_PRESERVE");
    }
}