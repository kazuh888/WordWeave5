//! Only settings reported by Codex, never guessed from configured defaults.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Execution {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub at: i64,
}
impl Execution {
    pub fn from_response(response: &Value) -> Self {
        let field = |name| response.get(name).and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty()).map(str::to_owned);
        Self { model: field("model"), effort: field("reasoningEffort"), at: chrono::Utc::now().timestamp() }
    }
    pub fn label(&self) -> String {
        format!("モデル：{} / effort：{}",
            self.model.as_deref().unwrap_or("未取得"),
            self.effort.as_deref().unwrap_or("未取得（Codexにお任せ）"))
    }
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub active: bool,
    pub execution: Option<Execution>,
    id: u64,
}
static STATE: OnceLock<Mutex<Snapshot>> = OnceLock::new();
pub fn snapshot() -> Snapshot { STATE.get_or_init(Default::default).lock().unwrap().clone() }
pub struct Run(u64);
impl Run {
    pub fn begin() -> Self {
        let mut state = STATE.get_or_init(Default::default).lock().unwrap();
        state.id = state.id.wrapping_add(1);
        state.active = true;
        state.execution = None;
        Self(state.id)
    }
    pub fn observed(&self, execution: &Execution) {
        let mut state = STATE.get_or_init(Default::default).lock().unwrap();
        if state.id == self.0 { state.execution = Some(execution.clone()); }
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        let mut state = STATE.get_or_init(Default::default).lock().unwrap();
        if state.id == self.0 { state.active = false; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn missing_or_null_effort_is_not_inferred_from_model() {
        for response in [json!({"model":"test"}), json!({"model":"test","reasoningEffort":null})] {
            let settings = Execution::from_response(&response);
            assert_eq!(settings.model.as_deref(), Some("test"));
            assert_eq!(settings.effort, None);
            assert!(settings.label().contains("未取得"));
        }
    }
    #[test]
    fn reported_settings_are_preserved() {
        let settings = Execution::from_response(&json!({"model":"returned-model","reasoningEffort":"high"}));
        assert_eq!(settings.model.as_deref(), Some("returned-model"));
        assert_eq!(settings.effort.as_deref(), Some("high"));
    }
}
