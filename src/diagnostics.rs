//! Bounded, memory-only diagnostics. Never retain RPC bodies or raw stderr.
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone)]
pub struct Trace(Arc<Mutex<VecDeque<String>>>);
static LAST: OnceLock<Mutex<Option<Trace>>> = OnceLock::new();

impl Trace {
    pub fn begin() -> Self {
        let trace = Self(Arc::new(Mutex::new(VecDeque::new())));
        *LAST.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(trace.clone());
        trace.note(&format!("WordWeave {} / OS {} / {}", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH));
        trace
    }
    // Callers must only pass app-owned labels, counts or OS error codes.
    pub fn note(&self, text: &str) {
        let mut lines = self.0.lock().unwrap();
        if lines.len() == 128 { lines.pop_front(); }
        lines.push_back(format!("{} {}", chrono::Utc::now().format("%H:%M:%S%.3fZ"), text.chars().take(300).collect::<String>()));
    }
    pub fn stderr(&self, bytes: &[u8]) {
        self.note(&format!("stderr ({} bytes): {}", bytes.len(), classify(bytes)));
    }
    pub fn snapshot(&self) -> String {
        self.0.lock().unwrap().iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

// Conservative allow-list classification instead of trying to redact every
// possible secret. Unrecognized text is discarded, not copied into a report.
fn classify(bytes: &[u8]) -> &'static str {
    let text = String::from_utf8_lossy(bytes).to_lowercase();
    if text.contains("not logged in") { "未ログイン" }
    else if text.contains("invalid configuration") { "設定ファイルが不正" }
    else if text.contains("not recognized") || text.contains("認識されていません") { "コマンドが見つからない" }
    else if text.contains("cannot find") || text.contains("not found") || text.contains("見つかりません") { "ファイル/パス/対象が見つからない" }
    else if text.contains("access is denied") || text.contains("permission denied") { "アクセス拒否" }
    else if text.contains("syntax") || text.contains("構文") || text.contains("unexpected at this time") || text.contains("このとき予期しない") { "コマンド構文エラー" }
    else if text.contains("unc") { "UNCパス/作業ディレクトリ関連" }
    else if text.contains("certificate") || text.contains("tls") { "証明書/TLS関連" }
    else if text.contains("unauthorized") || text.contains("401") { "認証拒否" }
    else if text.contains("forbidden") || text.contains("403") { "アクセス禁止" }
    else if text.contains("deserialize") || text.contains("invalid json") { "JSON通信の解析失敗" }
    else if text.contains("volta") { "Volta関連メッセージ（本文非保存）" }
    else if text.contains("panic") { "子プロセスのpanic" }
    else { "未分類（機密情報保護のため本文非保存）" }
}

pub fn report() -> String {
    let last = LAST.get_or_init(|| Mutex::new(None)).lock().unwrap().clone();
    let body = last.map(|t| {
        let lines = t.0.lock().unwrap();
        lines.iter().cloned().collect::<Vec<_>>().join("\n")
    })
        .unwrap_or_else(|| "診断情報なし。「接続・ChatGPT認証を確認」を実行してください。".into());
    format!("WordWeaveのCodex接続障害を分析してください。事実と仮説を分け、次の安全な確認手順を提案してください。\n\n注意：これは本文を含まない診断要約です。stderrは分類のみで、原文・認証情報・教材・ユーザーパスは保存していません。未分類の原因はこの情報だけでは特定できません。\n\n{body}\n\n追加情報（必要なら手入力）：codex --version / codex login status の結果。認証URL、ワンタイムコード、auth.jsonは貼らないでください。")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stderr_never_returns_secrets() {
        for input in ["access_token=SECRET", "https://auth.openai.com/?code=SECRET", "volta: SECRET", "401 Bearer SECRET", "日本語SECRET", "C:\\Users\\SECRET", "not logged in SECRET"] {
            assert!(!classify(input.as_bytes()).contains("SECRET"));
        }
    }
    #[test]
    fn ring_is_bounded() {
        let t = Trace(Arc::new(Mutex::new(VecDeque::new())));
        for _ in 0..1000 { t.note("event"); }
        assert_eq!(t.0.lock().unwrap().len(), 128);
    }
    #[test]
    fn captured_stderr_has_categories_but_no_raw_text() {
        let t = Trace(Arc::new(Mutex::new(VecDeque::new())));
        t.stderr(b"volta: not found SECRET");
        let lines = t.0.lock().unwrap();
        assert!(lines[0].contains("見つからない"));
        assert!(!lines[0].contains("SECRET"));
    }
}
