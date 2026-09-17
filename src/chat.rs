//! Persistent transcripts and explicit, bounded context selection.
use crate::execution::Execution;
use crate::model::Entry;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const CONTEXT_BYTES: usize = 64_000;
pub const MAX_EXCHANGES: usize = 200;
pub const MAX_CHATS: usize = 50;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub original: crate::assets::AssetRef,
    pub image: Option<crate::assets::AssetRef>,
    #[serde(default)]
    pub background: Option<crate::assets::AssetRef>,
    pub source_text: String,
    pub transcript: Option<String>,
}
impl Attachment {
    pub fn validate(&self) -> Result<(), String> {
        use crate::assets::AssetKind;
        self.original.validate()?;
        if let Some(image) = &self.image { image.validate()?; if image.kind != AssetKind::ImagePng { return Err("添付画像の形式が不正です。".into()); } }
        if let Some(image) = &self.background { image.validate()?; if image.kind != AssetKind::ImagePng { return Err("背景画像の形式が不正です。".into()); } }
        if self.source_text.chars().count() > 4000 || self.transcript.as_ref().is_some_and(|s| s.chars().count() > 4000)
            || (self.original.kind == AssetKind::InkJson && self.image.is_none()) {
            return Err("添付の説明・画像が不正です。".into());
        }
        Ok(())
    }
}

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
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub created_at: i64,
    pub memo: String,
    pub draft: String,
    pub exchanges: Vec<Exchange>,
    #[serde(default)]
    pub draft_attachments: Vec<Attachment>,
}
impl Conversation {
    pub fn new() -> Self {
        Self {
            id: format!("{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()),
            title: "新しい会話".into(), pinned: false, created_at: chrono::Utc::now().timestamp(),
            memo: String::new(), draft: String::new(), exchanges: Vec::new(), draft_attachments: Vec::new(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        for attachments in std::iter::once(&self.draft_attachments).chain(self.exchanges.iter().map(|e| &e.attachments)) {
            if attachments.len() > 8 { return Err("一つの発言の添付は8件までです。".into()); }
            for attachment in attachments { attachment.validate()?; }
        }
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
            self.title = automatic_title(&question);
        }
        self.exchanges.push(Exchange { question: question.clone(), answer, at: chrono::Utc::now().timestamp(), execution, pinned: false, for_material: false,
            attachments: self.draft_attachments.clone() });
        self.draft_attachments.clear();
        if self.draft.trim() == question.trim() { self.draft.clear(); }
        Ok(())
    }
    pub fn apply_recognition(&mut self, expected: &str, text: &str) -> Result<(), String> {
        if self.draft != expected { return Err("入力欄が変更されたため、認識結果を自動反映しませんでした。結果を確認して貼り付けてください。".into()); }
        let next = if expected.trim().is_empty() { text.to_owned() } else { format!("{expected}\n{text}") };
        if text.trim().is_empty() || next.chars().count() > 4000 { return Err("認識結果が空、または入力上限を超えています。".into()); }
        self.draft = next;
        Ok(())
    }
}

/// Display order only: never reorder stored conversations or invalidate an active index.
pub fn ordered_indices(chats: &[Conversation]) -> Vec<usize> {
    let mut indices: Vec<_> = (0..chats.len()).collect();
    indices.sort_by_key(|&i| {
        let c = &chats[i];
        (std::cmp::Reverse(c.pinned), std::cmp::Reverse(c.exchanges.last().map_or(c.created_at, |e| e.at)))
    });
    indices
}

fn automatic_title(question: &str) -> String {
    let normalized = question.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= 48 { return normalized; }
    let mut title: String = normalized.chars().take(47).collect();
    // Prefer a whole English word, while still supporting Japanese without spaces.
    if let Some((prefix, _)) = title.rsplit_once(' ') {
        if prefix.chars().count() >= 24 { title = prefix.to_owned(); }
    }
    title.push('…');
    title
}

pub struct Context {
    pub payload: Value,
    pub included: usize,
    pub omitted: usize,
    pub bytes: usize,
    /// Zero-based indices in the saved transcript, in chronological order.
    pub included_indices: Vec<usize>,
    pub omitted_indices: Vec<usize>,
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
        let numbers = |indices: &[usize]| indices.iter().map(|i| (i + 1).to_string()).collect::<Vec<_>>().join(", ");
        text.push_str(&format!("今回の質問：\n{}\n\n送信対象の往復番号：{}\n送信しない過去のやり取り：{}往復（番号：{}）\n保存済みの履歴は削除されません。",
            self.payload["current_question"].as_str().unwrap_or_default(), numbers(&self.included_indices),
            self.omitted, numbers(&self.omitted_indices)));
        if self.payload.get("existing_entries").is_some() {
            text.push_str(&format!("\n\n送信する既存単語の一覧：\n{}\n一覧に含めない単語：{}件\n省略した単語の存在や内容は、この一覧だけから判断できません。",
                serde_json::to_string_pretty(&self.payload["existing_entries"]).unwrap(), self.payload["omitted_catalog"]));
        }
        text
    }
}
pub fn prepare(conversation: &Conversation) -> Result<Context, String> {
    prepare_with_limit(conversation, CONTEXT_BYTES)
}
fn prepare_with_limit(conversation: &Conversation, limit: usize) -> Result<Context, String> {
    prepare_selection(conversation, limit, None)
}

/// A bounded catalog is part of the same byte budget as the conversation.
pub fn prepare_with_catalog(conversation: &Conversation, entries: &[Entry], deleted_entries: &BTreeSet<String>) -> Result<Context, String> {
    conversation.validate()?;
    let mut required = conversation.clone();
    required.exchanges.retain(|e| e.pinned);
    let required_bytes = prepare_with_limit(&required, CONTEXT_BYTES)?.bytes;
    let budget = ((CONTEXT_BYTES - required_bytes) / 4).min(8_000);
    let query = relevance_terms(&conversation.draft);
    let mut candidates: Vec<_> = entries.iter().enumerate().filter(|(_, e)| !deleted_entries.contains(&e.id))
        .map(|(i, e)| {
            let terms = relevance_terms(&format!("{} {}", e.base, e.meaning));
            (i, terms.intersection(&query).count(), e)
        }).collect();
    candidates.sort_by_key(|&(i, score, _)| (std::cmp::Reverse(score), i));
    let total = candidates.len();
    let mut selected = Vec::new();
    let mut bytes = 2; // JSON array brackets.
    for (_, _, e) in candidates {
        let item = json!({"id": e.id, "base": e.base, "meaning": e.meaning});
        let item_bytes = serde_json::to_vec(&item).unwrap().len() + usize::from(!selected.is_empty());
        if bytes + item_bytes <= budget { bytes += item_bytes; selected.push(item); }
    }
    let mut catalog = json!({"existing_entries": selected, "omitted_catalog": total - selected.len()});
    // Reserve field-name overhead too, including when required pins nearly fill the budget.
    loop {
        match prepare_selection(conversation, CONTEXT_BYTES, Some(&catalog)) {
            Ok(context) => return Ok(context),
            Err(error) => {
                let selected = catalog["existing_entries"].as_array_mut().unwrap();
                if selected.pop().is_none() { return Err(error); }
                let omitted = total - selected.len();
                catalog["omitted_catalog"] = json!(omitted);
            }
        }
    }
}

fn prepare_selection(conversation: &Conversation, limit: usize, catalog: Option<&Value>) -> Result<Context, String> {
    conversation.validate()?;
    let question = conversation.draft.trim();
    if question.is_empty() { return Err("英語についての質問を入力してください。".into()); }
    let mut included: Vec<bool> = conversation.exchanges.iter().map(|e| e.pinned).collect();
    let payload_for = |selected: &[bool]| {
        let history: Vec<_> = conversation.exchanges.iter().zip(selected).filter(|(_, yes)| **yes)
            .map(|(e, _)| {
                let mut item = json!({"user": e.question, "assistant": e.answer});
                if !e.attachments.is_empty() { item["attachment_notes_not_original_media"] = json!(e.attachments.iter().map(|a| &a.source_text).collect::<Vec<_>>()); }
                item
            }).collect();
        let mut payload = json!({"conversation_history": history, "learner_memo": conversation.memo,
            "current_question": question, "omitted_exchanges": selected.iter().filter(|b| !**b).count()});
        if !conversation.draft_attachments.is_empty() { payload["current_attachments"] = json!(conversation.draft_attachments); }
        if let Some(catalog) = catalog {
            payload.as_object_mut().unwrap().extend(catalog.as_object().unwrap().clone());
        }
        payload
    };
    let size = |value: &Value| serde_json::to_vec(value).unwrap().len();
    if size(&payload_for(&included)) > limit {
        return Err("質問・引き継ぎメモ・「文脈に必ず含める」の合計が送信上限を超えています。参照指定やメモを減らしてください。".into());
    }
    let required_bytes = size(&payload_for(&included));
    let recent_budget = required_bytes + (limit - required_bytes) / 2;
    let mut recent_start = included.len();
    // Keep recent complete exchanges first; reserve room to recover older relevant turns.
    for index in (0..included.len()).rev() {
        if included[index] { recent_start = index; continue; }
        included[index] = true;
        let bytes = size(&payload_for(&included));
        if bytes > limit || (bytes > recent_budget && recent_start < included.len()) {
            included[index] = false;
            break;
        }
        recent_start = index;
    }
    let query_terms = relevance_terms(question);
    let mut relevant: Vec<_> = conversation.exchanges.iter().enumerate()
        .filter(|(i, _)| !included[*i])
        .map(|(i, e)| {
            let terms = relevance_terms(&format!("{} {}", e.question, e.answer));
            (i, terms.intersection(&query_terms).count())
        }).filter(|(_, score)| *score > 0).collect();
    relevant.sort_by_key(|&(i, score)| (std::cmp::Reverse(score), std::cmp::Reverse(i)));
    for (index, _) in relevant {
        included[index] = true;
        if size(&payload_for(&included)) > limit { included[index] = false; }
    }
    // Use remaining room for the recent suffix; a gap never splits an exchange.
    for index in (0..recent_start).rev() {
        if included[index] { continue; }
        included[index] = true;
        if size(&payload_for(&included)) > limit {
            included[index] = false;
            break;
        }
    }
    let payload = payload_for(&included);
    let count = included.iter().filter(|b| **b).count();
    let included_indices = included.iter().enumerate().filter_map(|(i, yes)| yes.then_some(i)).collect();
    let omitted_indices = included.iter().enumerate().filter_map(|(i, yes)| (!yes).then_some(i)).collect();
    Ok(Context { bytes: size(&payload), payload, included: count, omitted: included.len() - count,
        included_indices, omitted_indices })
}

// Local retrieval is a deterministic lexical heuristic, not semantic understanding.
// English words and Japanese adjacent-character pairs cover mixed learner questions.
fn relevance_terms(text: &str) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let lower = word.to_lowercase();
        if lower.is_ascii() {
            if lower.len() >= 2 && !["the", "and", "for", "that", "this", "what", "how", "can", "you", "is", "it", "to", "of", "in", "an", "are", "with", "please"].contains(&lower.as_str()) {
                terms.insert(lower);
            }
        } else {
            // Retain English embedded in unspaced Japanese and Japanese bigrams.
            for latin in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
                if latin.len() >= 2 { terms.insert(latin.to_owned()); }
            }
            let chars: Vec<_> = lower.chars().collect();
            for pair in chars.windows(2) {
                if pair.iter().all(|c| !c.is_ascii()) { terms.insert(pair.iter().collect()); }
            }
        }
    }
    terms
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_chat_keeps_media_and_recognition_never_overwrites_new_draft() {
        let mut c = Conversation::new();
        c.draft = "丸で囲んだ部分を説明して".into();
        c.draft_attachments.push(Attachment {
            original: crate::assets::AssetRef { id: "a".repeat(64), kind: crate::assets::AssetKind::InkJson, bytes: 100 },
            image: Some(crate::assets::AssetRef { id: "b".repeat(64), kind: crate::assets::AssetKind::ImagePng, bytes: 200 }),
            background: None,
            source_text: "Could you make it?".into(), transcript: None,
        });
        let captured = c.draft.clone();
        c.complete(captured.clone(), "回答".into(), Execution::default()).unwrap();
        assert_eq!(c.exchanges[0].attachments.len(), 1);
        assert!(c.draft_attachments.is_empty());
        c.draft = "新しい入力".into();
        assert!(c.apply_recognition(&captured, "認識結果").is_err());
        assert_eq!(c.draft, "新しい入力");
        c.apply_recognition("新しい入力", "認識結果").unwrap();
        assert_eq!(c.draft, "新しい入力\n認識結果");
        let restored: Conversation = serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        assert_eq!(restored.exchanges[0].attachments[0].source_text, "Could you make it?");
    }
    fn exchange(i: usize) -> Exchange {
        Exchange { question: format!("質問{i}"), answer: "説明".repeat(80), at: 0,
            execution: Execution::default(), pinned: false, for_material: false, attachments: Vec::new() }
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
    #[test]
    fn legacy_chats_load_without_pin_or_creation_time() {
        let c: Conversation = serde_json::from_value(json!({"id":"old", "title":"old title",
            "memo":"memo", "draft":"draft", "exchanges":[]})).unwrap();
        assert!(!c.pinned);
        assert_eq!(c.created_at, 0);
        assert_eq!(c.memo, "memo");
        let mut pinned = c.clone();
        pinned.pinned = true;
        pinned.created_at = 123;
        let restored: Conversation = serde_json::from_value(serde_json::to_value(&pinned).unwrap()).unwrap();
        assert!(restored.pinned);
        assert_eq!(restored.created_at, 123);
    }
    #[test]
    fn titles_normalize_whitespace_and_keep_custom_titles() {
        let mut c = Conversation::new();
        c.complete("  Explain\n  affect versus effect  ".into(), "answer".into(), Execution::default()).unwrap();
        assert_eq!(c.title, "Explain affect versus effect");
        c.complete("next topic".into(), "answer".into(), Execution::default()).unwrap();
        assert_eq!(c.title, "Explain affect versus effect");
        assert!(automatic_title(&"英".repeat(60)).chars().count() <= 48);
    }
    #[test]
    fn display_order_is_pins_then_latest_output_with_stable_ties() {
        let mut chats: Vec<_> = (0..5).map(|_| Conversation::new()).collect();
        for c in &mut chats { c.created_at = 10; }
        chats[0].exchanges = vec![exchange(0)];
        chats[0].exchanges[0].at = 30;
        chats[1].pinned = true;
        chats[2].exchanges = vec![exchange(2)];
        chats[2].exchanges[0].at = 40;
        chats[3].pinned = true;
        chats[3].created_at = 20;
        assert_eq!(ordered_indices(&chats), vec![3, 1, 2, 0, 4]);
        chats[3].created_at = 10;
        assert_eq!(ordered_indices(&chats), vec![1, 3, 2, 0, 4]);
    }
    #[test]
    fn older_relevant_turn_is_recalled_alongside_recent_turns() {
        let mut c = Conversation::new();
        c.draft = "Explain SERENDIPITY again".into();
        c.exchanges = (0..8).map(exchange).collect();
        c.exchanges[0].question = "serendipity".into();
        let before = serde_json::to_value(&c).unwrap();
        let context = prepare_with_limit(&c, 1800).unwrap();
        assert!(context.included_indices.contains(&0));
        assert!(context.included_indices.contains(&7));
        assert!(context.bytes <= 1800);
        assert_eq!(context.bytes, serde_json::to_vec(&context.payload).unwrap().len());
        assert_eq!(context.included_indices.len() + context.omitted_indices.len(), 8);
        assert!(context.included_indices.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(serde_json::to_value(&c).unwrap(), before);
        assert!(context.preview().contains("送信対象の往復番号"));
    }
    #[test]
    fn budget_covers_escaped_unicode_and_does_not_split_pairs() {
        let mut c = Conversation::new();
        c.draft = "意味を説明".into();
        c.exchanges = (0..8).map(exchange).collect();
        for e in &mut c.exchanges { e.answer = "日本語\n\"\\".repeat(40); }
        for limit in [150, 800, 1600, 64000] {
            let context = prepare_with_limit(&c, limit).unwrap();
            assert!(context.bytes <= limit);
            for (pair, &index) in context.payload["conversation_history"].as_array().unwrap().iter().zip(&context.included_indices) {
                assert_eq!(pair["user"], c.exchanges[index].question);
                assert_eq!(pair["assistant"], c.exchanges[index].answer);
            }
        }
        assert!(relevance_terms("serendipityの意味").contains("serendipity"));
    }
    #[test]
    fn catalog_is_bounded_relevant_and_excludes_deleted_entries() {
        let mut c = Conversation::new();
        c.draft = "serendipityについて追加したい".into();
        c.exchanges = (0..8).map(exchange).collect();
        let template = crate::model::parse_deck(crate::model::BUILTIN_DECK).unwrap().remove(0);
        let mut entries: Vec<_> = (0..1000).map(|i| {
            let mut e = template.clone();
            e.id = format!("entry-{i}");
            e.base = format!("unrelated{i}");
            e.meaning = "意味".repeat(100);
            e
        }).collect();
        entries[998].base = "serendipity".into();
        entries[999].base = "serendipity".into();
        let deleted = BTreeSet::from(["entry-999".to_owned()]);
        let context = prepare_with_catalog(&c, &entries, &deleted).unwrap();
        let catalog = context.payload["existing_entries"].as_array().unwrap();
        assert_eq!(catalog[0]["id"], "entry-998");
        assert!(catalog.iter().all(|e| e["id"] != "entry-999"));
        assert_eq!(catalog.len() + context.payload["omitted_catalog"].as_u64().unwrap() as usize, 999);
        assert!(context.bytes <= CONTEXT_BYTES);
        assert_eq!(context.bytes, serde_json::to_vec(&context.payload).unwrap().len());
        assert!(context.preview().contains("entry-998"));
    }
}
