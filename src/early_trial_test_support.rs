use std::{
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// One exact case per child: isolates the trial registry and poison state.
pub(crate) fn isolated(exact: &str, body: impl FnOnce(&Path)) {
    const CHILD: &str = "WW5_ET2_CHILD";
    const BASE: &str = "WW5_ET2_BASE";
    if std::env::var(CHILD).as_deref() == Ok(exact) {
        #[link(name = "shell32")]
        extern "system" {
            fn IsUserAnAdmin() -> i32;
        }
        assert_eq!(
            unsafe { IsUserAnAdmin() },
            0,
            "FIXTURE_FAILURE: Administrator must be False"
        );
        let base = PathBuf::from(std::env::var_os(BASE).expect("isolated child base"));
        assert!(base.is_absolute());
        assert_eq!(
            std::env::var_os("LOCALAPPDATA").unwrap(),
            base.join("local")
        );
        body(&base);
        println!("ET2_CASE_COMPLETE:{exact}:Administrator=False");
        return;
    }
    let base = std::env::temp_dir().join(format!(
        "ww5-et2-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ));
    fs::create_dir(&base).unwrap();
    for name in ["local", "temp", "cwd"] {
        fs::create_dir(base.join(name)).unwrap();
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([exact, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, exact)
        .env(BASE, &base)
        .env("LOCALAPPDATA", base.join("local"))
        .env("TEMP", base.join("temp"))
        .env("TMP", base.join("temp"))
        .current_dir(base.join("cwd"))
        .creation_flags(0x08000000)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut timed_out = false;
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!(
        "ET2_CHILD:{exact}:{}:timeout={timed_out}\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    // Only this uniquely created synthetic tree is removed, after child reap.
    fs::remove_dir_all(&base).expect("FIXTURE_FAILURE: synthetic fixture cleanup");
    assert!(!base.exists(), "FIXTURE_FAILURE: fixture remains");
    assert!(
        !timed_out,
        "FIXTURE_FAILURE: child timed out and was reaped"
    );
    assert!(output.status.success(), "child case failed");
    assert!(stdout.contains("running 1 test") && stdout.contains("1 passed; 0 failed"));
    assert!(stdout.contains(&format!("ET2_CASE_COMPLETE:{exact}:Administrator=False")));
}
