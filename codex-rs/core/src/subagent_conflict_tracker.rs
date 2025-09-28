use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use codex_protocol::mcp_protocol::ConversationId;
use crate::protocol::FileChange;
use crate::protocol::PatchApplyBeginEvent;
use crate::protocol::PatchApplyEndEvent;

/// Tracks which subagents have modified which files to detect conflicts.
#[derive(Debug, Clone)]
pub struct SubagentConflictTracker {
    inner: Arc<Mutex<ConflictTrackerInner>>,
}

#[derive(Debug)]
struct ConflictTrackerInner {
    /// Maps file paths to the most recent subagent that modified them.
    file_attributions: HashMap<PathBuf, SubagentFileAttribution>,
    /// Active patch applications by call_id for tracking in-progress edits.
    active_patches: HashMap<String, PatchInProgress>,
    /// History of all patch applications for rollback support.
    patch_history: Vec<CompletedPatch>,
}

#[derive(Debug, Clone)]
struct SubagentFileAttribution {
    agent_name: String,
    sub_conversation_id: ConversationId,
    call_id: String,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
struct PatchInProgress {
    agent_name: Option<String>,
    sub_conversation_id: Option<ConversationId>,
    affected_files: HashSet<PathBuf>,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct CompletedPatch {
    pub call_id: String,
    pub agent_name: Option<String>,
    pub sub_conversation_id: Option<ConversationId>,
    pub affected_files: Vec<PathBuf>,
    pub success: bool,
    pub timestamp: std::time::SystemTime,
}

/// Result of conflict detection when a new patch is being applied.
#[derive(Debug, Clone)]
pub enum ConflictDetectionResult {
    /// No conflicts detected, safe to proceed.
    Clear,
    /// Warning: files modified by different subagents in this session.
    Warning {
        conflicting_files: Vec<FileConflict>,
        message: String,
    },
    /// Blocking conflict: another subagent is currently modifying these files.
    Blocked {
        conflicting_files: Vec<PathBuf>,
        blocking_agent: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct FileConflict {
    pub file_path: PathBuf,
    pub previous_agent: String,
    pub previous_timestamp: std::time::SystemTime,
}

impl SubagentConflictTracker {
    /// Create a new conflict tracker.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ConflictTrackerInner {
                file_attributions: HashMap::new(),
                active_patches: HashMap::new(),
                patch_history: Vec::new(),
            })),
        }
    }

    /// Check for conflicts before applying a patch.
    /// Should be called when receiving a PatchApplyBeginEvent.
    pub fn check_conflicts(&self, event: &PatchApplyBeginEvent) -> ConflictDetectionResult {
        let mut inner = self.inner.lock().unwrap();

        let affected_files: HashSet<PathBuf> = event.changes.keys().cloned().collect();
        let current_agent = event.originating_subagent.clone();

        // Check for active patches from other agents
        for (active_call_id, active_patch) in &inner.active_patches {
            if *active_call_id == event.call_id {
                continue; // Same patch, not a conflict
            }

            if let Some(active_agent) = &active_patch.agent_name {
                if current_agent.as_ref() != Some(active_agent) {
                    let conflicts: Vec<PathBuf> = affected_files
                        .intersection(&active_patch.affected_files)
                        .cloned()
                        .collect();

                    if !conflicts.is_empty() {
                        return ConflictDetectionResult::Blocked {
                            conflicting_files: conflicts,
                            blocking_agent: active_agent.clone(),
                            message: format!(
                                "Cannot apply patch: {} is currently modifying {} files that {} is trying to edit",
                                active_agent,
                                conflicts.len(),
                                current_agent.as_deref().unwrap_or("main agent")
                            ),
                        };
                    }
                }
            }
        }

        // Check for attribution warnings (different agents modified same files)
        let mut conflicting_files = Vec::new();
        for file_path in &affected_files {
            if let Some(attribution) = inner.file_attributions.get(file_path) {
                if let Some(current_agent_name) = &current_agent {
                    if attribution.agent_name != *current_agent_name {
                        conflicting_files.push(FileConflict {
                            file_path: file_path.clone(),
                            previous_agent: attribution.agent_name.clone(),
                            previous_timestamp: attribution.timestamp,
                        });
                    }
                }
            }
        }

        if conflicting_files.is_empty() {
            ConflictDetectionResult::Clear
        } else {
            ConflictDetectionResult::Warning {
                message: format!(
                    "Warning: {} files were previously modified by different subagents in this session",
                    conflicting_files.len()
                ),
                conflicting_files,
            }
        }
    }

    /// Record the start of a patch application.
    /// Should be called when processing a PatchApplyBeginEvent.
    pub fn record_patch_begin(&self, event: &PatchApplyBeginEvent) {
        let mut inner = self.inner.lock().unwrap();

        let affected_files: HashSet<PathBuf> = event.changes.keys().cloned().collect();

        let patch_in_progress = PatchInProgress {
            agent_name: event.originating_subagent.clone(),
            sub_conversation_id: event.sub_conversation_id.as_ref().map(|s| ConversationId::from_string(s.clone()).unwrap()),
            affected_files,
            timestamp: std::time::SystemTime::now(),
        };

        inner.active_patches.insert(event.call_id.clone(), patch_in_progress);
    }

    /// Record the completion of a patch application and update attributions.
    /// Should be called when processing a PatchApplyEndEvent.
    pub fn record_patch_end(&self, event: &PatchApplyEndEvent) {
        let mut inner = self.inner.lock().unwrap();

        if let Some(active_patch) = inner.active_patches.remove(&event.call_id) {
            if event.success {
                // Update file attributions for successful patches
                if let Some(agent_name) = &active_patch.agent_name {
                    let sub_conversation_id = active_patch.sub_conversation_id.unwrap_or_else(||
                        ConversationId::from_string("main".to_string()).unwrap()
                    );

                    for file_path in &active_patch.affected_files {
                        inner.file_attributions.insert(
                            file_path.clone(),
                            SubagentFileAttribution {
                                agent_name: agent_name.clone(),
                                sub_conversation_id: sub_conversation_id.clone(),
                                call_id: event.call_id.clone(),
                                timestamp: active_patch.timestamp,
                            },
                        );
                    }
                }
            }

            // Add to patch history
            inner.patch_history.push(CompletedPatch {
                call_id: event.call_id.clone(),
                agent_name: active_patch.agent_name,
                sub_conversation_id: active_patch.sub_conversation_id,
                affected_files: active_patch.affected_files.into_iter().collect(),
                success: event.success,
                timestamp: active_patch.timestamp,
            });
        }
    }

    /// Get attribution information for a specific file.
    pub fn get_file_attribution(&self, file_path: &PathBuf) -> Option<SubagentFileAttribution> {
        let inner = self.inner.lock().unwrap();
        inner.file_attributions.get(file_path).cloned()
    }

    /// Get all patches by a specific subagent for rollback support.
    pub fn get_patches_by_agent(&self, agent_name: &str) -> Vec<CompletedPatch> {
        let inner = self.inner.lock().unwrap();
        inner.patch_history
            .iter()
            .filter(|patch| patch.agent_name.as_ref() == Some(agent_name))
            .cloned()
            .collect()
    }

    /// Clear all tracking data (useful for testing or session reset).
    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.file_attributions.clear();
        inner.active_patches.clear();
        inner.patch_history.clear();
    }

    /// Get summary statistics for status display.
    pub fn get_summary(&self) -> ConflictTrackerSummary {
        let inner = self.inner.lock().unwrap();
        ConflictTrackerSummary {
            total_files_tracked: inner.file_attributions.len(),
            active_patches: inner.active_patches.len(),
            total_patches_applied: inner.patch_history.len(),
            successful_patches: inner.patch_history.iter().filter(|p| p.success).count(),
        }
    }
}

impl Default for SubagentConflictTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ConflictTrackerSummary {
    pub total_files_tracked: usize,
    pub active_patches: usize,
    pub total_patches_applied: usize,
    pub successful_patches: usize,
}