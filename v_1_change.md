# Subagents Architecture Implementation Analysis for codex-cv

> Analysis of implementing Claude Code's subagents system in codex-cv (forked from OpenAI Codex) to leverage GPT-5 and GPT-5-Codex capabilities.

## **Current Architecture Assessment**

**✅ Strong Foundation:**
- **Rust Core + TypeScript Frontend**: Hybrid architecture already supports complex orchestration
- **MCP Integration**: Existing MCP client/server infrastructure can be leveraged
- **Protocol System**: Well-defined protocol layer for message passing
- **Conversation Management**: `ConversationManager` and `CodexConversation` provide conversation primitives
- **Slash Commands**: `/mcp`, `/model`, etc. - infrastructure exists for new commands
- **Custom Prompts**: `custom_prompts.rs` shows pattern for file-based configuration

**🚧 Missing Components:**
- No existing agent delegation system
- No separate context management per subagent
- No YAML frontmatter parsing for agent configs
- No dynamic tool permission system

## **Implementation Scope: SIGNIFICANT LIFT** 

This would be a **major architectural addition** requiring changes across all layers.

## **Required Changes by Component**

### **1. Core Protocol Extensions (Rust)**
**Files to modify:** `protocol/src/models.rs`, `protocol/src/protocol.rs`

```rust
// New protocol messages needed
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SubagentConfig {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub tools: Option<Vec<String>>, // None = inherit all
    pub model: Option<String>,
    pub scope: SubagentScope, // Project vs User
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub enum SubagentScope {
    Project, // .composer/agents/
    User,    // ~/.composer/agents/
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DelegateToSubagentRequest {
    pub subagent_name: String,
    pub task_description: String,
    pub context: Option<Vec<String>>,
}
```

### **2. Subagent Manager (New Rust Crate)**
**New crate:** `codex-subagents/`

```rust
// Core subagent management
pub struct SubagentManager {
    project_agents: HashMap<String, SubagentConfig>,
    user_agents: HashMap<String, SubagentConfig>,
    active_contexts: HashMap<String, ConversationId>, // subagent -> context
}

impl SubagentManager {
    pub async fn load_agents(&mut self, project_path: &Path) -> Result<()> {
        // Load from .codex-cv/agents/ and ~/.codex-cv/agents/
        // Parse YAML frontmatter + markdown
    }
    
    pub async fn delegate_task(&self, request: DelegateToSubagentRequest) -> Result<ConversationId> {
        // Create new conversation context for subagent
        // Apply tool restrictions
        // Set custom system prompt
    }
    
    pub fn should_delegate(&self, task: &str) -> Option<String> {
        // Smart matching logic for automatic delegation
    }
}
```

### **3. YAML Frontmatter Parser (New)**
Currently missing - would need to add markdown + YAML parsing:

```rust
// New dependency: serde-yaml, markdown-parser
pub struct AgentFile {
    pub config: SubagentConfig,
    pub system_prompt: String,
}

pub fn parse_agent_file(content: &str) -> Result<AgentFile> {
    // Parse YAML frontmatter
    // Extract markdown system prompt
}
```

### **4. Enhanced Conversation Manager**
**Modify:** `core/src/conversation_manager.rs`

```rust
impl ConversationManager {
    // NEW: Create subagent conversation with restricted tools
    pub async fn create_subagent_conversation(
        &self,
        config: &SubagentConfig,
        parent_id: Option<ConversationId>,
    ) -> Result<NewConversation> {
        // Create isolated context
        // Apply tool restrictions 
        // Set custom system prompt
    }
}
```

### **5. Tool Permission System (Major)**
**New functionality** - Currently tools are all-or-nothing:

```rust
pub struct ToolPermissions {
    allowed_tools: HashSet<String>,
    inherit_all: bool,
}

impl ToolPermissions {
    pub fn can_use_tool(&self, tool_name: &str) -> bool {
        self.inherit_all || self.allowed_tools.contains(tool_name)
    }
}
```

### **6. New Slash Commands**
**Modify:** `tui/src/slash_command.rs`

```rust
pub enum SlashCommand {
    // ... existing commands
    Agents,      // /agents - manage subagents
    Delegate,    // /delegate <agent> <task> - explicit delegation
}
```

### **7. Frontend Integration (TypeScript)**
**Major changes needed in:** `src/core/controller/`

```typescript
// New agent management UI
interface SubagentConfig {
  name: string;
  description: string;
  systemPrompt: string;
  tools?: string[];
  model?: string;
  scope: 'project' | 'user';
}

class SubagentController {
  async listAgents(): Promise<SubagentConfig[]> {}
  async createAgent(config: SubagentConfig): Promise<void> {}
  async delegateTask(agentName: string, task: string): Promise<void> {}
}
```

### **8. File System Integration**
**New directories to support:**
```
~/.codex-cv/agents/           # User-level agents
.codex-cv/agents/            # Project-level agents
```

**File watcher needed** to reload agents when files change.

## **Complexity Assessment**

### **HIGH COMPLEXITY AREAS:**

1. **Context Isolation (★★★★★)**
   - Each subagent needs separate conversation context
   - Memory management for multiple active contexts
   - Context cleanup and lifecycle management

2. **Tool Permission System (★★★★☆)**
   - Currently no granular tool restrictions
   - Need to modify every tool invocation point
   - MCP tool filtering integration

3. **Smart Delegation Logic (★★★★☆)**
   - Natural language matching for automatic delegation
   - Task classification and routing
   - Conflict resolution when multiple agents match

4. **Frontend UI Complexity (★★★☆☆)**
   - Agent creation/editing interface
   - Tool permission selection UI
   - Active context management display

### **MEDIUM COMPLEXITY AREAS:**

1. **YAML Frontmatter Parsing (★★★☆☆)**
   - Standard markdown/YAML parsing
   - Validation and error handling

2. **File System Management (★★☆☆☆)**
   - Directory structure creation
   - File watching and reloading

3. **Configuration Integration (★★☆☆☆)**
   - Extend existing config system
   - Agent discovery and loading

## **Optimal Implementation Strategy for GPT-5/GPT-5-Codex**

### **Phase 1: Foundation (4-6 weeks)**
1. **YAML frontmatter parser** + basic agent loading
2. **New slash command** `/agents` for basic management
3. **Subagent configuration storage** in `~/.codex-cv/agents/`
4. **Basic delegation** (manual only, no auto-routing)

### **Phase 2: Core Functionality (6-8 weeks)**
1. **Context isolation** - separate conversation contexts
2. **Tool permission system** - granular tool access control
3. **Model selection** per subagent
4. **Frontend UI** for agent management

### **Phase 3: Intelligence (4-6 weeks)**
1. **Auto-delegation logic** using GPT-5's reasoning capabilities  
2. **Context-aware routing** based on task analysis
3. **Agent chaining** for complex workflows
4. **Performance optimization**

### **Phase 4: GPT-5 Optimization (2-4 weeks)**
1. **Advanced reasoning prompts** for delegation decisions
2. **Dynamic agent creation** via GPT-5-Codex
3. **Self-improving agents** that learn from outcomes
4. **Multi-step planning** across agent boundaries

## **GPT-5/GPT-5-Codex Specific Advantages**

### **🧠 Reasoning Capabilities**
```rust
// Leverage GPT-5's reasoning for smart delegation
pub async fn should_delegate_with_reasoning(&self, task: &str, context: &[String]) -> Option<DelegationDecision> {
    let prompt = format!(
        "Analyze this task and determine which specialized agent should handle it:
        Task: {task}
        Context: {context:?}
        Available agents: {agents:?}
        
        Think through:
        1. Task complexity and domain
        2. Required tools and permissions  
        3. Context preservation needs
        4. Expected outcome quality
        
        Provide reasoning and recommendation."
    );
    
    // GPT-5 provides detailed reasoning about delegation choice
}
```

### **🤖 Codex Integration**
```rust
// Use GPT-5-Codex for dynamic agent generation
pub async fn generate_agent_for_task(&self, task_description: &str) -> Result<SubagentConfig> {
    let prompt = format!(
        "Generate a specialized subagent configuration for this task:
        {task_description}
        
        Return YAML frontmatter + system prompt optimized for:
        - Task-specific expertise
        - Minimal tool requirements
        - Clear success criteria
        - Robust error handling"
    );
    
    // GPT-5-Codex generates optimized agent configs
}
```

## **Example Subagent Configurations**

### **Code Reviewer Agent**
```markdown
---
name: code-reviewer
description: Expert code review specialist. Proactively reviews code for quality, security, and maintainability. Use immediately after writing or modifying code.
tools: Read, Grep, Glob, Bash
model: sonnet
---

You are a senior code reviewer ensuring high standards of code quality and security.

When invoked:
1. Run git diff to see recent changes
2. Focus on modified files
3. Begin review immediately

Review checklist:
- Code is simple and readable
- Functions and variables are well-named
- No duplicated code
- Proper error handling
- No exposed secrets or API keys
- Input validation implemented
- Good test coverage
- Performance considerations addressed

Provide feedback organized by priority:
- Critical issues (must fix)
- Warnings (should fix)
- Suggestions (consider improving)

Include specific examples of how to fix issues.
```

### **Debugger Agent**
```markdown
---
name: debugger
description: Debugging specialist for errors, test failures, and unexpected behavior. Use proactively when encountering any issues.
tools: Read, Edit, Bash, Grep, Glob
model: inherit
---

You are an expert debugger specializing in root cause analysis.

When invoked:
1. Capture error message and stack trace
2. Identify reproduction steps
3. Isolate the failure location
4. Implement minimal fix
5. Verify solution works

Debugging process:
- Analyze error messages and logs
- Check recent code changes
- Form and test hypotheses
- Add strategic debug logging
- Inspect variable states

For each issue, provide:
- Root cause explanation
- Evidence supporting the diagnosis
- Specific code fix
- Testing approach
- Prevention recommendations

Focus on fixing the underlying issue, not just symptoms.
```

### **Rust Performance Agent**
```markdown
---
name: rust-perf-optimizer
description: Rust performance optimization specialist. Use for performance bottlenecks, memory issues, and optimization tasks.
tools: Read, Edit, Bash, Grep
model: gpt-5-codex
---

You are a Rust performance optimization expert specializing in:

- Memory allocation patterns and zero-copy optimizations
- Async/await performance and tokio runtime tuning
- SIMD vectorization and CPU cache optimization
- Compiler optimization hints and profile-guided optimization
- Benchmarking with criterion.rs and performance regression detection

When invoked:
1. Profile the code to identify bottlenecks
2. Analyze memory allocation patterns
3. Look for unnecessary clones and allocations
4. Check for optimal data structures and algorithms
5. Implement performance improvements
6. Verify improvements with benchmarks

Always measure before and after optimization, and ensure correctness is preserved.
```

## **Integration with codex-cv's Existing Architecture**

### **Leveraging MCP for Subagents**
```rust
// Subagents can be MCP servers themselves
pub struct SubagentMcpServer {
    agent_config: SubagentConfig,
    conversation_id: ConversationId,
}

impl McpServer for SubagentMcpServer {
    async fn call_tool(&self, name: &str, args: Value) -> Result<CallToolResult> {
        // Route tool calls through subagent's restricted permissions
        if !self.agent_config.can_use_tool(name) {
            return Err("Tool not allowed for this subagent".into());
        }
        
        // Execute with subagent's context
        self.execute_in_context(name, args).await
    }
}
```

### **TUI Integration**
```rust
// Enhanced TUI with subagent status
impl App {
    fn render_subagent_status(&mut self, f: &mut Frame, area: Rect) {
        let active_agents: Vec<String> = self.subagent_manager
            .active_contexts()
            .keys()
            .cloned()
            .collect();
            
        if !active_agents.is_empty() {
            let status = format!("Active agents: {}", active_agents.join(", "));
            let paragraph = Paragraph::new(status)
                .style(Style::default().fg(Color::Cyan))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, area);
        }
    }
}
```

## **File System Structure**

```
~/.codex-cv/
├── agents/                          # User-level agents
│   ├── code-reviewer.md
│   ├── debugger.md
│   ├── rust-perf-optimizer.md
│   └── data-scientist.md
└── config.toml

.codex-cv/                           # Project-level agents
├── agents/
│   ├── api-tester.md               # Project-specific API testing agent
│   ├── deployment-manager.md       # Deployment automation agent
│   └── migration-helper.md         # Database migration specialist
└── config.toml
```

## **Performance Considerations**

### **Context Management**
- **Memory Efficiency**: Each subagent context should be lazily loaded
- **Context Cleanup**: Automatic cleanup of inactive subagent contexts
- **Context Sharing**: Read-only context sharing between related agents

### **Latency Optimization**
- **Agent Preloading**: Keep frequently-used agents warm
- **Batch Operations**: Group related subagent operations
- **Caching**: Cache agent configurations and compiled prompts

## **Security Implications**

### **Tool Isolation**
```rust
// Secure tool access control
impl ToolPermissions {
    pub fn validate_tool_call(&self, tool_name: &str, args: &Value) -> Result<()> {
        if !self.can_use_tool(tool_name) {
            return Err(SecurityError::ToolNotAllowed(tool_name.to_string()));
        }
        
        // Additional validation for dangerous operations
        match tool_name {
            "bash" => self.validate_bash_command(args)?,
            "edit" => self.validate_file_access(args)?,
            _ => {}
        }
        
        Ok(())
    }
}
```

### **Sandboxing**
- **File System Access**: Restrict agents to specific directories
- **Network Access**: Control which agents can make external requests
- **Process Execution**: Limit command execution capabilities per agent

## **Testing Strategy**

### **Unit Tests**
```rust
#[tokio::test]
async fn test_subagent_delegation() {
    let manager = SubagentManager::new();
    manager.load_agents(&test_project_path()).await.unwrap();
    
    let decision = manager.should_delegate("fix the failing tests").await;
    assert_eq!(decision, Some("test-runner".to_string()));
}

#[tokio::test] 
async fn test_tool_permission_enforcement() {
    let config = SubagentConfig {
        tools: Some(vec!["read".to_string(), "grep".to_string()]),
        ..Default::default()
    };
    
    let permissions = ToolPermissions::from_config(&config);
    assert!(permissions.can_use_tool("read"));
    assert!(!permissions.can_use_tool("bash"));
}
```

### **Integration Tests**
- **End-to-end agent workflows**
- **Context isolation verification**  
- **Tool permission enforcement**
- **Agent chaining scenarios**

## **Migration and Rollout Plan**

### **Backward Compatibility**
- **Feature Flag**: `CODEX_CV_SUBAGENTS_ENABLED=1`
- **Graceful Degradation**: Fall back to main context if subagents fail
- **Progressive Rollout**: Start with power users and beta testers

### **Documentation Requirements**
- **Agent Creation Guide**: How to write effective subagents
- **Best Practices**: Tool selection and prompt engineering
- **Troubleshooting**: Common issues and debugging techniques
- **Migration Guide**: Converting existing workflows to use subagents

## **Final Assessment: SUBSTANTIAL BUT WORTHWHILE**

### **Effort Estimate:** 16-22 weeks for full implementation
### **Complexity Level:** ★★★★☆ (High)
### **Strategic Value:** ★★★★★ (Critical for competitive positioning)

### **Key Benefits:**
- **Superior Context Management**: Longer effective conversations through context isolation
- **Specialized Expertise**: Higher task success rates with domain-specific agents
- **Competitive Differentiation**: Unique selling proposition in the AI coding assistant market
- **GPT-5 Optimization**: Full leverage of reasoning capabilities for intelligent task routing
- **Scalability**: Framework for unlimited specialized capabilities

### **Risks:**
- **Complexity**: Significant increase in system complexity
- **Performance**: Potential latency overhead from context switching
- **User Experience**: Learning curve for effective agent utilization
- **Maintenance**: Ongoing effort to maintain and improve agent quality

### **Recommendation:**
**Implement in phases** starting with basic delegation and building up to GPT-5-powered intelligence. The subagents architecture would be a significant differentiator, especially with GPT-5's reasoning capabilities enabling sophisticated task routing and agent orchestration.

This should be considered a **major feature release** requiring dedicated development cycles and thorough testing across all supported platforms. However, the strategic value and competitive advantages justify the substantial investment.

---
