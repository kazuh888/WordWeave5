//! Offline integration checks use a mock executable, never an authenticated service.
use fs2::FileExt;
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use wordweave5::codex;

#[test]
fn structured_chat_action_reaches_transport_and_is_only_a_proposal() {
    let f = Fixture::new("structured_chat");
    let reply = codex::generate_with_settings(&f.exe, &f.dir, "", "test",
        vec![json!({"type":"text","text":"apologize for を新規登録して"})],
        Some(wordweave5::chat_action::schema()), Arc::new(AtomicBool::new(false))).unwrap();
    let parsed = wordweave5::chat_action::parse(&reply.text, reply.execution).unwrap();
    assert_eq!(parsed.title, "apologize for の登録");
    assert_eq!(parsed.execution.model.as_deref(), Some("test"));
    assert_eq!(parsed.execution.effort, None);
    let action = parsed.action.unwrap();
    assert_eq!(action.operation, wordweave5::chat_action::Operation::New);
    assert_eq!(action.base, "apologize for");
    assert!(!f.dir.join("progress.json").exists());
}

#[test]
fn generation_reports_server_settings_instead_of_requested_model() {
    let f = Fixture::new("reported_settings");
    let reply = codex::generate_with_settings(&f.exe, &f.dir, "requested-model", "test",
        vec![json!({"type":"text","text":"test"})], None, Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(reply.execution.model.as_deref(), Some("returned-model"));
    assert_eq!(reply.execution.effort.as_deref(), Some("high"));
}

#[test]
fn missing_and_null_server_effort_stay_unknown() {
    for mode in ["ok", "null_effort"] {
        let f = Fixture::new(mode);
        let reply = codex::generate_with_settings(&f.exe, &f.dir, "", "test",
            vec![json!({"type":"text","text":"test"})], None, Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(reply.execution.model.as_deref(), Some("test"));
        assert_eq!(reply.execution.effort, None);
    }
}

#[test]
fn chat_followup_transmits_saved_previous_exchange_after_reload() {
    let f = Fixture::new("chat");
    let mut conversation = wordweave5::chat::Conversation::new();
    conversation.draft = "apologize forの用法は？".into();
    for question in ["apologize forの用法は？", "社外メールでの別の例は？"] {
        conversation.draft = question.into();
        let context = wordweave5::chat::prepare(&conversation).unwrap();
        if !conversation.exchanges.is_empty() {
            assert_eq!(context.payload["conversation_history"][0]["user"], "apologize forの用法は？");
            assert_eq!(context.payload["conversation_history"][0]["assistant"], "{\"answer\":\"ok\"}");
        }
        fs::write(f.dir.join("expected-input.json"), serde_json::to_vec(&context.payload).unwrap()).unwrap();
        let reply = codex::generate_with_settings(&f.exe, &f.dir, "", "test",
            vec![json!({"type":"text","text":context.payload.to_string()})], None,
            Arc::new(AtomicBool::new(false))).unwrap();
        conversation.complete(question.into(), reply.text, reply.execution).unwrap();
        // Separate Codex processes and a disk-style round trip: continuity is
        // provided by WordWeave's saved transcript, not implicit model memory.
        conversation = serde_json::from_str(&serde_json::to_string(&conversation).unwrap()).unwrap();
    }
    assert_eq!(conversation.exchanges.len(), 2);
}

#[test]
fn unauthenticated_account_is_distinct_from_transport_exit() {
    let f = Fixture::new("noauth");
    let e = f.run(Arc::new(AtomicBool::new(false))).unwrap_err();
    assert!(e.contains("未ログイン"), "{e}");
    assert!(!f.requests().iter().any(|m| m == "thread/start"));
}

#[test]
fn early_exit_reports_stage_and_code_without_stderr_secrets() {
    let f = Fixture::new("early_exit");
    let e = f.run(Arc::new(AtomicBool::new(false))).unwrap_err();
    assert!(e.contains("initialize") && e.contains("7"), "{e}");
    assert!(!e.contains("SECRET"));
}

#[test]
fn malformed_stdout_is_distinct_from_authentication_failure() {
    let f = Fixture::new("bad_json");
    let e = f.run(Arc::new(AtomicBool::new(false))).unwrap_err();
    assert!(e.contains("JSON"), "{e}");
}

#[cfg(windows)]
#[test]
fn windows_volta_shim_preserves_piped_rpc_with_unicode_and_spaces() {
    let mut f = Fixture::new("ok");
    fs::copy(&f.exe, f.dir.join("volta.exe")).unwrap();
    let wrapper = f.dir.join("codex.cmd");
    fs::write(&wrapper, b"@echo off\r\nvolta run %~n0 %*\r\n").unwrap();
    f.exe = fs::canonicalize(wrapper).unwrap();
    let result = f.run(Arc::new(AtomicBool::new(false)));
    let stage = fs::read_to_string(f.dir.join("mock-stage.txt")).unwrap_or_else(|_| "volta.exe未到達（または段階記録失敗）".into());
    match result {
        Ok(text) => assert_eq!(text, "{\"answer\":\"ok\"}"),
        Err(error) => panic!("Windowsバッチ通信失敗 / mock stage: {stage}\n{error}"),
    }
}
// Run this one check in a separate process so its PATH cannot race with any
// other test. No set_var calls or requirement for --test-threads=1.
#[cfg(windows)]
#[test]
fn windows_volta_from_path_preserves_piped_rpc_with_separate_directories() {
    const CHILD_ROOT: &str = "WORDWEAVE_PATH_TEST_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        let root = PathBuf::from(root);
        let wrapper = fs::canonicalize(root.join("shims/codex.cmd")).unwrap();
        assert!(!wrapper.parent().unwrap().join("volta.exe").exists());
        let result = codex::generate(&wrapper, &root, "", "text only",
            vec![json!({"type":"text","text":"data"})], None,
            Arc::new(AtomicBool::new(false)));
        let stage = fs::read_to_string(root.join("tools 日本語/mock-stage.txt")).unwrap_or_default();
        assert_eq!(result.unwrap_or_else(|e| panic!("PATH経由起動失敗 / mock stage: {stage}\n{e}")),
            "{\"answer\":\"ok\"}");
        let requests = fs::read_to_string(root.join("requests.log")).unwrap();
        assert!(requests.lines().any(|s| s == "initialize"));
        assert!(requests.lines().any(|s| s == "turn/start"));
        let report = wordweave5::diagnostics::report();
        assert!(report.contains("launch: PATH volta.exe run codex app-server"), "{report}");
        assert!(!report.contains("launch: cmd.exe"), "{report}");
        return;
    }
    let f = Fixture::new("ok");
    let bin = f.dir.join("tools 日本語");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(f.dir.join("shims")).unwrap();
    fs::copy(&f.exe, bin.join("volta.exe")).unwrap();
    fs::write(bin.join("expected-cwd.txt"), f.dir.to_str().unwrap()).unwrap();
    // A real Volta wrapper; its sibling directory intentionally lacks volta.exe.
    fs::write(f.dir.join("shims/codex.cmd"), b"@echo off\r\nvolta run %~n0 %*\r\n").unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "windows_volta_from_path_preserves_piped_rpc_with_separate_directories", "--nocapture"])
        .env(CHILD_ROOT, &f.dir)
        .env("PATH", std::env::join_paths([bin]).unwrap())
        .output().unwrap();
    assert!(output.status.success(), "isolated PATH test failed: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

struct Fixture {
    dir: PathBuf,
    exe: PathBuf,
}
impl Fixture {
    fn new(mode: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "wordweave rpc 日本語-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(&dir).unwrap();
        let exe = PathBuf::from(env!("CARGO_BIN_EXE_wordweave-mock-codex"));
        assert!(exe.is_file(), "Cargo did not build the mock executable: {}", exe.display());
        fs::write(dir.join("mode.txt"), mode).unwrap();
        Self { dir, exe }
    }
    fn requests(&self) -> Vec<String> {
        fs::read_to_string(self.dir.join("requests.log")).unwrap()
            .lines().map(str::to_owned).collect()
    }
    fn run(&self, cancel: Arc<AtomicBool>) -> Result<String, String> {
        codex::generate(
            &self.exe,
            &self.dir,
            "",
            "text only",
            vec![json!({"type":"text","text":"data"})],
            None,
            cancel,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}
#[test]
fn fragmented_json_and_early_completion_are_handled() {
    assert_eq!(
        Fixture::new("ok")
            .run(Arc::new(AtomicBool::new(false)))
            .unwrap(),
        "{\"answer\":\"ok\"}"
    );
}
#[test]
fn paid_account_is_rejected_before_turn() {
    let f = Fixture::new("paid");
    assert!(f.run(Arc::new(AtomicBool::new(false)))
        .unwrap_err().contains("ChatGPT認証"));
    let requests = f.requests();
    assert!(!requests.iter().any(|m| m == "thread/start" || m == "turn/start"));
}
#[test]
fn partial_result_on_failed_turn_is_rejected() {
    let f = Fixture::new("failed");
    let error = f.run(Arc::new(AtomicBool::new(false))).unwrap_err();
    assert!(error.contains("Codex生成が完了しませんでした"), "{error}");
    assert!(f.requests().iter().any(|m| m == "turn/start"));
}
#[test]
fn cancellation_terminates_and_reaps_a_waiting_process() {
    let f = Fixture::new("cancel");
    let c = Arc::new(AtomicBool::new(false));
    let signal = c.clone();
    let ready = f.dir.join("waiting");
    let worker = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(15);
        while !ready.is_file() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let reached_turn = ready.is_file();
        let cancelled_at = Instant::now();
        signal.store(true, Ordering::Relaxed);
        (reached_turn, cancelled_at)
    });
    let result = f.run(c);
    let returned_at = Instant::now();
    let (reached_turn, cancelled_at) = worker.join().unwrap();
    assert!(reached_turn, "Mock did not reach turn/start within 15 seconds: {result:?}");
    assert!(result.unwrap_err().contains("中断"));
    assert!(returned_at.duration_since(cancelled_at) < Duration::from_secs(3));
    // The child holds this OS lock forever; reacquiring it proves termination.
    let lock = fs::OpenOptions::new().read(true).write(true)
        .open(f.dir.join("process.lock")).unwrap();
    lock.try_lock_exclusive().expect("Cancelled mock process is still alive");
}

#[test]
fn audio_is_rejected_before_turn_when_model_does_not_support_it() {
    let f = Fixture::new("ok");
    let result = codex::generate(
        &f.exe,
        &f.dir,
        "",
        "transcribe",
        vec![json!({"type":"audio","url":"data:audio/wav;base64,AAAA"})],
        None,
        Arc::new(AtomicBool::new(false)),
    );
    assert!(result.unwrap_err().contains("録音入力に対応していません"));
    let requests = f.requests();
    assert!(requests.iter().any(|m| m == "model/list"));
    assert!(!requests.iter().any(|m| m == "turn/start"));
}
#[test]
fn audio_can_reach_a_capable_mock_model() {
    let f = Fixture::new("audio");
    let result = codex::generate(
        &f.exe,
        &f.dir,
        "",
        "transcribe",
        vec![json!({"type":"audio","url":"data:audio/wav;base64,AAAA"})],
        None,
        Arc::new(AtomicBool::new(false)),
    );
    assert_eq!(result.unwrap(), "{\"answer\":\"ok\"}");
}
