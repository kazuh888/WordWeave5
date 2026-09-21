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
fn codex_diagnostics_correlate_phases_without_retaining_content_or_paths() {
    const CHILD: &str = "WORDWEAVE_DIAGNOSTICS_TEST_CHILD";
    // The sink is process-global. Parallel runs can connect before it is
    // initialized and leave partial runs in the export, so isolate this test.
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "codex_diagnostics_correlate_phases_without_retaining_content_or_paths", "--nocapture"])
            .env(CHILD, "1")
            .output().unwrap();
        assert!(output.status.success(), "isolated diagnostics test failed: {:?}\nstdout: {}\nstderr: {}",
            output.status.code(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        return;
    }
    use wordweave5::diagnostics;
    let f = Fixture::new("effort");
    diagnostics::initialize(&f.dir.join("logs"));
    codex::generate_with_effort(&f.exe, &f.dir, "test", "high", "SECRET instructions",
        vec![json!({"type":"text","text":"SECRET prompt"})], None,
        Arc::new(AtomicBool::new(false))).unwrap();
    let denied = Fixture::new("paid");
    assert!(codex::check(&denied.exe, &denied.dir, Arc::new(AtomicBool::new(false))).is_err());
    let report = diagnostics::export().unwrap();
    assert!(!report.contains("SECRET") && !report.contains(f.dir.to_str().unwrap()));
    let events: Vec<serde_json::Value> = report.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    // Select the completed generation that queried capabilities.
    let finished = events.iter().find(|event| event["stage"] == "generate" && event["event"] == "completed"
        && events.iter().any(|phase| phase["run_id"] == event["run_id"]
            && phase["stage"] == "model_list" && phase["event"] == "completed"))
        .expect("completed generation must be recorded");
    for stage in ["connect", "initialize", "authenticate", "model_list", "thread_start", "turn_start"] {
        assert!(events.iter().any(|event| event["run_id"] == finished["run_id"]
            && event["entry_point"] == "codex" && event["stage"] == stage
            && event["event"] == "completed"), "missing correlated {stage}");
    }
    assert!(events.iter().any(|event| event["stage"] == "authenticate" && event["error"] == "authentication"));
    // Disable this test-process-only sink before its temporary directory is removed.
    diagnostics::initialize(&f.dir.join("mode.txt"));
}

#[test]
fn effort_catalog_preserves_server_choices_and_missing_values_across_pages() {
    let f = Fixture::new("effort");
    let models = codex::list_model_efforts(&f.exe, &f.dir, Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(models.len(), 3);
    assert_eq!(models[1].model, "test");
    assert_eq!(models[1].display_name, "Test model");
    assert_eq!(models[1].supported_efforts.as_ref().unwrap()[1].effort, "future-effort");
    assert_eq!(models[1].default_effort.as_deref(), Some("high"));
    assert_eq!(models[2].supported_efforts, None);
    assert_eq!(models[2].default_effort, None);
    assert!(!f.requests().iter().any(|m| m == "thread/start" || m == "turn/start"));
}

#[test]
fn effort_is_sent_as_thread_override_but_actual_execution_uses_server_value() {
    let f = Fixture::new("effort");
    let reply = f.run_effort("test", "high").unwrap();
    assert_eq!(reply.execution.effort.as_deref(), Some("low"));
    let params: serde_json::Value = serde_json::from_slice(&fs::read(f.dir.join("thread-start.json")).unwrap()).unwrap();
    assert_eq!(params["config"]["model_reasoning_effort"], "high");
    assert!(!f.dir.join("config.toml").exists());
}

#[test]
fn effort_default_omits_override_and_does_not_depend_on_catalog() {
    let f = Fixture::new("effort_unavailable");
    f.run_effort("", "").unwrap();
    let params: serde_json::Value = serde_json::from_slice(&fs::read(f.dir.join("thread-start.json")).unwrap()).unwrap();
    assert!(params["config"].get("model_reasoning_effort").is_none());
    assert!(!f.requests().iter().any(|m| m == "model/list"));
}

#[test]
fn effort_not_supported_by_requested_model_stops_before_thread_or_turn() {
    let f = Fixture::new("effort");
    assert!(f.run_effort("other", "high").is_err());
    assert!(!f.requests().iter().any(|m| m == "thread/start" || m == "turn/start"));
}

#[test]
fn effort_rechecks_actual_model_after_server_rerouting() {
    let f = Fixture::new("effort_rerouted");
    assert!(f.run_effort("test", "high").is_err());
    assert!(f.requests().iter().any(|m| m == "thread/start"));
    assert!(!f.requests().iter().any(|m| m == "turn/start"));
}

#[test]
fn effort_missing_capabilities_or_failed_lookup_stops_before_generation() {
    for mode in ["effort_missing", "effort_unavailable", "effort_cycle"] {
        let f = Fixture::new(mode);
        assert!(f.run_effort("test", "high").is_err(), "{mode}");
        assert!(!f.requests().iter().any(|m| m == "turn/start"), "{mode}");
    }
}

#[test]
fn effort_default_model_uses_returned_model_not_catalog_default_and_keeps_unknown_execution() {
    let f = Fixture::new("effort_unknown_execution");
    let reply = f.run_effort("", "high").unwrap();
    assert_eq!(reply.execution.model.as_deref(), Some("test"));
    assert_eq!(reply.execution.effort, None);
}

#[test]
fn effort_catalog_requires_chatgpt_authentication() {
    let f = Fixture::new("paid");
    assert!(codex::list_model_efforts(&f.exe, &f.dir, Arc::new(AtomicBool::new(false))).is_err());
}

#[test]
fn durable_result_and_terminal_states_are_distinct() {
    use wordweave5::run_journal::{self,Outcome};
    for (mode,expected) in [("ok",Outcome::Completed),("failed",Outcome::Failed),("interrupted",Outcome::Interrupted),("lost_result",Outcome::Unknown)] {
        let f=Fixture::new(mode);
        let result=f.run(Arc::new(AtomicBool::new(false)));
        let records=run_journal::list(&f.dir).unwrap();
        assert_eq!(records.len(),1);
        assert_eq!(records[0].outcome,expected);
        assert_eq!(records[0].turn_id.as_deref(),Some("u"));
        assert_eq!(records[0].response.as_deref(),result.as_ref().ok().map(String::as_str));
    }
}
#[test]
fn unknown_result_can_be_read_without_a_second_generation() {
    let f=Fixture::new("lost_result");
    assert!(f.run(Arc::new(AtomicBool::new(false))).is_err());
    let id=wordweave5::run_journal::list(&f.dir).unwrap()[0].id.clone();
    let result=codex::recover(&f.exe,&f.dir,&id,Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(result.response.as_deref(),Some("recovered"));
    assert_eq!(f.requests().iter().filter(|m|*m=="turn/start").count(),1);
    assert_eq!(result.execution.effort,None);
}
#[test]
fn recovery_does_not_substitute_a_different_turn() {
    let f=Fixture::new("lost_result");
    assert!(f.run(Arc::new(AtomicBool::new(false))).is_err());
    let id=wordweave5::run_journal::list(&f.dir).unwrap()[0].id.clone();
    fs::write(f.dir.join("mode.txt"),"wrong_turn").unwrap();
    assert!(codex::recover(&f.exe,&f.dir,&id,Arc::new(AtomicBool::new(false))).is_err());
    assert_eq!(f.requests().iter().filter(|m|*m=="turn/start").count(),1);
}

#[test]
fn missing_turn_id_stays_unknown_without_guessing_the_last_turn() {
    let f=Fixture::new("lost_id");
    assert!(f.run(Arc::new(AtomicBool::new(false))).is_err());
    let record=wordweave5::run_journal::list(&f.dir).unwrap().remove(0);
    assert_eq!(record.outcome,wordweave5::run_journal::Outcome::Unknown);
    assert!(record.turn_id.is_none());
    assert!(codex::recover(&f.exe,&f.dir,&record.id,Arc::new(AtomicBool::new(false))).is_err());
    let requests=f.requests();
    assert_eq!(requests.iter().filter(|m|*m=="turn/start").count(),1);
    assert!(!requests.iter().any(|m|m=="thread/read"));
}

#[test]
fn recovery_accepts_original_image_history_larger_than_generation_line_limit() {
    let f=Fixture::new("large_lost_result");
    let image=format!("data:image/png;base64,{}","A".repeat(5*1024*1024));
    assert!(codex::generate(&f.exe,&f.dir,"","read image",vec![json!({"type":"image","url":image})],None,Arc::new(AtomicBool::new(false))).is_err());
    let record=wordweave5::run_journal::list(&f.dir).unwrap().remove(0);
    let recovered=codex::recover(&f.exe,&f.dir,&record.id,Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(recovered.response.as_deref(),Some("large recovered"));
    assert_eq!(f.requests().iter().filter(|m|*m=="turn/start").count(),1);
}

#[test]
fn image_input_requires_explicit_model_capability() {
    for (mode, supported) in [("ok",false),("image",true)] {
        let f=Fixture::new(mode);
        let result=codex::generate(&f.exe,&f.dir,"","read image",
            vec![json!({"type":"image","url":"data:image/png;base64,AAAA"})],None,
            Arc::new(AtomicBool::new(false)));
        assert_eq!(result.is_ok(),supported,"{mode}: {result:?}");
        assert_eq!(f.requests().iter().any(|m|m=="turn/start"),supported);
    }
}

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
    fn run_effort(&self, model: &str, effort: &str) -> Result<codex::Generated, String> {
        codex::generate_with_effort(&self.exe, &self.dir, model, effort, "test",
            vec![json!({"type":"text","text":"fixture"})], None, Arc::new(AtomicBool::new(false)))
    }
    fn new(mode: &str) -> Self {
        Self::with_stamp(mode, chrono::Utc::now().timestamp_nanos_opt().unwrap())
    }
    fn with_stamp(mode:&str,stamp:i64)->Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "wordweave rpc 日本語-{}-{}-{}",
            std::process::id(),
            stamp,
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
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
#[test]
fn fixtures_remain_isolated_when_clock_values_are_identical() {
    let a=Fixture::with_stamp("audio",123);
    let b=Fixture::with_stamp("cancel",123);
    assert_ne!(a.dir,b.dir);
    assert_eq!(fs::read_to_string(a.dir.join("mode.txt")).unwrap(),"audio");
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
