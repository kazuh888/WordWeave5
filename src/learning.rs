//! Validated learning content; AI output never changes grades automatically.
use crate::model::{self, Entry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const NGSL_PAGE: &str = "https://www.newgeneralservicelist.com/new-general-service-list";
pub const NGSL_CSV: &str = "https://www.newgeneralservicelist.com/s/NGSL_12_stats.csv";
pub const NGSL_SUPPLEMENT: &str = "https://www.newgeneralservicelist.com/s/SUP_lemmatized.csv";
pub const BUNDLED_NGSL: &str = include_str!("../data/ngsl-1.2.txt");
pub const EXTRA_DECK: &str = include_str!("../data/phrases.tsv");

pub enum VocabularyFile {
    Words(Vec<String>),
    Materials(Vec<Entry>),
}

/// Classify before parsing: a malformed material file must never become an AI word list.
pub fn parse_vocabulary_file(text: &str) -> Result<VocabularyFile, String> {
    if text.len() > 64_000_000 {
        return Err("教材ファイルは64MB以下にしてください。".into());
    }
    let text = text.trim_start_matches('\u{feff}');
    let (header_line, header) = text.lines().enumerate()
        .find(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
        .ok_or("ファイルに語彙や教材がありません。")?;
    let delimiter = if header.contains('\t') { '\t' } else { ',' };
    let columns: Vec<String> = header.split(delimiter)
        .map(|name| name.trim().trim_matches('"').to_ascii_lowercase()).collect();
    let canonical = model::deck_text(&[]);
    let expected: Vec<&str> = canonical.trim_end().split('\t').collect();
    let has_word_column = columns.iter().any(|c|
        ["word", "headword", "lemma", "ngsl", "ngsl 1.2"].contains(&c.as_str()));
    let material_like = columns.len() > 1 && (columns.iter().any(|c| c == "base")
        || columns.iter().any(|c| c == "id")
        || (!has_word_column && columns.len() >= 15)
        || columns.iter().filter(|c| expected[3..].contains(&c.as_str())).count() >= 3);
    if !material_like {
        return parse_words(text).map(VocabularyFile::Words);
    }
    let unique: BTreeSet<&str> = columns.iter().map(String::as_str).collect();
    let required = if columns.len() == 15 { &expected[..15] } else { &expected[..] };
    if delimiter != '\t' || ![15, 17].contains(&columns.len())
        || unique.len() != columns.len()
        || required.iter().any(|name| !unique.contains(name))
    {
        return Err("完成済み教材TSVの列名が不足・重複、または不明です。WordWeave5から書き出した15列または17列の見出しを使用してください。単語リストとしてのAI生成は行いません。".into());
    }
    let order: Vec<usize> = required.iter()
        .map(|name| columns.iter().position(|column| column == name).unwrap()).collect();
    let mut normalized = String::with_capacity(text.len());
    for (index, line) in text.lines().enumerate() {
        if index == header_line {
            normalized.push_str(&required.join("\t"));
        } else if index < header_line || line.trim().is_empty() || line.starts_with('#') {
            normalized.push_str(line);
        } else {
            let mut cells: Vec<&str> = line.split('\t').collect();
            // Existing exports use a 17-column header but omit the final two
            // optional JSON cells when both supplemental lists are empty.
            if columns.len() == 17 && cells.len() == 15
                && columns[15..].iter().all(|name| ["replacements", "examples"].contains(&name.as_str())) {
                cells.extend(["[]", "[]"]);
            }
            if cells.len() != columns.len() {
                return Err(format!("{}行目: 教材の列数が見出しと異なります（見出し{}列、現在{}列）。", index + 1, columns.len(), cells.len()));
            }
            for (position, column) in order.iter().enumerate() {
                if position > 0 { normalized.push('\t'); }
                normalized.push_str(cells[*column]);
            }
        }
        normalized.push('\n');
    }
    model::parse_deck(&normalized).map(VocabularyFile::Materials)
}

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
    fn vocabulary_file_routes_words_and_reordered_material_columns() {
        for source in ["happy\nready", "base\nexample", "Headword,Meaning\nhappy,うれしい", "Lemma\tMeaning\nhappy\tうれしい"] {
            assert!(matches!(parse_vocabulary_file(source).unwrap(), VocabularyFile::Words(_)));
        }
        let entry = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
        let original = model::deck_text(&[entry.clone()]);
        for count in [15, 17] {
            let reordered = original.lines().map(|line| {
                let mut cells: Vec<_> = line.split('\t').collect();
                if cells.len() == 15 && count == 17 { cells.extend(["[]", "[]"]); }
                cells.into_iter().take(count).rev().collect::<Vec<_>>().join("\t") })
                .collect::<Vec<_>>().join("\n");
            let VocabularyFile::Materials(items) = parse_vocabulary_file(&format!("\u{feff}# exported\n\n{reordered}\n")).unwrap() else { panic!("must be materials"); };
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, entry.id);
            assert_eq!(items[0].example, entry.example);
            assert_eq!(items[0].meaning, entry.meaning);
        }
    }

    #[test]
    fn malformed_materials_never_fall_back_to_ai_words() {
        let entry = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
        let valid = model::deck_text(&[entry]);
        for text in [
            valid.replacen("meaning", "unknown", 1),
            valid.replacen("meaning", "base", 1),
            valid.replacen("___", "blank", 1),
            valid.replacen("replacements\texamples", "replacements", 1),
            valid.replace('\t', ","),
            "id\tWord\na\thappy".into(),
            "base\tmeaning\nhappy\tうれしい".into(),
            valid.lines().skip(1).collect::<Vec<_>>().join("\n"),
        ] {
            assert!(parse_vocabulary_file(&text).is_err(), "must reject incomplete/invalid material");
        }
        assert!(parse_vocabulary_file(&(valid.clone() + valid.lines().nth(1).unwrap())).is_err(), "duplicate IDs");
    }

    #[test]
    fn material_file_keeps_larger_limit_than_word_lists() {
        let mut entry = model::parse_deck(model::BUILTIN_DECK).unwrap().remove(0);
        entry.business = "a".repeat(3900);
        let entries: Vec<_> = (0..600).map(|index| {
            let mut next = entry.clone(); next.id = format!("large_{index}"); next
        }).collect();
        let text = model::deck_text(&entries);
        assert!(text.len() > 2_000_000);
        assert!(matches!(parse_vocabulary_file(&text).unwrap(), VocabularyFile::Materials(items) if items.len() == 600));
        assert!(parse_vocabulary_file(&"happy\n".repeat(333_334)).is_err());
    }
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
