//! Validated learning content; AI output never changes grades automatically.
use crate::model::{self, Entry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const NGSL_PAGE: &str = "https://www.newgeneralservicelist.com/new-general-service-list";
pub const NGSL_CSV: &str = "https://www.newgeneralservicelist.com/s/NGSL_12_stats.csv";
pub const NGSL_SUPPLEMENT: &str = "https://www.newgeneralservicelist.com/s/SUP_lemmatized.csv";
pub const BUNDLED_NGSL: &str = include_str!("../data/ngsl-1.2.txt");
pub const EXTRA_DECK: &str = include_str!("../data/phrases.tsv");

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exercise {
    pub japanese: String,
    pub situation: String,
    pub target: String,
    pub reference: String,
}
impl Exercise {
    pub fn parse(text: &str, expected: &str) -> Result<Self, String> {
        let e: Self = serde_json::from_str(text).map_err(|e| format!("出題JSONが不正です: {e}"))?;
        for s in [&e.japanese, &e.situation, &e.target, &e.reference] {
            if s.trim().is_empty() || s.chars().count() > 600 {
                return Err("問題が空、または長すぎます。".into());
            }
        }
        if model::normalize(&e.target) != model::normalize(expected) {
            return Err("出題の対象表現が教材と異なります。".into());
        }
        if !e
            .japanese
            .chars()
            .any(|c| matches!(c,'\u{3040}'..='\u{30ff}'|'\u{4e00}'..='\u{9fff}'))
        {
            return Err("日本語の問題が返りませんでした。".into());
        }
        Ok(e)
    }
}
pub fn kind(e: &Entry) -> &'static str {
    if e.id.starts_with("pattern_") {
        "構文"
    } else if e.id.starts_with("phrase_") {
        "熟語"
    } else {
        "単語"
    }
}
pub fn target(e: &Entry) -> &str {
    if kind(e) == "構文" {
        &e.base
    } else {
        e.answer()
    }
}

/// Headword first, or rank followed by headword. Supports quoted single-line CSV.
pub fn parse_words(text: &str) -> Result<Vec<String>, String> {
    if text.len() > 2_000_000 || text.trim_start().starts_with('<') {
        return Err("語彙CSVではない、または2MBを超えています。".into());
    }
    let mut words = Vec::new();
    let mut seen = BTreeSet::new();
    let mut first_record = true;
    let mut word_column = None;
    for line in text.trim_start_matches('\u{feff}').lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut cols = Vec::new();
        let mut field = String::new();
        let mut quoted = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '"' if quoted && chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => quoted = !quoted,
                ',' | '\t' if !quoted => {
                    cols.push(field);
                    field = String::new();
                }
                _ => field.push(c),
            }
        }
        if quoted {
            return Err("CSVの引用符が閉じていません。複数行セルには対応していません。".into());
        }
        cols.push(field);
        if first_record && cols.len() > 1 {
            let header = cols.iter().position(|c| {
                ["word", "headword", "lemma", "ngsl", "ngsl 1.2"]
                    .contains(&c.trim().to_lowercase().as_str())
            });
            if let Some(pos) = header {
                word_column = Some(pos);
                first_record = false;
                continue;
            }
        }
        let pos = if cols[0].trim().parse::<usize>().is_ok() && word_column.unwrap_or(0) == 0 {
            1
        } else {
            word_column.unwrap_or(0)
        };
        let word = cols
            .get(pos)
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default();
        let header = first_record
            && cols.len() > 1
            && [
                "word",
                "headword",
                "lemma",
                "rank",
                "frequency",
                "ngsl",
                "ngsl 1.2",
            ]
            .contains(&word.as_str());
        first_record = false;
        if header {
            continue;
        }
        if word.is_empty()
            || word.len() > 80
            || !word.chars().any(|c| c.is_ascii_alphabetic())
            || !word
                .chars()
                .all(|c| c.is_ascii_alphabetic() || matches!(c, '\'' | '-' | ' '))
        {
            return Err("CSVの基本語欄を読めません。先頭列を基本語（または順位・基本語の順）にしてください。".into());
        }
        if seen.insert(word.clone()) {
            words.push(word);
        }
        if words.len() > 10000 {
            return Err("語彙は10,000語以下にしてください。".into());
        }
    }
    if words.is_empty() {
        return Err("語彙がありません。".into());
    }
    Ok(words)
}
pub fn parse_draft(text: &str, word: &str) -> Result<Entry, String> {
    let mut e: Entry =
        serde_json::from_str(text).map_err(|e| format!("教材案JSONが不正です: {e}"))?;
    if model::normalize(&e.base) != model::normalize(word) {
        return Err("教材案の基本語が選択した語と異なります。".into());
    }
    e.id = format!(
        "web_{}",
        word.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    );
    e.level = "目安未判定".into();
    Ok(model::parse_deck(&model::deck_text(&[e]))?.remove(0))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_ngsl_files_and_bundle_are_consistent() {
        let main = parse_words(include_str!("../data/NGSL_12_stats.csv")).unwrap();
        let sup = parse_words(include_str!("../data/SUP_lemmatized.csv")).unwrap();
        assert_eq!(main.len(), 2809);
        assert_eq!(sup.len(), 52);
        assert_eq!(main[0], "the");
        let combined = parse_words(&[main.join("\n"), sup.join("\n")].join("\n")).unwrap();
        assert_eq!(combined, parse_words(BUNDLED_NGSL).unwrap());
        assert_eq!(combined.len(), 2859);
    }
    #[test]
    fn csv_quotes_ranks_duplicates_and_html() {
        assert_eq!(
            parse_words("headword,forms\n\"ask\",\"ask,asks\"\nask,asked\n2,answer\n").unwrap(),
            vec!["ask", "answer"]
        );
        assert_eq!(
            parse_words("word\nfrequency\nrank\n").unwrap(),
            vec!["word", "frequency", "rank"]
        );
        assert!(parse_words("<!DOCTYPE html>").is_err());
        assert!(parse_words("\"ask").is_err());
    }
    #[test]
    fn exercise_target_is_bound_to_card() {
        let s = r#"{"japanese":"返信の遅れを謝罪する。","situation":"取引先へのメール","target":"apologize","reference":"We apologize for the delay."}"#;
        assert!(Exercise::parse(s, "apologize").is_ok());
        assert!(Exercise::parse(s, "purchase").is_err());
    }
    #[test]
    fn extra_deck_has_valid_unique_ids() {
        let d = model::parse_deck(&format!("{}\n{}", model::BUILTIN_DECK, EXTRA_DECK)).unwrap();
        assert!(d.iter().any(|e| kind(e) == "構文"));
        assert!(d.iter().any(|e| kind(e) == "熟語"));
    }
    #[test]
    fn generated_id_cannot_replace_another_card() {
        let mut e = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
        e.id = "problem".into();
        assert!(parse_draft(&serde_json::to_string(&e).unwrap(), "sorry")
            .unwrap()
            .id
            .starts_with("web_"));
        assert!(parse_draft(&serde_json::to_string(&e).unwrap(), "problem").is_err());
    }
}
