use crate::error::TaskContextError;
use serde::Serialize;
use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::collections::HashMap;
use std::sync::RwLock;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticEntry {
    pub timestamp: OffsetDateTime,
    pub level: DiagnosticLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskContextSnapshot {
    pub typed_slots: Vec<String>,
    pub scratchpads: HashMap<String, serde_json::Value>,
    pub diagnostics: Vec<DiagnosticEntry>,
}

struct TypedSlot {
    type_name: &'static str,
    value: RwLock<Box<dyn Any + Send + Sync + 'static>>,
}

impl TypedSlot {
    fn new<T: Any + Send + Sync + 'static>(value: T) -> Self {
        Self {
            type_name: type_name::<T>(),
            value: RwLock::new(Box::new(value)),
        }
    }
}

#[derive(Default)]
pub struct TaskContext {
    typed_slots: RwLock<HashMap<TypeId, TypedSlot>>,
    scratchpads: RwLock<HashMap<String, serde_json::Value>>,
    diagnostics: RwLock<Vec<DiagnosticEntry>>,
}

impl std::fmt::Debug for TaskContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskContext").finish_non_exhaustive()
    }
}

impl TaskContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_typed<T>(&self, value: T) -> Result<(), TaskContextError>
    where
        T: Any + Send + Sync + 'static,
    {
        let mut slots = self
            .typed_slots
            .write()
            .map_err(|_| TaskContextError::Poisoned)?;
        slots.insert(TypeId::of::<T>(), TypedSlot::new(value));
        Ok(())
    }

    pub fn get_typed<T>(&self) -> Result<Option<T>, TaskContextError>
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let slots = self
            .typed_slots
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;
        Ok(slots
            .get(&TypeId::of::<T>())
            .and_then(|slot| slot.value.read().ok())
            .and_then(|guard| guard.downcast_ref::<T>().cloned()))
    }

    pub fn with_typed<T, R, F>(&self, f: F) -> Result<Option<R>, TaskContextError>
    where
        T: Any + Send + Sync + 'static,
        F: FnOnce(&T) -> R,
    {
        let slots = self
            .typed_slots
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;
        if let Some(slot) = slots.get(&TypeId::of::<T>()) {
            let guard = slot.value.read().map_err(|_| TaskContextError::Poisoned)?;
            let typed = guard
                .downcast_ref::<T>()
                .ok_or(TaskContextError::SlotDowncast {
                    expected: slot.type_name,
                })?;
            Ok(Some(f(typed)))
        } else {
            Ok(None)
        }
    }

    pub fn take_typed<T>(&self) -> Result<Option<T>, TaskContextError>
    where
        T: Any + Send + Sync + 'static,
    {
        let mut slots = self
            .typed_slots
            .write()
            .map_err(|_| TaskContextError::Poisoned)?;
        if let Some(slot) = slots.remove(&TypeId::of::<T>()) {
            let value = slot
                .value
                .into_inner()
                .map_err(|_| TaskContextError::Poisoned)?
                .downcast::<T>()
                .map_err(|_| TaskContextError::SlotDowncast {
                    expected: type_name::<T>(),
                })?;
            Ok(Some(*value))
        } else {
            Ok(None)
        }
    }

    pub fn set_scratchpad(
        &self,
        namespace: impl Into<String>,
        value: serde_json::Value,
    ) -> Result<(), TaskContextError> {
        let mut pads = self
            .scratchpads
            .write()
            .map_err(|_| TaskContextError::Poisoned)?;
        pads.insert(namespace.into(), value);
        Ok(())
    }

    pub fn get_scratchpad(
        &self,
        namespace: &str,
    ) -> Result<Option<serde_json::Value>, TaskContextError> {
        let pads = self
            .scratchpads
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;
        Ok(pads.get(namespace).cloned())
    }

    pub fn remove_scratchpad(
        &self,
        namespace: &str,
    ) -> Result<Option<serde_json::Value>, TaskContextError> {
        let mut pads = self
            .scratchpads
            .write()
            .map_err(|_| TaskContextError::Poisoned)?;
        Ok(pads.remove(namespace))
    }

    pub fn push_diagnostic(
        &self,
        level: DiagnosticLevel,
        message: impl Into<String>,
    ) -> Result<(), TaskContextError> {
        let mut diagnostics = self
            .diagnostics
            .write()
            .map_err(|_| TaskContextError::Poisoned)?;
        diagnostics.push(DiagnosticEntry {
            timestamp: OffsetDateTime::now_utc(),
            level,
            message: message.into(),
        });
        Ok(())
    }

    pub fn diagnostics(&self) -> Result<Vec<DiagnosticEntry>, TaskContextError> {
        let diagnostics = self
            .diagnostics
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;
        Ok(diagnostics.clone())
    }

    pub fn snapshot(&self) -> Result<TaskContextSnapshot, TaskContextError> {
        let slots = self
            .typed_slots
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;
        let pads = self
            .scratchpads
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;
        let diagnostics = self
            .diagnostics
            .read()
            .map_err(|_| TaskContextError::Poisoned)?;

        let typed_slots = slots
            .values()
            .map(|slot| slot.type_name.to_string())
            .collect();
        Ok(TaskContextSnapshot {
            typed_slots,
            scratchpads: pads.clone(),
            diagnostics: diagnostics.clone(),
        })
    }

    pub fn debug_dump(&self) -> Result<Option<String>, TaskContextError> {
        if std::env::var("CODEX_DEBUG_SUBAGENTS").is_err() {
            return Ok(None);
        }
        let snapshot = self.snapshot()?;
        serde_json::to_string_pretty(&snapshot)
            .map(Some)
            .map_err(TaskContextError::Serialization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn stores_and_recovers_typed_slots() {
        let ctx = TaskContext::new();
        ctx.insert_typed::<String>("hello".to_string()).unwrap();
        let value = ctx.get_typed::<String>().unwrap();
        assert_eq!(value.as_deref(), Some("hello"));
        let taken = ctx.take_typed::<String>().unwrap();
        assert_eq!(taken.as_deref(), Some("hello"));
        assert!(ctx.get_typed::<String>().unwrap().is_none());
    }

    #[test]
    fn manages_scratchpads() {
        let ctx = TaskContext::new();
        ctx.set_scratchpad("plan", serde_json::json!({"steps": 3}))
            .unwrap();
        let stored = ctx.get_scratchpad("plan").unwrap();
        assert_eq!(stored, Some(serde_json::json!({"steps": 3})));
        let removed = ctx.remove_scratchpad("plan").unwrap();
        assert_eq!(removed, Some(serde_json::json!({"steps": 3})));
    }

    #[test]
    fn snapshot_contains_metadata() {
        let ctx = TaskContext::new();
        ctx.insert_typed::<String>("data".to_string()).unwrap();
        ctx.set_scratchpad("notes", serde_json::json!("hi"))
            .unwrap();
        ctx.push_diagnostic(DiagnosticLevel::Info, "ready").unwrap();
        let snapshot = ctx.snapshot().unwrap();
        assert_eq!(
            snapshot.typed_slots,
            vec![type_name::<String>().to_string()]
        );
        assert_eq!(
            snapshot.scratchpads.get("notes"),
            Some(&serde_json::json!("hi"))
        );
        assert_eq!(snapshot.diagnostics.len(), 1);
    }
}
