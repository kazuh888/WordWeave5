use std::{fs, time::{Duration, Instant}};
use wordweave5::diagnostics::{self, EntryPoint, ErrorClass, Event, Operation, Stage};

// A separate test process owns the global sink. It never initializes the live data path.
#[test]
fn diagnostic_api_keeps_operation_identity_and_reports_failures_without_panicking() {
    let root = std::env::temp_dir().join(format!("ww-diagnostic-api-{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
    fs::create_dir(&root).unwrap();
    let logs = root.join("logs");
    diagnostics::initialize(&logs);
    assert!(diagnostics::status().contains("記録はまだない"));
    assert!(diagnostics::export().unwrap().is_empty());

    let op = Operation::begin(EntryPoint::Recording);
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(2) { std::thread::yield_now(); }
    op.event(Stage::Record, Event::Paused);
    op.event(Stage::Record, Event::Resumed);
    op.fail(Stage::Save, ErrorClass::Io);
    let text = diagnostics::export().unwrap();
    let events: Vec<serde_json::Value> = text.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(events.len(), 4);
    assert!(events.iter().all(|event| event["run_id"] == events[0]["run_id"]));
    assert!(events.iter().all(|event| event["entry_point"] == "recording"));
    assert_eq!(events[0]["event"], "started");
    assert_eq!(events[3]["error"], "io");
    assert!(events[3]["elapsed_ms"].as_u64().unwrap() >= 2);
    assert!(!text.contains(root.to_str().unwrap()));

    diagnostics::initialize(&logs);
    assert_eq!(diagnostics::export().unwrap(), text);
    // A failed reinitialization disables this diagnostic sink, not the user's operation.
    let blocked = root.join("not-a-directory");
    fs::write(&blocked, "SECRET").unwrap();
    diagnostics::initialize(&blocked);
    let op = Operation::begin(EntryPoint::Storage);
    op.event(Stage::Save, Event::Completed);
    assert!(diagnostics::export().is_err());
    let status = diagnostics::status();
    assert!(!status.contains("SECRET") && !status.contains(root.to_str().unwrap()));
    fs::remove_dir_all(root).unwrap();
}
