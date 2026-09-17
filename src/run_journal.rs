//! Durable evidence of a generation. Local cancellation is not proof of server cancellation.
use crate::{execution::Execution, store::atomic_write};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    Prepared,
    Submitted,
    Completed,
    Failed,
    Interrupted,
    Unknown,
}
impl Outcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Prepared => "送信準備",
            Self::Submitted => "結果待ち",
            Self::Completed => "生成完了（教材登録とは別）",
            Self::Failed => "失敗",
            Self::Interrupted => "Codexの中断を確認",
            Self::Unknown => "結果不明（自動再生成しない）",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub at: i64,
    pub outcome: Outcome,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub execution: Execution,
    pub request: Value,
    pub response: Option<String>,
    pub note: String,
}
pub fn directory(cwd: &Path) -> PathBuf {
    cwd.join("wordweave-runs")
}
fn path(cwd: &Path, id: &str) -> Result<PathBuf, String> {
    if id.is_empty() || id.len() > 100 || !id.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return Err("実行IDが不正です。".into());
    }
    Ok(directory(cwd).join(format!("{id}.json")))
}
impl RunRecord {
    pub fn begin(
        cwd: &Path,
        instructions: &str,
        input: &[Value],
        schema: &Option<Value>,
    ) -> Result<Self, String> {
        // Binary originals live in assets. Do not duplicate data URLs or ephemeral file paths.
        let inputs: Vec<_> = input
            .iter()
            .map(|v| {
                if v["type"] == "text" {
                    v.clone()
                } else {
                    serde_json::json!({"type":v["type"],"media_bytes_omitted":true})
                }
            })
            .collect();
        let record = Self {
            id: format!(
                "{}-{}",
                std::process::id(),
                chrono::Utc::now().timestamp_nanos_opt().unwrap()
            ),
            at: chrono::Utc::now().timestamp(),
            outcome: Outcome::Prepared,
            thread_id: None,
            turn_id: None,
            execution: Execution::default(),
            request: serde_json::json!({"instructions":instructions,"input":inputs,"schema":schema}),
            response: None,
            note: String::new(),
        };
        record.save(cwd)?;
        Ok(record)
    }
    pub fn save(&self, cwd: &Path) -> Result<(), String> {
        fs::create_dir_all(directory(cwd)).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() > 8_000_000 {
            return Err("実行記録が保存上限を超えています。".into());
        }
        atomic_write(&path(cwd, &self.id)?, &bytes)
    }
    pub fn observe_terminal(&mut self, v: &Value) {
        if v["method"] == "turn/completed"
            && v["params"]["threadId"].as_str() == self.thread_id.as_deref()
            && v["params"]["turn"]["id"].as_str() == self.turn_id.as_deref()
        {
            match v["params"]["turn"]["status"].as_str() {
                Some("failed") => self.outcome = Outcome::Failed,
                Some("interrupted") => self.outcome = Outcome::Interrupted,
                _ => {}
            }
        }
    }
}
pub fn load(cwd: &Path, id: &str) -> Result<RunRecord, String> {
    let path = path(cwd, id)?;
    if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 8_000_000 {
        return Err("実行記録が大きすぎます。".into());
    }
    let mut r: RunRecord = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if r.id != id {
        return Err("実行記録のIDが一致しません。".into());
    }
    if matches!(r.outcome, Outcome::Prepared | Outcome::Submitted) {
        r.outcome = Outcome::Unknown;
        r.note = "前回の処理の終了を確認できない。".into();
    }
    Ok(r)
}
pub struct Listing {
    pub records: Vec<RunRecord>,
    pub warnings: Vec<String>,
}
pub fn list(cwd: &Path) -> Result<Vec<RunRecord>, String> {
    Ok(scan(cwd)?.records)
}
pub fn scan(cwd: &Path) -> Result<Listing, String> {
    if !directory(cwd).exists() {
        return Ok(Listing {
            records: Vec::new(),
            warnings: Vec::new(),
        });
    }
    let entries = fs::read_dir(directory(cwd))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut paths = entries
        .into_iter()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect::<Vec<_>>();
    paths.sort_by_key(|p| {
        std::cmp::Reverse(
            p.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.rsplit('-').next())
                .unwrap_or("")
                .to_owned(),
        )
    });
    let mut out = Vec::new();
    let mut warnings = Vec::new();
    for p in paths.into_iter().take(200) {
        if let Some(id) = p.file_stem().and_then(|s| s.to_str()) {
            match load(cwd, id) {
                Ok(record) => out.push(record),
                Err(e) => warnings.push(format!("実行記録 {id} を読めません（原本は保持）：{e}")),
            }
        }
    }
    out.sort_by_key(|r| std::cmp::Reverse(r.at));
    Ok(Listing {
        records: out,
        warnings,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_record_does_not_hide_other_recoverable_results() {
        let root = std::env::temp_dir().join(format!(
            "ww-run-corrupt-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let mut a = RunRecord::begin(&root, "rules", &[], &None).unwrap();
        a.id = "1-100".into();
        a.save(&root).unwrap();
        std::fs::write(directory(&root).join("2-200.json"), b"broken").unwrap();
        let results = scan(&root).unwrap();
        assert_eq!(results.records.len(), 2);
        assert_eq!(results.warnings.len(), 1);
        assert_eq!(
            std::fs::read(directory(&root).join("2-200.json")).unwrap(),
            b"broken"
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn stopped_run_is_unknown_and_media_bytes_are_not_duplicated() {
        let root = std::env::temp_dir().join(format!(
            "ww-run-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let mut r = RunRecord::begin(
            &root,
            "rules",
            &[serde_json::json!({"type":"audio","url":"secret-data"})],
            &None,
        )
        .unwrap();
        assert!(!r.request.to_string().contains("secret-data"));
        r.outcome = Outcome::Submitted;
        r.save(&root).unwrap();
        assert_eq!(load(&root, &r.id).unwrap().outcome, Outcome::Unknown);
        r.outcome = Outcome::Completed;
        r.response = Some("done".into());
        r.save(&root).unwrap();
        assert_eq!(
            load(&root, &r.id).unwrap().response.as_deref(),
            Some("done")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
