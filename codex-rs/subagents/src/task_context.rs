use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::collections::HashMap;
use std::env;
use std::ops::Deref;
use std::ops::DerefMut;

/// Container for passing shared state between subagents within a pipeline.
#[derive(Default)]
pub struct TaskContext {
    typed_slots: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    typed_slot_names: HashMap<TypeId, String>,
    scratchpads: HashMap<String, Value>,
    diagnostics: Vec<TaskDiagnostic>,
}

impl TaskContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a typed payload into the context, returning the previous value for that type.
    pub fn insert_typed<T>(&mut self, value: T) -> Option<T>
    where
        T: Any + Send + Sync,
    {
        let key = TypeId::of::<T>();
        self.typed_slot_names
            .insert(key, type_name::<T>().to_string());
        self.typed_slots
            .insert(key, Box::new(value))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Borrow a typed payload immutably.
    pub fn get_typed<T>(&self) -> Option<&T>
    where
        T: Any + Send + Sync,
    {
        self.typed_slots
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Borrow a typed payload mutably.
    pub fn get_typed_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any + Send + Sync,
    {
        self.typed_slots
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove and return a typed payload.
    pub fn take_typed<T>(&mut self) -> Option<T>
    where
        T: Any + Send + Sync,
    {
        self.typed_slots
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Access a scratchpad namespace. The guard writes back upon drop.
    pub fn scratchpad(&mut self, namespace: impl Into<String>) -> TaskScratchpadGuard<'_> {
        let key = namespace.into();
        TaskScratchpadGuard::new(key, &mut self.scratchpads)
    }

    /// Read-only view of the scratchpads map.
    pub fn scratchpads(&self) -> &HashMap<String, Value> {
        &self.scratchpads
    }

    /// Append a diagnostic message to the history.
    pub fn push_diagnostic(&mut self, level: DiagnosticLevel, message: impl Into<String>) {
        self.diagnostics.push(TaskDiagnostic {
            level,
            message: message.into(),
            timestamp: Utc::now(),
        });
    }

    pub fn diagnostics(&self) -> &[TaskDiagnostic] {
        &self.diagnostics
    }

    /// Return a JSON snapshot of the context state when `CODEX_DEBUG_SUBAGENTS=1`.
    pub fn debug_snapshot(&self) -> Option<String> {
        if !debug_enabled() {
            return None;
        }

        #[derive(Serialize)]
        struct Snapshot<'a> {
            typed_slots: Vec<&'a String>,
            scratchpads: &'a HashMap<String, Value>,
            diagnostics: &'a [TaskDiagnostic],
        }

        let typed_slots = self.typed_slot_names.values().collect();
        let snapshot = Snapshot {
            typed_slots,
            scratchpads: &self.scratchpads,
            diagnostics: &self.diagnostics,
        };

        serde_json::to_string_pretty(&snapshot).ok()
    }
}

fn debug_enabled() -> bool {
    matches!(
        env::var("CODEX_DEBUG_SUBAGENTS").ok().as_deref(),
        Some("1" | "true" | "TRUE")
    )
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

pub struct TaskScratchpadGuard<'a> {
    key: String,
    map: &'a mut HashMap<String, Value>,
    value: Value,
    dirty: bool,
}

impl<'a> TaskScratchpadGuard<'a> {
    fn new(key: String, map: &'a mut HashMap<String, Value>) -> Self {
        let value = map
            .remove(&key)
            .unwrap_or_else(|| Value::Object(Default::default()));
        Self {
            key,
            map,
            value,
            dirty: true, // ensure existing value is preserved even if never mutated
        }
    }

    pub fn into_inner(mut self) -> Value {
        self.dirty = false;
        std::mem::take(&mut self.value)
    }
}

impl<'a> Deref for TaskScratchpadGuard<'a> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> DerefMut for TaskScratchpadGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.value
    }
}

impl<'a> Drop for TaskScratchpadGuard<'a> {
    fn drop(&mut self) {
        if self.dirty {
            let value = std::mem::take(&mut self.value);
            self.map.insert(self.key.clone(), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq)]
    struct Marker(&'static str);

    #[test]
    fn typed_slots_round_trip() {
        let mut ctx = TaskContext::new();
        assert!(ctx.insert_typed(Marker("alpha")).is_none());
        assert_eq!(ctx.get_typed::<Marker>(), Some(&Marker("alpha")));
        ctx.insert_typed(Marker("beta"));
        assert_eq!(ctx.get_typed::<Marker>(), Some(&Marker("beta")));
        assert_eq!(ctx.take_typed::<Marker>(), Some(Marker("beta")));
        assert!(ctx.get_typed::<Marker>().is_none());
    }

    #[test]
    fn scratchpad_guard_persists_changes() {
        let mut ctx = TaskContext::new();
        {
            let mut guard = ctx.scratchpad("notes");
            *guard = serde_json::json!({"lines": ["a", "b"]});
        }
        assert_eq!(
            ctx.scratchpads()["notes"],
            serde_json::json!({"lines": ["a", "b"]})
        );
    }

    #[test]
    fn diagnostics_capture_message() {
        let mut ctx = TaskContext::new();
        ctx.push_diagnostic(DiagnosticLevel::Info, "hello");
        assert_eq!(ctx.diagnostics().len(), 1);
        assert_eq!(ctx.diagnostics()[0].message, "hello");
    }
}
