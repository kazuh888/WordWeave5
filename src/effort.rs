//! Server-advertised effort choices. Missing capabilities are never inferred.
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EffortOption {
    #[serde(rename = "reasoningEffort")]
    pub effort: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelEffort {
    pub model: String,
    #[serde(default, rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "supportedReasoningEfforts")]
    pub supported_efforts: Option<Vec<EffortOption>>,
    #[serde(rename = "defaultReasoningEffort")]
    pub default_effort: Option<String>,
}

pub(crate) fn valid_effort(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

impl ModelEffort {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.model.is_empty() || self.model.len() > 256 || self.model.chars().any(char::is_control)
            || self.display_name.len() > 1024
            || self.default_effort.as_deref().is_some_and(|v| !valid_effort(v))
            || self.supported_efforts.as_ref().is_some_and(|choices| choices.len() > 64
                || choices.iter().any(|v| !valid_effort(&v.effort) || v.description.len() > 4096)) {
            return Err("Codexが返したモデル・effort候補が不正または上限超過です。推測で補完しません。".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effort_tokens_remain_compatible_with_settings_validation() {
        for token in ["future-effort".to_string(), "high".into(), "a".repeat(64),
            "a".repeat(65), "high\nSECRET".into(), "未知".into(), "high effort".into()] {
            let mut progress = crate::store::Progress::default();
            progress.settings.codex_effort = token.clone();
            assert_eq!(valid_effort(&token), progress.validate().is_ok(), "{token:?}");
        }
    }

    #[test]
    fn malformed_advertised_effort_is_rejected_before_it_can_be_selected() {
        let mut model: ModelEffort = serde_json::from_value(serde_json::json!({
            "model":"test", "supportedReasoningEfforts":[{"reasoningEffort":"future-effort"}]
        })).unwrap();
        assert!(model.validate().is_ok());
        assert_eq!(model.default_effort, None);
        model.supported_efforts.as_mut().unwrap()[0].effort = "high\nSECRET".into();
        assert!(model.validate().is_err());
        model.supported_efforts = None;
        model.default_effort = Some("high effort".into());
        assert!(model.validate().is_err());
    }
}
