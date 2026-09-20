//! Reviewable material proposals from explicitly selected chat exchanges.
use crate::{chat::Conversation, model::{self, Entry}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Mode { New, Append, Correct }
impl Mode {
    pub fn label(self) -> &'static str {
        match self { Self::New => "新規登録", Self::Append => "追加（例文・言い換え）", Self::Correct => "訂正（既存内容を変更）" }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub entry_id: String,
    pub conversation_id: String,
    pub exchange_indices: Vec<usize>,
    pub at: i64,
    pub mode: Mode,
    #[serde(default)]
    pub snapshots: Vec<Snapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub exchange_index: usize,
    pub exchange: crate::chat::Exchange,
}
impl Source {
    pub fn validate(&self) -> Result<(),String> {
        if self.entry_id.is_empty() || self.entry_id.len()>1000 || self.conversation_id.is_empty() || self.conversation_id.len()>1000
            || self.exchange_indices.len()>crate::chat::MAX_EXCHANGES || self.snapshots.len()>crate::chat::MAX_EXCHANGES {
            return Err("教材の根拠参照が不正です。".into());
        }
        let indices:BTreeSet<_>=self.exchange_indices.iter().copied().collect();
        let mut seen=BTreeSet::new();
        for snapshot in &self.snapshots {
            if !indices.contains(&snapshot.exchange_index) || !seen.insert(snapshot.exchange_index) {return Err("根拠の往復番号が不正です。".into());}
            let mut chat=Conversation::new();chat.exchanges.push(snapshot.exchange.clone());chat.validate()?;
        }
        if !self.snapshots.is_empty() && seen!=indices {return Err("教材根拠の固定版が不足しています。".into());}
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quote { pub exchange_index: usize, pub role: String, pub quote: String }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeReason { pub path: String, pub reason: String, pub quotes: Vec<Quote> }
#[derive(Clone, Serialize, Deserialize)]
pub struct Request {
    pub base: String,
    pub mode: Mode,
    pub baseline: Option<Entry>,
    pub source: Source,
    pub payload: Value,
}
impl Request {
    pub fn new(chat: &Conversation, base: &str, mode: Mode, baseline: Option<Entry>) -> Result<Self, String> {
        if chat.deleted_at.is_some() { return Err("ごみ箱の会話は教材化できません。先に復元してください。".into()); }
        let base = base.trim();
        if base.is_empty() || base.len() > 200 { return Err("対象の基本語・熟語を入力してください（200バイト以内）。".into()); }
        if (mode == Mode::New) != baseline.is_none() { return Err("既存教材の反映先を選択してください。".into()); }
        if baseline.as_ref().is_some_and(|e| model::normalize(&e.base) != model::normalize(base)) {
            return Err("対象語と選択した教材の基本語が一致しません。".into());
        }
        let base = baseline.as_ref().map(|e| e.base.as_str()).unwrap_or(base);
        let indices: Vec<_> = chat.exchanges.iter().enumerate().filter(|(_, e)| e.for_material).map(|(i, _)| i).collect();
        if indices.is_empty() { return Err("教材に反映するやり取りを選択してください。".into()); }
        let snapshots: Vec<_> = indices.iter().map(|&i| Snapshot { exchange_index: i, exchange: chat.exchanges[i].clone() }).collect();
        let exchanges: Vec<_> = indices.iter().map(|&i| json!({"exchange_index":i,"user":chat.exchanges[i].question,"assistant":chat.exchanges[i].answer,
            "attachments":chat.exchanges[i].attachments})).collect();
        let payload = json!({"base":base,"mode":mode.label(),"existing_entry":baseline,"selected_exchanges":exchanges});
        if serde_json::to_vec(&payload).map_err(|e| e.to_string())?.len() > 128_000 {
            return Err("選択した会話と既存教材が大きすぎます。やり取りの選択を減らしてください。".into());
        }
        let id = baseline.as_ref().map(|e| e.id.clone()).unwrap_or_else(|| format!("chat_{}_{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        Ok(Self { base:base.into(), mode, baseline,
            source:Source { entry_id:id, conversation_id:chat.id.clone(), exchange_indices:indices, snapshots, at:chrono::Utc::now().timestamp(), mode }, payload })
    }
    pub fn build(&self, mut candidate: Entry) -> Result<Draft, String> {
        if model::normalize(&candidate.base) != model::normalize(&self.base) {
            return Err("Codexが別の基本語を返したため教材案を採用しませんでした。".into());
        }
        candidate.id = self.source.entry_id.clone();
        candidate.base = self.base.clone();
        let mut notices = Vec::new();
        if self.mode == Mode::Append {
            let mut merged = self.baseline.clone().ok_or("既存教材がありません。")?;
            if merged.fingerprint() != candidate.fingerprint() {
                notices.push("基本の説明・問題・正解の変更案は追加モードでは採用しない。訂正が必要な場合は訂正モードで作り直してください。".into());
            }
            for r in candidate.replacements {
                if let Some(old) = merged.replacements.iter().find(|x| model::normalize(&x.phrase) == model::normalize(&r.phrase)) {
                    if old.meaning != r.meaning || old.conditions != r.conditions {
                        notices.push(format!("言い換え「{}」の説明変更は追加モードでは反映しない。必要なら訂正モードを使用する。", r.phrase));
                    }
                } else { merged.replacements.push(r); }
            }
            for x in candidate.examples {
                if model::normalize(&x.english) == model::normalize(&merged.completed()) {
                    notices.push("空欄問題の完成英文と同じ例文は追加しない。既存問題の訳・説明の変更は訂正モードを使用する。".into());
                    continue;
                }
                if let Some(old) = merged.examples.iter().find(|e| model::normalize(&e.english) == model::normalize(&x.english)) {
                    if old.japanese != x.japanese || old.note != x.note {
                        notices.push(format!("例文「{}」の訳・解説変更は追加モードでは反映しない。必要なら訂正モードを使用する。", x.english));
                    }
                } else { merged.examples.push(x); }
            }
            candidate = merged;
        } else {
            let mut seen = BTreeSet::new();
            let before = candidate.replacements.len();
            candidate.replacements.retain(|r| seen.insert(model::normalize(&r.phrase)));
            if before != candidate.replacements.len() { notices.push("生成案内の重複した言い換えを除いた。残した説明を確認してください。".into()); }
            seen.clear();
            let before = candidate.examples.len();
            candidate.examples.retain(|e| seen.insert(model::normalize(&e.english)));
            if before != candidate.examples.len() { notices.push("生成案内の重複した例文を除いた。残した訳と説明を確認してください。".into()); }
        }
        candidate = canonical(candidate)?;
        let draft = Draft { mode:self.mode, baseline:self.baseline.clone(), candidate, source:self.source.clone(), notices, reasons:vec![], generated:None };
        draft.check_mode()?;
        Ok(draft)
    }
    pub fn build_response(&self, text: &str) -> Result<Draft, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Response { entry: Entry, reasons: Vec<ChangeReason> }
        let response: Response = serde_json::from_str(text).map_err(|e| format!("教材案の形式が不正です: {e}"))?;
        if response.reasons.len() > 100 { return Err("変更理由が多すぎます。".into()); }
        let raw = serde_json::to_value(&response.entry).map_err(|e| e.to_string())?;
        let mut draft = self.build(response.entry.clone())?;
        let merged = serde_json::to_value(&draft.candidate).map_err(|e| e.to_string())?;
        let old = self.baseline.as_ref().map(|v| serde_json::to_value(v).unwrap());
        for mut reason in response.reasons {
            if !allowed_path(&reason.path) || raw.pointer(&reason.path).is_none()
                || reason.reason.trim().is_empty() || reason.reason.chars().count() > 2000
                || reason.quotes.is_empty() || reason.quotes.len() > 10 {
                return Err("変更理由の項目・説明・引用が不正です。".into());
            }
            for quote in &reason.quotes {
                let source = self.source.snapshots.iter().find(|s| s.exchange_index == quote.exchange_index).ok_or("選択していない往復が根拠に指定されました。")?;
                let text = match quote.role.as_str() { "user" => &source.exchange.question, "assistant" => &source.exchange.answer, _ => return Err("引用の話者が不正です。".into()) };
                if quote.quote.trim().is_empty() || quote.quote.chars().count() > 2000 || !text.contains(&quote.quote) { return Err("元の発言に存在しない引用です。".into()); }
            }
            let source_value = raw.pointer(&reason.path).unwrap();
            // Append/dedup can move an example. Remap by its actual text, never by a guessed index.
            let parts: Vec<_> = reason.path.split('/').collect();
            if parts.len() == 4 && ["examples", "replacements"].contains(&parts[1]) {
                let key = if parts[1] == "examples" { "english" } else { "phrase" };
                let index: usize = parts[2].parse().map_err(|_| "変更理由の番号が不正です。")?;
                let original = &raw[parts[1]][index][key];
                let Some(index) = merged[parts[1]].as_array().and_then(|a| a.iter().position(|v| v[key] == *original)) else { continue; };
                reason.path = format!("/{}/{index}/{}", parts[1], parts[3]);
            }
            if merged.pointer(&reason.path) != Some(source_value) { continue; }
            if old.as_ref().and_then(|o| o.pointer(&reason.path)) == merged.pointer(&reason.path) { continue; }
            draft.reasons.push(reason);
        }
        draft.generated = Some(draft.candidate.clone());
        Ok(draft)
    }
}

fn allowed_path(path: &str) -> bool {
    let p: Vec<_> = path.split('/').collect();
    if p.len() == 2 { return ["base","meaning","level","business","elevated","register","usage","context","example","translation","answers","question","explanation","tag"].contains(&p[1]); }
    p.len() == 4 && p[2].parse::<usize>().is_ok() && match p[1] {
        "examples" => ["english","japanese","note"].contains(&p[3]),
        "replacements" => ["phrase","meaning","conditions"].contains(&p[3]), _ => false,
    }
}
pub fn response_schema(entry: Value) -> Value {
    json!({"type":"object","properties":{"entry":entry,"reasons":{"type":"array","items":{
        "type":"object","properties":{"path":{"type":"string"},"reason":{"type":"string"},"quotes":{"type":"array","items":{
            "type":"object","properties":{"exchange_index":{"type":"integer"},"role":{"type":"string","enum":["user","assistant"]},"quote":{"type":"string"}},
            "required":["exchange_index","role","quote"],"additionalProperties":false}}},"required":["path","reason","quotes"],"additionalProperties":false}}},"required":["entry","reasons"],"additionalProperties":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat::Conversation, execution::Execution, model::{Example, Replacement}, store::Progress, scheduler::Memory, model::Skill};
    fn old_entry() -> Entry {
        let mut e = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
        e.examples = vec![Example { english:"This is a test.".into(), japanese:"これは例です。".into(), note:"最初の説明".into() }];
        e.replacements = vec![Replacement { phrase:"test phrase".into(), meaning:"意味".into(), conditions:"条件".into() }];
        e
    }
    fn chat() -> Conversation {
        let mut c = Conversation::new();
        c.complete("選択する質問".into(), "選択する回答".into(), Execution::default()).unwrap();
        c.exchanges[0].for_material = true;
        c.complete("送らない質問".into(), "送らない回答".into(), Execution::default()).unwrap();
        c
    }
    fn another_example() -> Example {
        Example { english:"Here is another example.".into(), japanese:"別の例です。".into(), note:"追加の説明".into() }
    }
    #[test]
    fn only_selected_exchanges_are_used_and_missing_selection_is_rejected() {
        let mut c = chat();
        let r = Request::new(&c, "word", Mode::New, None).unwrap();
        assert_eq!(r.source.exchange_indices, vec![0]);
        assert!(!r.payload.to_string().contains("送らない"));
        c.exchanges[0].for_material = false;
        assert!(Request::new(&c,"word",Mode::New,None).is_err());
    }
    #[test]
    fn append_deduplicates_and_does_not_overwrite_existing_explanations_or_grades() {
        let old = old_entry();
        let r = Request::new(&chat(),&old.base,Mode::Append,Some(old.clone())).unwrap();
        let mut proposed = old.clone();
        proposed.usage = "この変更は追加では反映しない".into();
        proposed.examples[0].note = "誤って上書きされてはならない説明".into();
        proposed.examples.push(another_example());
        proposed.examples.push(another_example());
        let draft = r.build(proposed).unwrap();
        assert_eq!(draft.candidate.examples.len(), 2);
        assert_eq!(draft.candidate.examples[0].note, old.examples[0].note);
        assert_eq!(draft.candidate.usage, old.usage);
        assert!(!draft.notices.is_empty());
        assert!(!draft.resets_learning());
        let mut p = Progress::default();
        p.reconcile_deck(&[old.clone()]);
        let key = Skill::Recall.key(&old.id);
        p.memories.insert(key.clone(),Memory::default());
        let accepted = draft.ready(&[old],false).unwrap();
        assert_eq!(p.reconcile_deck(&[accepted]),0);
        assert!(p.memories.contains_key(&key));
    }
    #[test]
    fn correction_resets_changed_core_and_append_rejects_manual_core_edits() {
        let old = old_entry();
        let r = Request::new(&chat(),&old.base,Mode::Correct,Some(old.clone())).unwrap();
        let mut proposed = old.clone();
        proposed.usage.push_str(" 訂正した説明");
        let draft = r.build(proposed).unwrap();
        assert!(draft.resets_learning());
        let mut p = Progress::default();
        p.reconcile_deck(&[old.clone()]);
        let key = Skill::Recall.key(&old.id);
        p.memories.insert(key.clone(),Memory::default());
        assert_eq!(p.reconcile_deck(&[draft.ready(&[old.clone()],false).unwrap()]),1);
        assert!(!p.memories.contains_key(&key));
        let r = Request::new(&chat(),&old.base,Mode::Append,Some(old.clone())).unwrap();
        let mut draft = r.build(old.clone()).unwrap();
        draft.candidate.usage.push_str(" 禁止された変更");
        assert!(draft.ready(&[old],false).is_err());
    }
    #[test]
    fn stale_supplemental_changes_block_commit_even_if_fingerprint_is_unchanged() {
        let old = old_entry();
        let r = Request::new(&chat(),&old.base,Mode::Append,Some(old.clone())).unwrap();
        let mut proposed = old.clone();
        proposed.examples.push(another_example());
        let draft = r.build(proposed).unwrap();
        let mut changed = old.clone();
        changed.examples[0].note = "別操作で更新済み".into();
        assert_eq!(old.fingerprint(),changed.fingerprint());
        assert!(draft.ready(&[changed],false).unwrap_err().contains("変更されました"));
    }
    #[test]
    fn new_entry_requires_explicit_same_base_choice_and_cannot_be_committed_twice() {
        let old = old_entry();
        let r = Request::new(&chat(),&old.base,Mode::New,None).unwrap();
        let mut proposed = old.clone();
        proposed.examples.push(another_example());
        let draft = r.build(proposed).unwrap();
        assert_ne!(draft.candidate.id,old.id);
        assert!(draft.ready(&[old.clone()],false).is_err());
        let accepted = draft.ready(&[old],true).unwrap();
        assert!(draft.ready(&[accepted],true).unwrap_err().contains("登録済み"));
    }
    #[test]
    fn incomplete_review_draft_and_source_survive_save_without_committing_to_deck() {
        let old = old_entry();
        let request = Request::new(&chat(),&old.base,Mode::New,None).unwrap();
        let mut draft = request.build(old).unwrap();
        draft.candidate.meaning.clear(); // User has not finished editing.
        let mut p = Progress::default();
        p.material_draft = Some(draft.clone());
        p.material_sources.push(draft.source.clone());
        p.validate().unwrap();
        let restored: Progress = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(restored.material_sources[0].exchange_indices,vec![0]);
        assert!(restored.material_draft.unwrap().ready(&[],false).is_err());
    }
    #[test]
    fn snapshots_are_fixed_and_legacy_sources_remain_readable() {
        let mut c = chat();
        let r = Request::new(&c, "word", Mode::New, None).unwrap();
        c.exchanges[0].answer = "後から変更".into();
        assert_eq!(r.source.snapshots[0].exchange.answer, "選択する回答");
        assert_eq!(r.source.snapshots[0].exchange_index, 0);
        assert_eq!(r.payload["selected_exchanges"][0]["exchange_index"], 0);
        let mut legacy = serde_json::to_value(&r.source).unwrap();
        legacy.as_object_mut().unwrap().remove("snapshots");
        let legacy: Source = serde_json::from_value(legacy).unwrap();
        assert!(legacy.snapshots.is_empty());
    }
    fn response(entry: &Entry, path: &str, index: usize, quote: &str) -> String {
        json!({"entry":entry,"reasons":[{"path":path,"reason":"選択した説明を反映した。", "quotes":[{"exchange_index":index,"role":"assistant","quote":quote}]}]}).to_string()
    }
    #[test]
    fn response_rejects_out_of_selection_fabricated_and_invalid_path_reasons() {
        let old = old_entry();
        let r = Request::new(&chat(), &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old.clone();
        candidate.usage = "訂正した説明".into();
        assert!(r.build_response(&response(&candidate, "/usage", 1, "送らない回答")).is_err());
        assert!(r.build_response(&response(&candidate, "/usage", 0, "存在しない回答")).is_err());
        assert!(r.build_response(&response(&candidate, "/id", 0, "選択する回答")).is_err());
        assert!(r.build_response(&response(&candidate, "/examples/999/note", 0, "選択する回答")).is_err());
        assert!(r.build_response(&response(&candidate, "/usage", 0, "")).is_err());
    }
    #[test]
    fn manual_edit_disables_ai_reasons_without_destroying_original_evidence() {
        let old = old_entry();
        let r = Request::new(&chat(), &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old.clone();
        candidate.usage = "訂正した説明".into();
        let mut draft = r.build_response(&response(&candidate, "/usage", 0, "選択する回答")).unwrap();
        assert_eq!(draft.active_reasons().len(), 1);
        draft.candidate.usage = "利用者による編集".into();
        assert!(draft.active_reasons().is_empty());
        assert_eq!(draft.reasons.len(), 1);
        assert_eq!(draft.generated.as_ref().unwrap().usage, "訂正した説明");
    }
    #[test]
    fn append_reason_is_remapped_to_merged_example_and_dropped_changes_have_no_reason() {
        let old = old_entry();
        let r = Request::new(&chat(), &old.base, Mode::Append, Some(old.clone())).unwrap();
        let mut candidate = old.clone();
        candidate.examples = vec![another_example()];
        candidate.usage = "採用しない変更".into();
        let draft = r.build_response(&response(&candidate, "/examples/0/note", 0, "選択する回答")).unwrap();
        assert_eq!(draft.reasons[0].path, "/examples/1/note");
        let draft = r.build_response(&response(&candidate, "/usage", 0, "選択する回答")).unwrap();
        assert!(draft.reasons.is_empty());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Draft {
    pub mode: Mode,
    pub baseline: Option<Entry>,
    pub candidate: Entry,
    pub source: Source,
    pub notices: Vec<String>,
    #[serde(default)]
    pub reasons: Vec<ChangeReason>,
    #[serde(default)]
    pub generated: Option<Entry>,
}
fn canonical(entry: Entry) -> Result<Entry, String> {
    Ok(model::parse_deck(&model::deck_text(&[entry]))?.remove(0))
}
impl Draft {
    pub fn validate_evidence(&self) -> Result<(),String> {
        self.source.validate()?;
        if self.reasons.len()>100 {return Err("変更理由が多すぎます。".into());}
        let generated=self.generated.as_ref().map(serde_json::to_value).transpose().map_err(|e|e.to_string())?;
        for reason in &self.reasons {
            if !allowed_path(&reason.path) || generated.as_ref().and_then(|v|v.pointer(&reason.path)).is_none()
                || reason.reason.trim().is_empty() || reason.reason.chars().count()>2000 || reason.quotes.is_empty() || reason.quotes.len()>10 {return Err("保存された変更理由が不正です。".into());}
            for q in &reason.quotes {
                let s=self.source.snapshots.iter().find(|s|s.exchange_index==q.exchange_index).ok_or("引用元がありません。")?;
                let text=match q.role.as_str(){"user"=>&s.exchange.question,"assistant"=>&s.exchange.answer,_=>return Err("引用の話者が不正です。".into())};
                if q.quote.trim().is_empty() || q.quote.chars().count()>2000 || !text.contains(&q.quote) {return Err("元の発言と引用が一致しません。".into());}
            }
        }
        Ok(())
    }
    pub fn active_reasons(&self) -> &[ChangeReason] {
        if self.generated.as_ref().is_some_and(|e| e.to_tsv() == self.candidate.to_tsv()) { &self.reasons } else { &[] }
    }
    fn check_mode(&self) -> Result<(), String> {
        if (self.mode == Mode::New) != self.baseline.is_none() {
            return Err("教材案の登録方法と比較元が一致しません。案を作り直してください。".into());
        }
        if self.candidate.id != self.source.entry_id { return Err("教材IDが変更されています。".into()); }
        if let Some(old) = &self.baseline {
            if old.id != self.candidate.id || old.base != self.candidate.base { return Err("反映先の語またはIDが変更されています。".into()); }
            if self.mode == Mode::Append {
                if old.fingerprint() != self.candidate.fingerprint() { return Err("追加モードでは基本の説明・問題・正解を変更できません。".into()); }
                for r in &old.replacements {
                    if !self.candidate.replacements.iter().any(|x| x.phrase==r.phrase && x.meaning==r.meaning && x.conditions==r.conditions) {
                        return Err("既存の言い換えを変更・削除するには訂正モードを使用してください。".into());
                    }
                }
                for e in &old.examples {
                    if !self.candidate.examples.iter().any(|x| x.english==e.english && x.japanese==e.japanese && x.note==e.note) {
                        return Err("既存の例文を変更・削除するには訂正モードを使用してください。".into());
                    }
                }
            }
        } else if self.mode != Mode::New { return Err("既存教材の比較元がありません。".into()); }
        Ok(())
    }
    pub fn ready(&self, deck: &[Entry], allow_same_base: bool) -> Result<Entry, String> {
        self.validate_evidence()?;
        self.check_mode()?;
        if let Some(old) = &self.baseline {
            let current = deck.iter().find(|e| e.id == old.id).ok_or("反映先の教材がなくなりました。案を作り直してください。")?;
            if current.to_tsv() != old.to_tsv() { return Err("教材案の生成後に既存教材が変更されました。案を作り直してください。".into()); }
        } else {
            if deck.iter().any(|e| e.id == self.candidate.id) { return Err("この教材案は既に登録済みです。".into()); }
            if !allow_same_base && deck.iter().any(|e| model::normalize(&e.base) == model::normalize(&self.candidate.base)) {
                return Err("同じ基本語が登録済みです。別用法として新規登録する場合は確認欄を選んでください。".into());
            }
            if self.candidate.examples.len() < 2 { return Err("新規教材には完成例文を2件以上用意してください。".into()); }
        }
        let candidate = canonical(self.candidate.clone())?;
        if self.baseline.as_ref().is_some_and(|e| e.to_tsv() == candidate.to_tsv()) {
            return Err("既存教材と同じ内容のため、登録する変更がありません。".into());
        }
        Ok(candidate)
    }
    pub fn resets_learning(&self) -> bool {
        self.baseline.as_ref().is_some_and(|e| e.fingerprint() != self.candidate.fingerprint())
    }
}
