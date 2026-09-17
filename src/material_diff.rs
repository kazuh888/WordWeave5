//! Bounded, Unicode-safe display differences; never used to decide registration rules.
use crate::model::{normalize, Entry};
use serde_json::Value;

pub struct Row {
    pub path: String,
    pub label: String,
    pub before: String,
    pub after: String,
    pub kind: &'static str,
}
fn text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        _ => serde_json::to_string_pretty(v).unwrap_or_default(),
    }
}
pub fn rows(old: Option<&Entry>, new: &Entry) -> Vec<Row> {
    let old = old
        .map(|e| serde_json::to_value(e).unwrap())
        .unwrap_or(Value::Null);
    let new = serde_json::to_value(new).unwrap();
    let mut rows = Vec::new();
    for (key, label) in [
        ("base", "基本語"),
        ("meaning", "意味"),
        ("level", "分類"),
        ("business", "社外メール"),
        ("elevated", "文体"),
        ("register", "語調"),
        ("usage", "説明・使い方"),
        ("context", "場面"),
        ("example", "空欄問題"),
        ("translation", "訳"),
        ("answers", "正解"),
        ("question", "用法の質問"),
        ("explanation", "解説"),
        ("tag", "タグ"),
    ] {
        if old[key] != new[key] {
            rows.push(Row {
                path: format!("/{key}"),
                label: label.into(),
                before: text(&old[key]),
                after: text(&new[key]),
                kind: if old.is_null() { "追加" } else { "変更" },
            });
        }
    }
    for (key, identity, label) in [
        ("examples", "english", "例文"),
        ("replacements", "phrase", "言い換え"),
    ] {
        let empty = Vec::new();
        let before = old[key].as_array().unwrap_or(&empty);
        let after = new[key].as_array().unwrap_or(&empty);
        let mut used = vec![false; before.len()];
        for (i, a) in after.iter().enumerate() {
            let matched = before.iter().enumerate().find(|(j, b)| {
                !used[*j]
                    && normalize(b[identity].as_str().unwrap_or_default())
                        == normalize(a[identity].as_str().unwrap_or_default())
            });
            if let Some((j, b)) = matched {
                used[j] = true;
                if b == a && i == j {
                    continue;
                }
                rows.push(Row {
                    path: format!("/{key}/{i}"),
                    label: format!("{label} {} → {}", j + 1, i + 1),
                    before: text(b),
                    after: text(a),
                    kind: if b == a { "並べ替え" } else { "変更" },
                });
            } else {
                rows.push(Row {
                    path: format!("/{key}/{i}"),
                    label: format!("{label} {}", i + 1),
                    before: String::new(),
                    after: text(a),
                    kind: "追加",
                });
            }
        }
        for (j, b) in before.iter().enumerate().filter(|(j, _)| !used[*j]) {
            rows.push(Row {
                path: format!("/removed/{key}/{j}"),
                label: format!("{label} {}", j + 1),
                before: text(b),
                after: String::new(),
                kind: "削除",
            });
        }
    }
    rows
}
/// (text, changed) runs for each side. Oversized inputs use prefix/suffix matching.
pub fn spans(before: &str, after: &str) -> (Vec<(String, bool)>, Vec<(String, bool)>) {
    let a: Vec<char> = before.chars().collect();
    let b: Vec<char> = after.chars().collect();
    let mut aa = vec![true; a.len()];
    let mut bb = vec![true; b.len()];
    if a.len().saturating_mul(b.len()) <= 250_000 && a.len() + b.len() <= 4000 {
        let w = b.len() + 1;
        let mut table = vec![0u16; (a.len() + 1) * w];
        for i in (0..a.len()).rev() {
            for j in (0..b.len()).rev() {
                table[i * w + j] = if a[i] == b[j] {
                    table[(i + 1) * w + j + 1] + 1
                } else {
                    table[(i + 1) * w + j].max(table[i * w + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < a.len() && j < b.len() {
            if a[i] == b[j] {
                aa[i] = false;
                bb[j] = false;
                i += 1;
                j += 1;
            } else if table[(i + 1) * w + j] >= table[i * w + j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    } else {
        let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
        let suffix = a[prefix..]
            .iter()
            .rev()
            .zip(b[prefix..].iter().rev())
            .take_while(|(x, y)| x == y)
            .count();
        for i in 0..prefix {
            aa[i] = false;
            bb[i] = false;
        }
        for i in 0..suffix {
            aa[a.len() - 1 - i] = false;
            bb[b.len() - 1 - i] = false;
        }
    }
    fn groups(chars: Vec<char>, flags: Vec<bool>) -> Vec<(String, bool)> {
        let mut out: Vec<(String, bool)> = Vec::new();
        for (c, changed) in chars.into_iter().zip(flags) {
            if out.last().is_none_or(|(_, b)| *b != changed) {
                out.push((String::new(), changed));
            }
            out.last_mut().unwrap().0.push(c);
        }
        out
    }
    (groups(a, aa), groups(b, bb))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_differences_preserve_both_originals_and_multiple_changes() {
        let (a, b) = spans("猫と犬🐕を学ぶ", "鳥と犬🐕を習う");
        assert_eq!(
            a.iter().map(|x| x.0.as_str()).collect::<String>(),
            "猫と犬🐕を学ぶ"
        );
        assert_eq!(
            b.iter().map(|x| x.0.as_str()).collect::<String>(),
            "鳥と犬🐕を習う"
        );
        assert!(a.iter().any(|(s, c)| !c && s.contains("と犬🐕を")));
        assert_eq!(
            spans(&"a".repeat(10000), &"a".repeat(10000)).0,
            vec![("a".repeat(10000), false)]
        );
    }
    #[test]
    fn array_reordering_is_not_reported_as_deleted_content() {
        let mut e = crate::model::parse_deck(crate::model::BUILTIN_DECK)
            .unwrap()
            .remove(0);
        e.examples = vec![
            crate::model::Example {
                english: "A".into(),
                japanese: "a".into(),
                note: "a".into(),
            },
            crate::model::Example {
                english: "B".into(),
                japanese: "b".into(),
                note: "b".into(),
            },
        ];
        let mut n = e.clone();
        n.examples.reverse();
        let rows = rows(Some(&e), &n);
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.kind == "並べ替え"));
    }
}
