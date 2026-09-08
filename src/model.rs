use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const BUILTIN_DECK: &str = include_str!("../data/core.tsv");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub base: String,
    pub meaning: String,
    pub level: String,
    pub business: String,
    pub elevated: String,
    pub register: String,
    pub usage: String,
    pub context: String,
    pub example: String,
    pub translation: String,
    pub answers: Vec<String>,
    pub question: String,
    pub explanation: String,
    pub tag: String,
    #[serde(default)]
    pub replacements: Vec<Replacement>,
    #[serde(default)]
    pub examples: Vec<Example>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replacement {
    pub phrase: String,
    pub meaning: String,
    pub conditions: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub english: String,
    pub japanese: String,
    pub note: String,
}

pub fn validate_extras(e: &Entry) -> Result<(), String> {
    if e.replacements.len() > 30 || e.examples.len() > 200 {
        return Err("言い換えは30件、例文は200件までです。".into());
    }
    let mut phrases = BTreeSet::new();
    let mut sentences = BTreeSet::new();
    for r in &e.replacements {
        for s in [&r.phrase, &r.meaning, &r.conditions] {
            validate_text(s)?;
        }
        if !phrases.insert(normalize(&r.phrase)) {
            return Err("言い換えが重複しています。".into());
        }
    }
    for x in &e.examples {
        for s in [&x.english, &x.japanese, &x.note] {
            validate_text(s)?;
        }
        if x.english.contains("___") || !sentences.insert(normalize(&x.english)) {
            return Err("例文に空欄または重複があります。".into());
        }
    }
    Ok(())
}
fn validate_text(s: &str) -> Result<(), String> {
    if s.trim().is_empty() || s.len() > 4000 {
        Err("追加教材の欄が空、または長すぎます。".into())
    } else {
        Ok(())
    }
}

impl Entry {
    pub fn answer(&self) -> &str {
        &self.answers[0]
    }
    pub fn completed(&self) -> String {
        self.example.replace("___", self.answer())
    }
    pub fn accepts(&self, text: &str) -> bool {
        let n = normalize(text);
        self.answers.iter().any(|s| normalize(s) == n)
    }
    pub fn hint(&self, amount: usize) -> String {
        self.answer()
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i < amount || c == ' ' || c == '-' {
                    c
                } else {
                    '＿'
                }
            })
            .collect()
    }
    pub fn to_tsv(&self) -> String {
        let answers = self.answers.join("|");
        let mut line = [
            &self.id,
            &self.base,
            &self.meaning,
            &self.level,
            &self.business,
            &self.elevated,
            &self.register,
            &self.usage,
            &self.context,
            &self.example,
            &self.translation,
            &answers,
            &self.question,
            &self.explanation,
            &self.tag,
        ]
        .iter()
        .map(|s| s.replace(['\t', '\n', '\r'], " "))
        .collect::<Vec<_>>()
        .join("\t");
        if !self.replacements.is_empty() || !self.examples.is_empty() {
            line.push('\t');
            line.push_str(&serde_json::to_string(&self.replacements).unwrap());
            line.push('\t');
            line.push_str(&serde_json::to_string(&self.examples).unwrap());
        }
        line
    }
    pub fn fingerprint(&self) -> String {
        let mut hash = 0xcbf29ce484222325_u64;
        // Supplemental reading examples do not invalidate existing recall grades.
        let core = self
            .to_tsv()
            .split('\t')
            .take(15)
            .collect::<Vec<_>>()
            .join("\t");
        for b in core.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{hash:016x}")
    }
}

pub fn deck_text(deck: &[Entry]) -> String {
    let mut text="id\tbase\tmeaning\tlevel\tbusiness\televated\tregister\tusage\tcontext\texample\ttranslation\tanswers\tquestion\texplanation\ttag\treplacements\texamples\n".to_string();
    for entry in deck {
        text.push_str(&entry.to_tsv());
        text.push('\n');
    }
    text
}

pub fn normalize(s: &str) -> String {
    s.trim()
        .trim_end_matches(['.', '!', '?', '。'])
        .replace('’', "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn parse_deck(text: &str) -> Result<Vec<Entry>, String> {
    if text.len() > 64_000_000 {
        return Err("教材は64MB以下にしてください。".into());
    }
    let mut entries = Vec::new();
    let mut ids = BTreeSet::new();
    for (line_no, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let c: Vec<&str> = raw.split('\t').collect();
        if c.first() == Some(&"id") && c.get(1) == Some(&"base") && c.get(14) == Some(&"tag") {
            continue;
        }
        if c.len() != 15 && c.len() != 17 {
            return Err(format!(
                "{}行目: 15列または17列必要です（現在{}列）。",
                line_no + 1,
                c.len()
            ));
        }
        if c.iter()
            .take(15)
            .any(|v| v.trim().is_empty() || v.len() > 4000)
        {
            return Err(format!(
                "{}行目: 空欄または長すぎる項目があります。",
                line_no + 1
            ));
        }
        if !c[0]
            .bytes()
            .all(|x| x.is_ascii_alphanumeric() || x == b'-' || x == b'_')
            || !ids.insert(c[0].to_string())
        {
            return Err(format!(
                "{}行目: IDは英数字・ハイフン・下線で一意にしてください。",
                line_no + 1
            ));
        }
        if c[9].matches("___").count() != 1 {
            return Err(format!(
                "{}行目: 例文には空欄 ___ が1個必要です。",
                line_no + 1
            ));
        }
        let answers: Vec<String> = c[11].split('|').map(|s| s.trim().to_string()).collect();
        if answers.iter().any(String::is_empty) {
            return Err(format!("{}行目: 解答が空です。", line_no + 1));
        }
        let entry = Entry {
            id: c[0].into(),
            base: c[1].into(),
            meaning: c[2].into(),
            level: c[3].into(),
            business: c[4].into(),
            elevated: c[5].into(),
            register: c[6].into(),
            usage: c[7].into(),
            context: c[8].into(),
            example: c[9].into(),
            translation: c[10].into(),
            answers,
            question: c[12].into(),
            explanation: c[13].into(),
            tag: c[14].into(),
            replacements: if c.len() == 17 {
                serde_json::from_str(c[15]).map_err(|e| format!("言い換えJSON: {e}"))?
            } else {
                Vec::new()
            },
            examples: if c.len() == 17 {
                serde_json::from_str(c[16]).map_err(|e| format!("例文JSON: {e}"))?
            } else {
                Vec::new()
            },
        };
        validate_extras(&entry)?;
        entries.push(entry);
    }
    if entries.is_empty() || entries.len() > 10000 {
        return Err("教材は1〜10,000項目にしてください。".into());
    }
    Ok(entries)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Skill {
    Recall,
    Usage,
    Listening,
    Sentence,
}
impl Skill {
    pub const ALL: [Self; 4] = [Self::Recall, Self::Usage, Self::Listening, Self::Sentence];
    pub fn code(self) -> &'static str {
        match self {
            Self::Recall => "recall",
            Self::Usage => "usage",
            Self::Listening => "listening",
            Self::Sentence => "sentence",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Recall => "語句・綴り",
            Self::Usage => "使い分け",
            Self::Listening => "聞き取り",
            Self::Sentence => "日本語から英作文",
        }
    }
    pub fn key(self, id: &str) -> String {
        format!("{}:{}", id, self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_deck_roundtrips_without_resetting_core_learning() {
        let mut e = parse_deck(BUILTIN_DECK).unwrap().remove(0);
        let old = e.fingerprint();
        e.replacements.push(Replacement {
            phrase: "apologize for".into(),
            meaning: "〜について謝罪する".into(),
            conditions: "forの後ろは名詞または動名詞".into(),
        });
        e.examples.push(Example {
            english: "We apologize for the delay.".into(),
            japanese: "遅延をお詫びする。".into(),
            note: "謝罪の理由をforで示す。".into(),
        });
        let restored = parse_deck(&deck_text(&[e.clone()])).unwrap().remove(0);
        assert_eq!(restored.examples[0].english, e.examples[0].english);
        assert_eq!(restored.replacements[0].phrase, e.replacements[0].phrase);
        assert_eq!(restored.fingerprint(), old);
        e.examples.push(e.examples[0].clone());
        assert!(parse_deck(&deck_text(&[e])).is_err());
    }
    #[test]
    fn builtin_has_valid_unique_entries() {
        let d = parse_deck(BUILTIN_DECK).unwrap();
        assert!(d.len() >= 100);
        for e in d {
            assert!(!e.completed().contains("___"));
            assert!(e.accepts(e.answer()));
        }
    }
    #[test]
    fn normalization_is_not_semantic_guessing() {
        assert_eq!(normalize("  Apologize. "), "apologize");
        assert_ne!(normalize("apologize"), normalize("apologizes"));
    }
    #[test]
    fn broken_import_is_rejected() {
        assert!(parse_deck("a\tb").is_err());
        let d = BUILTIN_DECK
            .lines()
            .find(|l| l.starts_with("sorry\t"))
            .unwrap();
        assert!(parse_deck(&format!("{d}\n{d}")).is_err());
    }
}
