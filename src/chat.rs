//! Persistent transcripts and explicit, bounded context selection.
use crate::execution::Execution;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const CONTEXT_BYTES: usize = 64_000;
pub const MAX_EXCHANGES: usize = 200;
pub const MAX_CHATS: usize = 50;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exchange {
    pub question: String,
    pub answer: String,
    pub at: i64,
    pub execution: Execution,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub for_material: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub memo: String,
    pub draft: String,
    pub exchanges: Vec<Exchange>,
}
impl Conversation {
    pub fn new() -> Self {
        Self {
            id: format!("{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()),
            title: "新しい会話".into(), memo: String::new(), draft: String::new(), exchanges: Vec::new(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.title.chars().count() > 100
            || self.memo.chars().count() > 2000 || self.draft.chars().count() > 4000
            || self.exchanges.len() > MAX_EXCHANGES
            || self.exchanges.iter().any(|e| e.question.trim().is_empty()
                || e.question.chars().count() > 4000 || e.answer.trim().is_empty()
                || e.answer.chars().count() > 16000) {
            return Err("チャットのデータが不正、または保存上限を超えています。".into());
        }
        Ok(())
    }
    // Only a completed answer is committed. Failed attempts keep the draft and
    // are never included in the next context as successful assistant replies.
    pub fn complete(&mut self, question: String, answer: String, execution: Execution) -> Result<(), String> {
        if self.exchanges.len() >= MAX_EXCHANGES { return Err("この会話は200往復に達しました。新しい会話を作成してください。".into()); }
        if question.trim().is_empty() || question.chars().count() > 4000
            || answer.trim().is_empty() || answer.chars().count() > 16000 {
            return Err("回答が空、または保存上限を超えたため会話に追加しませんでした。質問の下書きは保持しています。".into());
        }
        if self.exchanges.is_empty() && self.title == "新しい会話" {
            self.title = question.chars().take(40).collect();
        }
        self.exchanges.push(Exchange { question: question.clone(), answer, at: chrono::Utc::now().timestamp(), execution, pinned: false, for_material: false });
        if self.draft.trim() == question.trim() { self.draft.clear(); }
        Ok(())
    }
}

pub struct Context {
    pub payload: Value,
    pub included: usize,
    pub omitted: usize,
    pub bytes: usize,
}
impl Context {
    pub fn preview(&self) -> String {
        let mut text = format!("引き継ぎメモ：\n{}\n\n", self.payload["learner_memo"].as_str().unwrap_or_default());
        if let Some(history) = self.payload["conversation_history"].as_array() {
            for pair in history {
                text.push_str(&format!("あなた：\n{}\n\nCodex：\n{}\n\n",
                    pair["user"].as_str().unwrap_or_default(), pair["assistant"].as_str().unwrap_or_default()));
            }
        }
        text.push_str(&format!("今回の質問：\n{}\n\n送信しない過去のやり取り：{}往復",
            self.payload["current_question"].as_str().unwrap_or_default(), self.omitted));
        text
    }
}
pub fn prepare(conversation: &Conversation) -> Result<Context, String> {
    prepare_with_limit(conversation, CONTEXT_BYTES)
}
fn prepare_with_limit(conversation: &Conversation, limit: usize) -> Result<Context, String> {
    conversation.validate()?;
    let question = conversation.draft.trim();
    if question.is_empty() { return Err("英語についての質問を入力してください。".into()); }
    let mut included: Vec<bool> = conversation.exchanges.iter().map(|e| e.pinned).collect();
    let payload_for = |selected: &[bool]| {
        let history: Vec<_> = conversation.exchanges.iter().zip(selected).filter(|(_, yes)| **yes)
            .map(|(e, _)| json!({"user": e.question, "assistant": e.answer})).collect();
        json!({"conversation_history": history, "learner_memo": conversation.memo,
            "current_question": question, "omitted_exchanges": selected.iter().filter(|b| !**b).count()})
    };
    let size = |value: &Value| serde_json::to_vec(value).unwrap().len();
    if size(&payload_for(&included)) > limit {
        return Err("質問・引き継ぎメモ・「次回も参照」の合計が送信上限を超えています。参照指定やメモを減らしてください。".into());
    }
    for index in (0..included.len()).rev() {
        if included[index] { continue; }
        included[index] = true;
        if size(&payload_for(&included)) > limit {
            included[index] = false;
            break; // Keep a contiguous recent suffix, plus explicit pins.
        }
    }
    let payload = payload_for(&included);
    let count = included.iter().filter(|b| **b).count();
    Ok(Context { bytes: size(&payload), payload, included: count, omitted: included.len() - count })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn exchange(i: usize) -> Exchange {
        Exchange { question: format!("質問{i}"), answer: "説明".repeat(80), at: 0,
            execution: Execution::default(), pinned: false, for_material: false }
    }
    #[test]
    fn context_preserves_order_and_pins_within_budget() {
        let mut c = Conversation::new();
        c.draft = "追加質問".into();
        c.memo = "社外メール".into();
        c.exchanges = (0..8).map(exchange).collect();
        c.exchanges[0].pinned = true;
        let context = prepare_with_limit(&c, 1800).unwrap();
        let history = context.payload["conversation_history"].as_array().unwrap();
        assert_eq!(history.first().unwrap()["user"], "質問0");
        assert_eq!(history.last().unwrap()["user"], "質問7");
        assert!(context.bytes <= 1800 && context.omitted > 0);
        assert_eq!(context.payload["learner_memo"], "社外メール");
        assert_eq!(c.exchanges.len(), 8); // Sending less never deletes history.
    }
    #[test]
    fn oversized_pins_are_not_silently_dropped() {
        let mut c = Conversation::new();
        c.draft = "質問".into();
        c.exchanges = vec![exchange(0)];
        c.exchanges[0].pinned = true;
        assert!(prepare_with_limit(&c, 200).is_err());
    }
    #[test]
    fn failed_reply_keeps_draft_and_success_survives_reload() {
        let mut c = Conversation::new();
        c.draft = "前置詞について".into();
        assert!(c.complete(c.draft.clone(), " ".into(), Execution::default()).is_err());
        assert!(c.exchanges.is_empty());
        let question = c.draft.clone();
        c.complete(question, "前置詞の説明".into(), Execution::default()).unwrap();
        let mut restored: Conversation = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        restored.draft = "別の例は？".into();
        let payload = prepare(&restored).unwrap().payload;
        assert_eq!(payload["conversation_history"][0]["user"], "前置詞について");
        assert_eq!(payload["current_question"], "別の例は？");
    }
    #[test]
    fn new_conversation_has_no_other_chats_context() {
        let mut c = Conversation::new();
        c.draft = "新しい質問".into();
        assert_eq!(prepare(&c).unwrap().included, 0);
    }
}
