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
    pub fn ui_label(self) -> &'static str {
        match self { Self::New => "新規登録", Self::Append => "追加のみ", Self::Correct => "内容を見直す（追加・変更・削除）" }
    }
    pub fn ui_description(self) -> &'static str {
        match self {
            Self::New => "新しい教材として登録する。",
            Self::Append => "既存の内容と復習状態を保ち、例文や言い換えを追加する。",
            Self::Correct => "追加もできる。基本の説明・問題・正解等を変更した場合は再学習になる。",
        }
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
fn quote_preview(text: &str) -> String {
    let mut chars = text.chars();
    let preview: String = chars.by_ref().take(240).collect();
    let escaped = Value::String(preview).to_string();
    if chars.next().is_some() { format!("{escaped}…（省略）") } else { escaped }
}
#[derive(Clone, Debug)]
pub enum MaterialFailure { Other(String), Evidence(MaterialDiagnostic) }
impl MaterialFailure {
    pub fn legacy_message(&self) -> String {
        match self { Self::Other(text) => text.clone(), Self::Evidence(detail) => detail.legacy_text.clone() }
    }
    pub fn diagnostic(&self) -> Option<&MaterialDiagnostic> {
        match self { Self::Evidence(detail) => Some(detail), Self::Other(_) => None }
    }
}
impl From<String> for MaterialFailure {
    fn from(text: String) -> Self { Self::Other(text) }
}
impl From<&str> for MaterialFailure {
    fn from(text: &str) -> Self { Self::Other(text.into()) }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticStage { Generation, SavedDraft }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceCause {
    SourceReference, InvalidPath, MissingItem, EmptyReason, LongReason,
    MissingQuotes, TooManyQuotes, MissingSnapshot, InvalidRole,
    EmptyQuote, LongQuote, QuoteMismatch,
}
#[derive(Clone, Debug)]
pub struct QuoteEvidence {
    pub exchange_index: usize,
    pub role: String,
    pub quote: String,
    pub original: Option<String>,
}
#[derive(Clone, Debug)]
pub struct MaterialDiagnostic {
    pub stage: DiagnosticStage,
    pub cause: EvidenceCause,
    pub cause_detail: String,
    pub reason_number: Option<usize>,
    pub path: Option<String>,
    pub target_label: Option<String>,
    pub evidence: Option<QuoteEvidence>,
    pub legacy_text: String,
}
fn validate_quote(source: &Source, quote: &Quote, missing_source: &str) -> Result<(), (EvidenceCause, String)> {
    quote.exchange_index.checked_add(1).ok_or_else(|| (EvidenceCause::MissingSnapshot, missing_source.into()))?;
    let snapshot = source.snapshots.iter().find(|s| s.exchange_index == quote.exchange_index)
        .ok_or_else(|| (EvidenceCause::MissingSnapshot, missing_source.into()))?;
    let original = match quote.role.as_str() {
        "user" => &snapshot.exchange.question,
        "assistant" => &snapshot.exchange.answer,
        _ => return Err((EvidenceCause::InvalidRole, "引用の話者が不正です。".into())),
    };
    let (cause, reason) = if quote.quote.trim().is_empty() {
        (EvidenceCause::EmptyQuote, "引用が空白のみです")
    } else if quote.quote.chars().count() > 2000 {
        (EvidenceCause::LongQuote, "引用が上限2000文字を超えています")
    } else if !original.contains(&quote.quote) {
        (EvidenceCause::QuoteMismatch, "引用が指定した元発言に連続文字列として存在しないためです")
    } else {
        return Ok(());
    };
    Err((cause, reason.into()))
}

fn source_diagnostic(cause: &str) -> String {
    let evidence = if cause.contains("固定版") {
        "作成要求時の固定元発言を取得できない。"
    } else {
        "根拠参照を確認できない。"
    };
    format!(
        "結果: この案は登録できない。教材には登録していない。\n対象: 根拠参照の検査で止まったため、変更理由と教材項目は特定していない。\n原因: {cause}\n根拠: {evidence}\n比較: AIが示した引用と作成要求時の元発言は、この検査段階では比較していない。\n対処: 保存された案と作成要求時の根拠を確認し、必要なら教材案の作成を明示的に再試行してください。元チャットを現在編集しても固定本文は変わらない。\n技術情報: 根拠参照の検査で停止。変更理由番号・path・話者は未確認。"
    )
}

fn reason_diagnostic(
    source: &Source,
    entry: Option<&Value>,
    reason: &ChangeReason,
    number: usize,
    cause: &str,
    failed_quote: Option<&Quote>,
    stage: DiagnosticStage,
) -> String {
    let result = match stage {
        DiagnosticStage::Generation => "教材案を作成できなかった。教材には登録していない。",
        DiagnosticStage::SavedDraft => "この案は登録できない。教材には登録していない。",
    };
    let target = if allowed_path(&reason.path)
        && entry.and_then(|value| value.pointer(&reason.path)).is_some()
    {
        crate::material_diff::reason_path_label(&reason.path)
    } else {
        None
    };
    let target = target.map_or_else(
        || format!("AIが返した{number}番目の変更理由は対象項目を特定できない。"),
        |label| format!("AIが返した{number}番目の変更理由で、教材の「{label}」を確認できない。"),
    );
    let (evidence, quote, original, technical_quote) = if let Some(quote) = failed_quote {
        let turn = quote.exchange_index.checked_add(1);
        let snapshot = source.snapshots.iter().find(|snapshot| snapshot.exchange_index == quote.exchange_index);
        let speaker = match quote.role.as_str() {
            "user" => Some(("あなたの質問", "あなた（質問）")),
            "assistant" => Some(("Codexの回答", "Codex（AIの回答）")),
            _ => None,
        };
        let evidence = match (turn, snapshot, speaker) {
            (Some(turn), Some(_), Some((plain, detailed))) =>
                format!("{turn}組目の質問と回答のうち、{plain}（往復 {turn}・{detailed}）。"),
            (Some(turn), None, _) => format!("指定された{turn}組目の固定元発言を取得できない。"),
            (Some(turn), Some(_), None) => format!("指定された{turn}組目の話者を特定できない。"),
            (None, _, _) => "指定された往復番号を特定できない。".into(),
        };
        let original = match (snapshot, speaker) {
            (Some(snapshot), Some(("あなたの質問", _))) => quote_preview(&snapshot.exchange.question),
            (Some(snapshot), Some(("Codexの回答", _))) => quote_preview(&snapshot.exchange.answer),
            (None, _) => "固定元発言を取得できない".into(),
            _ => "話者を特定できない".into(),
        };
        (
            evidence,
            quote_preview(&quote.quote),
            original,
            format!("、role {}、exchange_index {}", quote_preview(&quote.role), quote.exchange_index),
        )
    } else {
        (
            "引用の照合前に失敗したため、根拠の発言は特定していない。".into(),
            "この検査段階では比較していない".into(),
            "この検査段階では比較していない".into(),
            String::new(),
        )
    };
    format!(
        "結果: {result}\n対象: {target} 番号はAI応答内の変更理由の順序であり、教材番号や会話番号ではない。\n原因: {cause}\n根拠: {evidence}\n比較（それぞれ先頭240文字の抜粋）:\nAIが返した引用: {quote}\n作成要求時の元発言: {original}\n対処: 選択した元チャットの発言と教材項目を確認し、必要なら教材案の作成を明示的に再試行してください。AIには固定した元発言を正確に引用し直させてください。現在の元チャットを編集しても作成要求時の固定本文は変わらない。原文を誤引用に合わせて書き換えないでください。再試行の成功は保証されない。\n技術情報: 変更理由 {number}、path {}{technical_quote}",
        quote_preview(&reason.path)
    )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeReason { pub path: String, pub reason: String, pub quotes: Vec<Quote> }
const REASON_TOP_FIELDS: &[&str] = &["base", "meaning", "level", "business", "elevated", "register", "usage", "context", "example", "translation", "answers", "question", "explanation", "tag"];
const REASON_EXAMPLE_FIELDS: &[&str] = &["english", "japanese", "note"];
const REASON_REPLACEMENT_FIELDS: &[&str] = &["phrase", "meaning", "conditions"];

pub fn reason_path_instructions() -> String {
    format!(
        "変更理由のpathはJSON Pointerで、許可するトップレベル項目は /{}。配列内は /examples/N/{} または /replacements/N/{} の個別フィールドだけを指定する。Nは対象配列の実在する0始まりの添字である。例: /examples/0/note、/replacements/0/conditions。/examples/0 や /replacements/0 のような配列要素全体は指定せず、変更した個別フィールドごとに理由を分ける。",
        REASON_TOP_FIELDS.join("、/"), REASON_EXAMPLE_FIELDS.join("・"), REASON_REPLACEMENT_FIELDS.join("・")
    )
}

fn validate_reason(source: &Source, entry: Option<&Value>, reason: &ChangeReason, number: usize, missing_source: &str, stage: DiagnosticStage) -> Result<(), MaterialFailure> {
    let mut failed_quote = None;
    let mut check = || -> Result<(), (EvidenceCause, String)> {
        if !allowed_path(&reason.path) {
            return Err((EvidenceCause::InvalidPath, "path形式が不正です。許可形式はトップレベルの指定項目、/examples/N/english・japanese・note、/replacements/N/phrase・meaning・conditionsです。配列の要素全体ではなく個別フィールドを指定してください。".into()));
        }
        if entry.and_then(|value| value.pointer(&reason.path)).is_none() {
            return Err((EvidenceCause::MissingItem, "pathの参照先項目が存在しません。".into()));
        }
        if reason.reason.trim().is_empty() {
            return Err((EvidenceCause::EmptyReason, "説明が空白のみです。".into()));
        }
        if reason.reason.chars().count() > 2000 {
            return Err((EvidenceCause::LongReason, "説明が上限2000文字を超えています。".into()));
        }
        if reason.quotes.is_empty() {
            return Err((EvidenceCause::MissingQuotes, "引用が0件です。1件以上指定してください。".into()));
        }
        if reason.quotes.len() > 10 {
            return Err((EvidenceCause::TooManyQuotes, "引用が上限10件を超えています。".into()));
        }
        for quote in &reason.quotes {
            if let Err(cause) = validate_quote(source, quote, missing_source) {
                failed_quote = Some(quote);
                return Err(cause);
            }
        }
        Ok(())
    };
    check().map_err(|(cause, cause_detail)| {
        let original = failed_quote.and_then(|quote| {
            let snapshot = source.snapshots.iter().find(|s| s.exchange_index == quote.exchange_index)?;
            match quote.role.as_str() {
                "user" => Some(snapshot.exchange.question.clone()),
                "assistant" => Some(snapshot.exchange.answer.clone()),
                _ => None,
            }
        });
        let evidence = failed_quote.map(|quote| QuoteEvidence {
            exchange_index: quote.exchange_index,
            role: quote.role.clone(),
            quote: quote.quote.clone(),
            original,
        });
        let target_label = if allowed_path(&reason.path)
            && entry.and_then(|value| value.pointer(&reason.path)).is_some() {
            crate::material_diff::reason_path_label(&reason.path)
        } else { None };
        MaterialFailure::Evidence(MaterialDiagnostic {
            stage, cause, cause_detail: cause_detail.clone(), reason_number: Some(number),
            path: Some(reason.path.clone()), target_label, evidence,
            legacy_text: reason_diagnostic(source, entry, reason, number, &cause_detail, failed_quote, stage),
        })
    })
}
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
        self.build_response_detailed(text).map_err(|failure| failure.legacy_message())
    }
    pub fn build_response_detailed(&self, text: &str) -> Result<Draft, MaterialFailure> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Response { entry: Entry, reasons: Vec<ChangeReason> }
        let response: Response = serde_json::from_str(text).map_err(|e| MaterialFailure::Other(format!("教材案の形式が不正です: {e}")))?;
        if response.reasons.len() > 100 { return Err(MaterialFailure::Other("変更理由が多すぎます。".into())); }
        let raw = serde_json::to_value(&response.entry).map_err(|e| e.to_string())?;
        let mut draft = self.build(response.entry.clone())?;
        let merged = serde_json::to_value(&draft.candidate).map_err(|e| e.to_string())?;
        let old = self.baseline.as_ref().map(|v| serde_json::to_value(v).unwrap());
        for (index, mut reason) in response.reasons.into_iter().enumerate() {
            validate_reason(&self.source, Some(&raw), &reason, index + 1, "選択していない往復が根拠に指定されました。", DiagnosticStage::Generation)?;
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
    let Some(body) = path.strip_prefix('/') else { return false; };
    let parts: Vec<_> = body.split('/').collect();
    if let [field] = parts.as_slice() { return REASON_TOP_FIELDS.contains(field); }
    let [array, index, field] = parts.as_slice() else { return false; };
    if !(*index == "0" || (!index.starts_with('0') && index.bytes().all(|byte| byte.is_ascii_digit())))
        || index.parse::<usize>().is_err() { return false; }
    match *array {
        "examples" => REASON_EXAMPLE_FIELDS.contains(field),
        "replacements" => REASON_REPLACEMENT_FIELDS.contains(field),
        _ => false,
    }
}
pub fn response_schema(entry: Value) -> Value {
    let path_pattern = format!("^/({}|examples/(0|[1-9][0-9]*)/({})|replacements/(0|[1-9][0-9]*)/({}))$",
        REASON_TOP_FIELDS.join("|"), REASON_EXAMPLE_FIELDS.join("|"), REASON_REPLACEMENT_FIELDS.join("|"));
    json!({"type":"object","properties":{"entry":entry,"reasons":{"type":"array","items":{
        "type":"object","properties":{"path":{"type":"string","pattern":path_pattern},"reason":{"type":"string"},"quotes":{"type":"array","minItems":1,"maxItems":10,"items":{
            "type":"object","properties":{"exchange_index":{"type":"integer","minimum":0},"role":{"type":"string","enum":["user","assistant"]},"quote":{"type":"string"}},
            "required":["exchange_index","role","quote"],"additionalProperties":false}}},"required":["path","reason","quotes"],"additionalProperties":false},"maxItems":100}},"required":["entry","reasons"],"additionalProperties":false})
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
    fn quote_response(entry: &Entry, index: usize, role: &str, quote: &str) -> String {
        json!({"entry":entry,"reasons":[{"path":"/usage","reason":"合成会話の説明を反映した。",
            "quotes":[{"exchange_index":index,"role":role,"quote":quote}]}]}).to_string()
    }
    fn quote_request(question: &str, answer: &str) -> (Request, Entry) {
        let old = old_entry();
        let mut conversation = Conversation::new();
        conversation.complete(question.into(), answer.into(), Execution::default()).unwrap();
        conversation.exchanges[0].for_material = true;
        for i in 1..3 {
            conversation.complete(format!("選択外の質問{i}"), format!("選択外の回答{i}"), Execution::default()).unwrap();
        }
        conversation.complete("4番目の質問 🦊".into(), "4番目の回答 **強調**".into(), Execution::default()).unwrap();
        conversation.exchanges[3].for_material = true;
        let request = Request::new(&conversation, &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old;
        candidate.usage = "合成会話に沿って訂正".into();
        (request, candidate)
    }
    #[test]
    fn feedback_ui_generation_quote_error_explains_seventh_reason_and_first_failure() {
        let (request, candidate) = quote_request("質問 🦊", "4番目とは別の回答 **強調**");
        let mut reasons = Vec::new();
        for _ in 0..6 {
            reasons.push(ChangeReason {
                path: "/usage".into(),
                reason: "合成の説明".into(),
                quotes: vec![Quote { exchange_index: 0, role: "assistant".into(), quote: "4番目とは別の回答".into() }],
            });
        }
        reasons.push(ChangeReason {
            path: "/usage".into(),
            reason: "合成の説明".into(),
            quotes: vec![Quote { exchange_index: 3, role: "assistant".into(), quote: "4番目の回答 強調".into() }],
        });
        reasons.push(ChangeReason {
            path: "/meaning".into(),
            reason: "別の理由".into(),
            quotes: vec![Quote { exchange_index: 0, role: "assistant".into(), quote: "別の誤引用".into() }],
        });
        let error = request.build_response(&serde_json::json!({"entry":candidate,"reasons":reasons}).to_string()).unwrap_err();
        for expected in ["結果", "対象", "原因", "根拠", "比較", "対処", "7番目", "説明・使い方", "4組目", "Codex", "4番目の回答 強調", "4番目の回答 **強調**"] {
            assert!(error.contains(expected), "{expected:?} がない: {error}");
        }
        assert!(error.contains("登録していない"), "失敗段階が不明: {error}");
        assert!(!error.contains("別の誤引用"), "後続理由を混入: {error}");
    }

    #[test]
    fn feedback_ui_saved_draft_quote_error_identifies_fixed_source_and_nonregistration() {
        let (request, candidate) = quote_request("質問 🦊", "固定された回答 **強調**");
        let mut draft = request.build_response(&quote_response(&candidate, 0, "assistant", "固定された回答")).unwrap();
        draft.reasons[0].quotes[0].quote = "固定された回答 強調".into();
        let error = draft.validate_evidence().unwrap_err();
        for expected in ["この案", "登録できない", "説明・使い方", "作成要求時", "固定された回答 **強調**", "固定された回答 強調", "再試行"] {
            assert!(error.contains(expected), "{expected:?} がない: {error}");
        }
        assert_eq!(draft.source.snapshots[0].exchange.answer, "固定された回答 **強調**");
    }

    #[test]
    fn feedback_ui_quote_diagnostic_names_only_an_existing_material_field() {
        let (request, mut candidate) = quote_request("質問 🦊", "固定した回答 **強調**");
        candidate.examples.push(another_example());
        let invalid_quote = Quote { exchange_index: 0, role: "assistant".into(), quote: "固定した回答 強調".into() };
        let valid_path = ChangeReason { path: "/examples/1/note".into(), reason: "合成の説明".into(), quotes: vec![invalid_quote] };
        let error = request.build_response(&format_response(&candidate, &[valid_path])).unwrap_err();
        assert!(error.contains("2件目の追加例文の補足"), "実在項目の人向け名: {error}");
        for path in ["/examples/2/note", "/id"] {
            let reason = ChangeReason { path: path.into(), reason: "合成の説明".into(), quotes: vec![Quote { exchange_index: 0, role: "assistant".into(), quote: "固定した回答".into() }] };
            let error = request.build_response(&format_response(&candidate, &[reason])).unwrap_err();
            assert!(error.contains("対象項目を特定できない"), "不存在の対象を名付けた: {error}");
            assert!(!error.contains("3件目の追加例文"), "存在しない配列添字を丸めた: {error}");
            assert!(error.contains(&serde_json::to_string(path).unwrap()), "技術識別値を失った: {error}");
        }
    }

    #[test]
    fn feedback_ui_missing_turn_and_invalid_role_never_borrow_another_fixed_utterance() {
        let (request, candidate) = quote_request("あなたの質問 🦊", "Codexの固定回答 **強調**");
        let missing = request.build_response(&quote_response(&candidate, 1, "assistant", "選択外の回答")).unwrap_err();
        assert!(missing.contains("2組目") && missing.contains("固定元発言を取得できない"), "欠落参照: {missing}");
        assert!(!missing.contains("Codexの固定回答 **強調**"), "別往復の原文を代用: {missing}");
        let role = request.build_response(&quote_response(&candidate, 0, "system", "架空の発言")).unwrap_err();
        assert!(role.contains("話者を特定できない"), "不正role: {role}");
        assert!(!role.contains("Codexの固定回答 **強調**") && !role.contains("あなたの質問 🦊"), "話者を推測して原文を代用: {role}");
        let overflow = request.build_response(&quote_response(&candidate, usize::MAX, "assistant", "架空の発言")).unwrap_err();
        assert!(overflow.contains("往復番号を特定できない"), "添字overflow: {overflow}");
        assert!(!overflow.contains("0組目") && !overflow.contains("Codexの固定回答 **強調**"), "別往復へ丸めた: {overflow}");
    }

    #[test]
    fn feedback_ui_saved_draft_source_failure_stops_before_reason_or_quote_diagnosis() {
        let (request, candidate) = quote_request("質問 🦊", "固定された回答 **強調**");
        let mut draft = request.build_response(&quote_response(&candidate, 0, "assistant", "固定された回答")).unwrap();
        draft.source.snapshots.remove(0);
        draft.reasons[0].quotes[0].quote = "別の誤引用".into();
        let error = draft.validate_evidence().unwrap_err();
        for expected in ["結果:", "この案は登録できない", "根拠参照", "固定元発言を取得できない", "比較:", "対処:"] {
            assert!(error.contains(expected), "根拠段階の表示から{expected:?}が欠落: {error}");
        }
        assert!(!error.contains("変更理由 1、path"), "未検査の理由や引用を失敗原因へ昇格: {error}");
        assert!(!error.contains("別の誤引用"), "未検査の引用を比較したと誤案内: {error}");
    }

    #[test]
    fn feedback_ui_reason_validation_keeps_path_before_explanation_and_quote_checks() {
        let (request, candidate) = quote_request("質問 🦊", "固定された回答");
        let base = request.build_response(&quote_response(&candidate, 0, "assistant", "固定された回答")).unwrap();
        for (path, cause) in [("/id", "path形式"), ("/examples/99/note", "pathの参照先項目が存在しません")] {
            let reason = ChangeReason { path: path.into(), reason: " \n".into(), quotes: vec![] };
            let generated = request.build_response(&format_response(&candidate, &[reason.clone()])).unwrap_err();
            let mut saved = base.clone();
            saved.reasons = vec![reason];
            let saved = saved.validate_evidence().unwrap_err();
            for error in [generated, saved] {
                assert!(error.contains(cause), "path前置検査を失った: {error}");
                assert!(!error.contains("引用が0件") && !error.contains("説明が空白のみ"), "後段原因を混入: {error}");
            }
        }
    }
    #[test]
    fn quote_rejection_distinguishes_reference_role_empty_length_and_mismatch() {
        let (request, candidate) = quote_request("質問 🦊", "Use **improve clarity** here.");
        let cases = [
            (1, "assistant", "選択外の回答1", "選択していない往復"),
            (0, "system", "質問 🦊", "話者"),
            (0, "assistant", " \n\t", "空"),
            (0, "assistant", &"x".repeat(2001), "2000"),
            (0, "assistant", "Use improve clarity here.", "連続"),
        ];
        for (index, role, quote, reason) in cases {
            let error = request.build_response(&quote_response(&candidate, index, role, quote)).unwrap_err();
            assert!(error.contains(reason), "{index} {role}: {error}");
            if index == 0 && role == "assistant" && quote.trim().is_empty() {
                assert!(!error.contains("存在しない"), "空白を非包含と誤案内: {error}");
            }
            if quote.chars().count() > 2000 {
                assert!(!error.contains("存在しない"), "過長を非包含と誤案内: {error}");
            }
        }
    }
    #[test]
    fn quote_rejection_identifies_original_nonconsecutive_turn_and_both_roles() {
        let (request, candidate) = quote_request("質問 🦊", "Use **improve clarity** here.");
        assert_eq!(request.source.exchange_indices, vec![0, 3]);
        for (index, role, rejected, source, label, turn) in [
            (0, "user", "質問 🐺", "質問 🦊", "あなた（質問）", "往復 1"),
            (3, "assistant", "4番目の回答 強調", "4番目の回答 **強調**", "Codex（AIの回答）", "往復 4"),
        ] {
            let error = request.build_response(&quote_response(&candidate, index, role, rejected)).unwrap_err();
            for expected in [turn, label, "AIが返した引用", "作成要求時の元発言", rejected, source] {
                assert!(error.contains(expected), "{expected:?} がない: {error}");
            }
            assert!(error.contains("元チャット"), "確認先がない: {error}");
            assert!(error.contains("編集") || error.contains("変更"), "固定本文との差が説明されない: {error}");
            assert!(!error.contains("選択外の回答"), "別往復が混入: {error}");
        }
    }
    #[test]
    fn quote_matching_remains_exact_and_accepts_2000_unicode_scalars() {
        let source = format!("{}Use **improve clarity** here.\n次へ", "界".repeat(2000));
        let (request, candidate) = quote_request("質問 🦊", &source);
        for quote in ["**improve clarity**", "界".repeat(2000).as_str(), "here.\n次へ"] {
            let result = request.build_response(&quote_response(&candidate, 0, "assistant", quote));
            assert!(result.is_ok(), "正しい連続部分引用: {result:?}");
        }
        for altered in ["Use improve clarity here.", "**Improve clarity**", "here. 次へ", "界".repeat(2001).as_str()] {
            let result = request.build_response(&quote_response(&candidate, 0, "assistant", altered));
            assert!(result.is_err(), "改変・過長引用を受理: {altered:?}");
        }
    }
    #[test]
    fn quote_previews_escape_before_display_and_mark_only_true_truncation() {
        let prefix = format!("{}\n\t\"\\🦊**", "あ".repeat(233));
        assert_eq!(prefix.chars().count(), 240);
        let source = format!("{prefix}末");
        let (request, candidate) = quote_request("質問 🦊", &source);
        let rejected = format!("{prefix}別");
        let error = request.build_response(&quote_response(&candidate, 0, "assistant", &rejected)).unwrap_err();
        let expected_quote = format!("{}…（省略）", serde_json::to_string(&prefix).unwrap());
        assert!(error.contains(&expected_quote), "241文字引用の抜粋: {error}");
        assert_eq!(error.matches("…（省略）").count(), 2, "引用・元文を個別に省略: {error}");
        assert!(error.contains("先頭") || error.contains("抜粋"), "先頭抜粋の説明がない: {error}");
        let exactly_240 = request.build_response(&quote_response(&candidate, 0, "assistant", &prefix)).unwrap();
        assert_eq!(exactly_240.reasons[0].quotes[0].quote.chars().count(), 240);
        let (request, candidate) = quote_request("質問 🦊", "short");
        let error = request.build_response(&quote_response(&candidate, 0, "assistant", "other")).unwrap_err();
        assert!(!error.contains("…（省略）"), "短文に省略表示: {error}");
    }
    #[test]
    fn saved_draft_rechecks_quotes_without_repairing_or_rebinding_sources() {
        let (request, candidate) = quote_request("質問 🦊", "Use **improve clarity** here.");
        let valid = request.build_response(&quote_response(&candidate, 0, "assistant", "**improve clarity**")).unwrap();
        let serialized = serde_json::to_string(&valid).unwrap();
        let restored: Draft = serde_json::from_str(&serialized).unwrap();
        assert!(restored.validate_evidence().is_ok());
        let mut legacy = restored.clone();
        legacy.reasons.clear();
        legacy.generated = None;
        assert!(serde_json::from_str::<Draft>(&serde_json::to_string(&legacy).unwrap()).unwrap().validate_evidence().is_ok());
        for (quote, reason) in [(" \n", "空"), (&"x".repeat(2001), "2000"), ("Use improve clarity here.", "連続")] {
            let mut invalid = restored.clone();
            invalid.reasons[0].quotes[0].quote = quote.into();
            let invalid: Draft = serde_json::from_str(&serde_json::to_string(&invalid).unwrap()).unwrap();
            let error = invalid.validate_evidence().unwrap_err();
            assert!(error.contains(reason), "保存案 {quote:?}: {error}");
            assert!(error.contains("往復 1") && error.contains("Codex（AIの回答）"), "対象がない: {error}");
        }
        let mut missing = restored;
        missing.source.snapshots.remove(0);
        let missing: Draft = serde_json::from_str(&serde_json::to_string(&missing).unwrap()).unwrap();
        let error = missing.validate_evidence().unwrap_err();
        assert!(error.contains("固定版") || error.contains("引用元"), "欠落根拠の分類: {error}");
        assert!(!error.contains("連続"), "欠落を非包含と誤診: {error}");
    }
    #[test]
    fn quote_turn_number_handles_usize_max_and_last_valid_conversation_index() {
        let (request, candidate) = quote_request("質問 🦊", "回答 🦊");
        let error = request.build_response(&quote_response(&candidate, usize::MAX, "assistant", "回答 🦊")).unwrap_err();
        assert!(error.contains("選択していない") || error.contains("参照"), "無効参照: {error}");
        assert!(!error.contains("往復 0"), "番号の丸め: {error}");
        let mut last = request.clone();
        last.source.exchange_indices = vec![crate::chat::MAX_EXCHANGES - 1];
        last.source.snapshots = vec![Snapshot { exchange_index: crate::chat::MAX_EXCHANGES - 1, exchange: request.source.snapshots[0].exchange.clone() }];
        let error = last.build_response(&quote_response(&candidate, crate::chat::MAX_EXCHANGES - 1, "assistant", "欠落 🦊")).unwrap_err();
        assert!(error.contains("往復 200"), "最終往復の表示: {error}");
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

    fn format_reason(path: &str, explanation: &str, quote_count: usize) -> ChangeReason {
        ChangeReason { path: path.into(), reason: explanation.into(), quotes: (0..quote_count)
            .map(|_| Quote { exchange_index: 0, role: "assistant".into(), quote: "選択する回答".into() }).collect() }
    }
    fn format_response(entry: &Entry, reasons: &[ChangeReason]) -> String {
        json!({"entry":entry,"reasons":reasons}).to_string()
    }
    #[test]
    fn format_accepts_all_specified_existing_paths_and_boundary_indices() {
        let old = old_entry();
        let request = Request::new(&chat(), &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old;
        candidate.usage = "訂正した説明".into();
        candidate.examples.push(another_example());
        candidate.replacements.push(Replacement { phrase:"another phrase".into(), meaning:"別の意味".into(), conditions:"別の条件".into() });
        for path in ["/base","/meaning","/level","/business","/elevated","/register",
            "/usage","/context","/example","/translation","/answers","/question","/explanation","/tag",
            "/examples/0/english","/examples/0/japanese","/examples/0/note",
            "/examples/1/english","/examples/1/japanese","/examples/1/note",
            "/replacements/0/phrase","/replacements/0/meaning","/replacements/0/conditions",
            "/replacements/1/phrase","/replacements/1/meaning","/replacements/1/conditions"] {
            let reason = format_reason(path, "有効な説明", 1);
            assert!(request.build_response(&format_response(&candidate, &[reason])).is_ok(), "valid path {path}");
        }
    }
    #[test]
    fn format_rejects_whole_items_and_bad_indices_with_reason_number_and_path_in_both_entries() {
        let old = old_entry();
        let request = Request::new(&chat(), &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old;
        candidate.usage = "訂正した説明".into();
        candidate.examples.push(another_example());
        let valid = format_reason("/usage", "有効な説明", 1);
        let base = request.build_response(&format_response(&candidate, &[valid.clone()])).unwrap();
        for (number, path, cause) in [
            (5, "/replacements/0", "個別"), (6, "/examples/0", "個別"),
            (7, "/examples/1", "個別"), (1, "/id", "形式"),
            (1, "/examples/01/note", "形式"), (1, "/examples/+1/note", "形式"),
            (1, "/examples/-1/note", "形式"), (1, "/examples/1.0/note", "形式"),
            (1, "/examples//note", "形式"), (1, "/examples/0/note/extra", "形式"),
            (1, "/examples/0/unknown", "形式"),
            (1, "/examples/999999999999999999999999999999/note", "形式"),
            (1, "/examples/0/note\n", "形式"),
            (1, "/examples/2/note", "存在"), (1, "/replacements/1/meaning", "存在"),
        ] {
            let mut reasons = vec![valid.clone(); number - 1];
            reasons.push(format_reason(path, "有効な説明", 1));
            let generated = request.build_response(&format_response(&candidate, &reasons)).unwrap_err();
            let mut saved = base.clone();
            saved.reasons = reasons;
            let saved = saved.validate_evidence().unwrap_err();
            for error in [generated, saved] {
                assert!(error.contains(&format!("変更理由 {number}")), "{path}: {error}");
                assert!(error.contains(&serde_json::to_string(path).unwrap()), "{path}: {error}");
                assert!(error.contains(cause), "{path}: {error}");
                assert!(!error.contains("元の発言に存在しない引用"), "pathを引用不一致と混同: {error}");
            }
        }
        let mut empty_arrays = base;
        empty_arrays.generated.as_mut().unwrap().examples.clear();
        empty_arrays.reasons = vec![format_reason("/examples/0/note", "有効な説明", 1)];
        let error = empty_arrays.validate_evidence().unwrap_err();
        assert!(error.contains("存在"), "空配列の添字0: {error}");
    }
    #[test]
    fn format_distinguishes_explanation_and_quote_count_boundaries_in_both_entries() {
        let old = old_entry();
        let request = Request::new(&chat(), &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old;
        candidate.usage = "訂正した説明".into();
        let base = request.build_response(&format_response(&candidate,
            &[format_reason("/usage", "有効な説明", 1)])).unwrap();
        for (explanation, count, cause) in [
            (" \n".to_owned(), 1, "空"), ("界".repeat(2001), 1, "2000"),
            ("有効な説明".to_owned(), 0, "0"), ("有効な説明".to_owned(), 11, "10"),
        ] {
            let reason = format_reason("/usage", &explanation, count);
            let generated = request.build_response(&format_response(&candidate, &[reason.clone()])).unwrap_err();
            let mut saved = base.clone();
            saved.reasons = vec![reason];
            let saved = saved.validate_evidence().unwrap_err();
            for error in [generated, saved] {
                assert!(error.contains("変更理由 1") && error.contains("/usage") && error.contains(cause), "{error}");
            }
        }
        for (explanation, count) in [("界".repeat(2000), 1), ("有効な説明".to_owned(), 10)] {
            let reason = format_reason("/usage", &explanation, count);
            assert!(request.build_response(&format_response(&candidate, &[reason])).is_ok());
        }
    }
    #[test]
    fn format_preserves_quote_diagnostics_and_saved_fixed_evidence() {
        let (request, candidate) = quote_request("質問 🦊", "Use **improve clarity** here.");
        let mut valid = format_reason("/usage", "有効な説明", 1);
        valid.quotes[0].quote = "**improve clarity**".into();
        let mut invalid = valid.clone();
        invalid.quotes[0] = Quote { exchange_index:0, role:"assistant".into(), quote:"Use improve clarity here.".into() };
        let error = request.build_response(&format_response(&candidate, &[invalid.clone()])).unwrap_err();
        for part in ["変更理由 1", "/usage", "連続", "往復 1", "Codex（AIの回答）", "**improve clarity**"] {
            assert!(error.contains(part), "{part}: {error}");
        }
        let mut saved = request.build_response(&format_response(&candidate, &[valid])).unwrap();
        saved.reasons = vec![invalid];
        let serialized = serde_json::to_string(&saved).unwrap();
        let restored: Draft = serde_json::from_str(&serialized).unwrap();
        let error = restored.validate_evidence().unwrap_err();
        assert!(error.contains("変更理由 1") && error.contains("/usage") && error.contains("連続") && error.contains("**improve clarity**"), "{error}");
    }
    #[test]
    fn format_path_preview_escapes_controls_and_truncates_only_after_240_scalars() {
        let old = old_entry();
        let request = Request::new(&chat(), &old.base, Mode::Correct, Some(old.clone())).unwrap();
        let mut candidate = old;
        candidate.usage = "訂正した説明".into();
        let base = request.build_response(&format_response(&candidate,
            &[format_reason("/usage", "有効な説明", 1)])).unwrap();
        let suffix = "\n\t\"\\🦊";
        let path_240 = format!("/{}{}", "あ".repeat(239 - suffix.chars().count()), suffix);
        assert_eq!(path_240.chars().count(), 240);
        let path_241 = format!("{path_240}末");
        for path in [&path_240, &path_241] {
            let reason = format_reason(path, "有効な説明", 1);
            let generated = request.build_response(&format_response(&candidate, &[reason.clone()])).unwrap_err();
            let mut saved = base.clone();
            saved.reasons = vec![reason];
            let saved = saved.validate_evidence().unwrap_err();
            for error in [generated, saved] {
                assert!(error.contains("変更理由 1"), "{error}");
                let escaped_prefix = serde_json::to_string(&path_240).unwrap();
                if path.chars().count() == 240 {
                    assert!(error.contains(&escaped_prefix), "240文字の制御文字escape: {error}");
                    assert!(!error.contains("…（省略）"), "240文字で省略: {error}");
                } else {
                    assert!(error.contains(&format!("{escaped_prefix}…（省略）")), "241文字の先頭抜粋: {error}");
                    assert!(!error.contains(&serde_json::to_string(path).unwrap()), "全文pathを表示: {error}");
                }
            }
        }
    }
    #[test]
    fn format_schema_constrains_paths_and_reason_sizes_without_changing_structure() {
        let schema = response_schema(json!({"type":"object"}));
        let path = &schema["properties"]["reasons"]["items"]["properties"]["path"];
        assert_eq!(path["type"], "string");
        assert_eq!(path["pattern"], "^/(base|meaning|level|business|elevated|register|usage|context|example|translation|answers|question|explanation|tag|examples/(0|[1-9][0-9]*)/(english|japanese|note)|replacements/(0|[1-9][0-9]*)/(phrase|meaning|conditions))$");
        assert_eq!(schema["properties"]["reasons"]["maxItems"], 100);
        let quotes = &schema["properties"]["reasons"]["items"]["properties"]["quotes"];
        assert_eq!(quotes["minItems"], 1);
        assert_eq!(quotes["maxItems"], 10);
        assert_eq!(quotes["items"]["properties"]["exchange_index"]["minimum"], 0);
        assert_eq!(schema["required"], json!(["entry","reasons"]));
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["properties"]["reasons"]["items"]["required"], json!(["path","reason","quotes"]));
        assert_eq!(schema["properties"]["reasons"]["items"]["additionalProperties"], false);
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
        self.validate_evidence_detailed().map_err(|failure| failure.legacy_message())
    }
    pub fn validate_evidence_detailed(&self) -> Result<(), MaterialFailure> {
        self.source.validate().map_err(|cause| MaterialFailure::Evidence(MaterialDiagnostic {
            stage: DiagnosticStage::SavedDraft, cause: EvidenceCause::SourceReference,
            cause_detail: cause.clone(), reason_number: None, path: None,
            target_label: None, evidence: None, legacy_text: source_diagnostic(&cause),
        }))?;
        if self.reasons.len()>100 {return Err("変更理由が多すぎます。".into());}
        let generated=self.generated.as_ref().map(serde_json::to_value).transpose().map_err(|e|e.to_string())?;
        for (index, reason) in self.reasons.iter().enumerate() {
            validate_reason(&self.source, generated.as_ref(), reason, index + 1, "引用元がありません。", DiagnosticStage::SavedDraft)?;
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
