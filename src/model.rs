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
}

impl Entry {
    pub fn answer(&self) -> &str { &self.answers[0] }
    pub fn completed(&self) -> String { self.example.replace("___", self.answer()) }
    pub fn accepts(&self, text: &str) -> bool {
        let n = normalize(text);
        self.answers.iter().any(|s| normalize(s) == n)
    }
    pub fn hint(&self, amount: usize) -> String {
        self.answer().chars().enumerate().map(|(i, c)| {
            if i < amount || c == ' ' || c == '-' { c } else { '＿' }
        }).collect()
    }
    pub fn to_tsv(&self)->String {
        let answers=self.answers.join("|");
        [&self.id,&self.base,&self.meaning,&self.level,&self.business,&self.elevated,&self.register,&self.usage,&self.context,&self.example,&self.translation,&answers,&self.question,&self.explanation,&self.tag]
            .iter().map(|s|s.replace(['\t','\n','\r']," ")).collect::<Vec<_>>().join("\t")
    }
    pub fn fingerprint(&self)->String {
        let mut hash=0xcbf29ce484222325_u64;
        for b in self.to_tsv().bytes(){hash^=b as u64;hash=hash.wrapping_mul(0x100000001b3);}
        format!("{hash:016x}")
    }
}

pub fn deck_text(deck:&[Entry])->String {
    let mut text="id\tbase\tmeaning\tlevel\tbusiness\televated\tregister\tusage\tcontext\texample\ttranslation\tanswers\tquestion\texplanation\ttag\n".to_string();
    for entry in deck{text.push_str(&entry.to_tsv());text.push('\n');}
    text
}

pub fn normalize(s: &str) -> String {
    s.trim().trim_end_matches(['.', '!', '?', '。']).replace('’', "'")
        .split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

pub fn parse_deck(text: &str) -> Result<Vec<Entry>, String> {
    if text.len() > 8_000_000 { return Err("教材は8MB以下にしてください。".into()); }
    let mut entries = Vec::new();
    let mut ids = BTreeSet::new();
    for (line_no, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        if raw.trim().is_empty() || raw.starts_with('#') { continue; }
        let c: Vec<&str> = raw.split('\t').collect();
        if c.first() == Some(&"id") && c.get(1)==Some(&"base") && c.get(14)==Some(&"tag") { continue; }
        if c.len() != 15 { return Err(format!("{}行目: 15列必要です（現在{}列）。", line_no + 1, c.len())); }
        if c.iter().any(|v| v.trim().is_empty() || v.len() > 4000) {
            return Err(format!("{}行目: 空欄または長すぎる項目があります。", line_no + 1));
        }
        if !c[0].bytes().all(|x| x.is_ascii_alphanumeric() || x == b'-' || x == b'_') || !ids.insert(c[0].to_string()) {
            return Err(format!("{}行目: IDは英数字・ハイフン・下線で一意にしてください。", line_no + 1));
        }
        if c[9].matches("___").count() != 1 {
            return Err(format!("{}行目: 例文には空欄 ___ が1個必要です。", line_no + 1));
        }
        let answers: Vec<String> = c[11].split('|').map(|s| s.trim().to_string()).collect();
        if answers.iter().any(String::is_empty) { return Err(format!("{}行目: 解答が空です。", line_no+1)); }
        entries.push(Entry {
            id: c[0].into(), base:c[1].into(), meaning:c[2].into(), level:c[3].into(),
            business:c[4].into(), elevated:c[5].into(), register:c[6].into(), usage:c[7].into(),
            context:c[8].into(), example:c[9].into(), translation:c[10].into(), answers,
            question:c[12].into(), explanation:c[13].into(), tag:c[14].into(),
        });
    }
    if entries.is_empty() || entries.len() > 10000 { return Err("教材は1〜10,000項目にしてください。".into()); }
    Ok(entries)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Skill { Recall, Usage, Listening, Sentence }
impl Skill {
    pub const ALL: [Self; 4] = [Self::Recall, Self::Usage, Self::Listening, Self::Sentence];
    pub fn code(self) -> &'static str { match self { Self::Recall=>"recall",Self::Usage=>"usage",Self::Listening=>"listening",Self::Sentence=>"sentence" } }
    pub fn label(self) -> &'static str { match self { Self::Recall=>"語句・綴り",Self::Usage=>"使い分け",Self::Listening=>"聞き取り",Self::Sentence=>"自分で作文" } }
    pub fn key(self, id: &str) -> String { format!("{}:{}", id, self.code()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn builtin_has_valid_unique_entries() {
        let d = parse_deck(BUILTIN_DECK).unwrap();
        assert!(d.len() >= 100);
        for e in d { assert!(!e.completed().contains("___")); assert!(e.accepts(e.answer())); }
    }
    #[test] fn normalization_is_not_semantic_guessing() {
        assert_eq!(normalize("  Apologize. "), "apologize");
        assert_ne!(normalize("apologize"), normalize("apologizes"));
    }
    #[test] fn broken_import_is_rejected() {
        assert!(parse_deck("a\tb").is_err());
        let d = BUILTIN_DECK.lines().find(|l| l.starts_with("sorry\t")).unwrap();
        assert!(parse_deck(&format!("{d}\n{d}")).is_err());
    }
}
