//! Validated suggestions from chat. Parsing never changes the vocabulary store.
use crate::execution::Execution;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Organize,
    New,
    Append,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub operation: Operation,
    pub base: String,
    pub entry_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ChatReply {
    pub answer: String,
    pub execution: Execution,
    pub title: String,
    pub action: Option<Action>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireReply {
    answer: String,
    title: String,
    action: WireAction,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireAction {
    operation: WireOperation,
    base: String,
    #[serde(deserialize_with = "required_entry_id")]
    entry_id: Option<String>,
}

fn required_entry_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum WireOperation {
    None,
    Organize,
    New,
    Append,
    Delete,
}

pub fn schema() -> Value {
    json!({
        "type":"object", "additionalProperties":false,
        "properties": {
            "answer":{"type":"string"},
            "title":{"type":"string"},
            "action":{
                "type":"object", "additionalProperties":false,
                "properties":{
                    "operation":{"type":"string","enum":["none","organize","new","append","delete"]},
                    "base":{"type":"string"},
                    "entry_id":{"type":["string","null"]}
                },
                "required":["operation","base","entry_id"]
            }
        },
        "required":["answer","title","action"]
    })
}

pub fn parse(text: &str, execution: Execution) -> Result<ChatReply, String> {
    let wire: WireReply = serde_json::from_str(text)
        .map_err(|e| format!("チャット応答の形式が不正です。再送してください: {e}"))?;
    let answer = wire.answer.trim().to_owned();
    if answer.is_empty() {
        return Err("チャット回答が空です。再送してください。".into());
    }
    let title = wire.title.trim().to_owned();
    if title.is_empty() || title.chars().count() > 100 || title.chars().any(char::is_control) {
        return Err("チャットのタイトルが不正です（1〜100文字の1行が必要）。".into());
    }
    let base = wire.action.base.trim().to_owned();
    let entry_id = wire.action.entry_id;
    if entry_id.as_ref().is_some_and(|id| {
        id.trim().is_empty()
            || id != id.trim()
            || id.len() > 1000
            || id.chars().any(char::is_control)
    }) {
        return Err("対象教材IDが不正です。".into());
    }
    let action = match wire.action.operation {
        WireOperation::None => {
            if !base.is_empty() || entry_id.is_some() {
                return Err("操作なしの回答に対象教材が指定されています。".into());
            }
            None
        }
        operation => {
            if base.is_empty() || base.chars().count() > 200 || base.chars().any(char::is_control) {
                return Err("操作対象の単語・表現が不正です。".into());
            }
            let operation = match operation {
                WireOperation::Organize => Operation::Organize,
                WireOperation::New => {
                    if entry_id.is_some() {
                        return Err("新規登録に既存教材IDは指定できません。".into());
                    }
                    Operation::New
                }
                WireOperation::Append => Operation::Append,
                WireOperation::Delete => Operation::Delete,
                WireOperation::None => unreachable!(),
            };
            Some(Action {
                operation,
                base,
                entry_id,
            })
        }
    };
    Ok(ChatReply {
        answer,
        execution,
        title,
        action,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(operation: &str, base: &str, entry_id: Option<&str>) -> String {
        json!({"answer":"確認してから登録してください。","title":"requestの使い方",
            "action":{"operation":operation,"base":base,"entry_id":entry_id}})
        .to_string()
    }

    #[test]
    fn ordinary_answer_has_no_action_and_preserves_server_execution() {
        let execution = Execution {
            model: Some("reported-model".into()),
            effort: None,
            at: 123,
        };
        let parsed = parse(&reply("none", "", None), execution.clone()).unwrap();
        assert_eq!(parsed.action, None);
        assert_eq!(parsed.execution, execution);
    }

    #[test]
    fn parses_each_proposed_operation_without_applying_it() {
        for (name, expected) in [
            ("organize", Operation::Organize),
            ("new", Operation::New),
            ("append", Operation::Append),
            ("delete", Operation::Delete),
        ] {
            let action = parse(&reply(name, "request", None), Execution::default())
                .unwrap()
                .action
                .unwrap();
            assert_eq!(action.operation, expected);
            assert_eq!(action.base, "request");
        }
        let action = parse(
            &reply("delete", "request", Some("entry-42")),
            Execution::default(),
        )
        .unwrap()
        .action
        .unwrap();
        assert_eq!(action.entry_id.as_deref(), Some("entry-42"));
    }

    #[test]
    fn rejects_malformed_unknown_and_inconsistent_actions() {
        for text in [
            "not json".into(),
            reply("update", "request", None),
            reply("delete", " ", None),
            reply("none", "request", None),
            reply("none", "", Some("id")),
            reply("new", "request", Some("id")),
            reply("append", "request", Some(" ")),
            reply("append", "request\nother", None),
        ] {
            assert!(parse(&text, Execution::default()).is_err(), "{text}");
        }
    }

    #[test]
    fn rejects_empty_answer_invalid_title_and_unknown_fields() {
        for (field, value) in [
            ("answer", json!(" ")),
            ("title", json!("")),
            ("title", json!("あ".repeat(101))),
            ("title", json!("one\ntwo")),
            ("extra", json!(true)),
        ] {
            let mut wire: Value = serde_json::from_str(&reply("none", "", None)).unwrap();
            wire[field] = value;
            assert!(parse(&wire.to_string(), Execution::default()).is_err());
        }
        let mut wire: Value = serde_json::from_str(&reply("none", "", None)).unwrap();
        wire["action"].as_object_mut().unwrap().remove("entry_id");
        assert!(parse(&wire.to_string(), Execution::default()).is_err());
    }
}
