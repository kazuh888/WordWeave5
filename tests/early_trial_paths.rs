//! ET1 specification-owned integration tests. All data are synthetic.
//! A fixture/compile failure is not a behavioral RED or acceptance PASS.
#![cfg(windows)]

use std::{
    collections::BTreeMap,
    ffi::c_void,
    fs::{self, OpenOptions},
    os::windows::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wordweave5::store::{
    create_external_directory, resolve_storage_root, write_external_file, Storage,
};

const CHILD: &str = "WW5_ET1_CHILD";
const BASE: &str = "WW5_ET1_BASE";
const V2: &str = r#"{"version":2,"audio_history":null,"settings":{"topic":"synthetic old"},"memories":{},"reviews":[],"suspended":[],"ai_calls":{},"study_seconds":{}}"#;
const PAYLOAD: &[u8] = b"synthetic external payload";
type Ledger = BTreeMap<PathBuf, (u32, Vec<u8>)>;

fn ledger(root: &Path) -> Ledger {
    fn visit(base: &Path, dir: &Path, out: &mut Ledger) {
        for entry in fs::read_dir(dir).expect("FIXTURE_FAILURE: ledger directory") {
            let p = entry.unwrap().path();
            let m = fs::symlink_metadata(&p).unwrap();
            let attrs = m.file_attributes() & 0x410;
            let bytes = if attrs & 0x400 != 0 {
                b"reparse: not traversed".to_vec()
            } else if p.file_name().unwrap() == "app.lock" {
                // A live fs2 byte-range lock forbids reading even in this process.
                // Observe its length; every data file still uses exact bytes.
                m.len().to_le_bytes().to_vec()
            } else if m.is_file() {
                fs::read(&p).unwrap()
            } else {
                vec![]
            };
            out.insert(p.strip_prefix(base).unwrap().to_owned(), (attrs, bytes));
            if m.is_dir() && attrs & 0x400 == 0 {
                visit(base, &p, out);
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

fn remove_fixture(path: &Path) {
    let m = fs::symlink_metadata(path).unwrap();
    if m.file_attributes() & 0x400 != 0 {
        // Junction metadata is a reparse type; is_dir() alone is not its
        // Windows directory attribute. Remove the link itself, never its target.
        if m.file_attributes() & 0x10 != 0 {
            fs::remove_dir(path).expect("FIXTURE_FAILURE: remove junction itself");
        } else {
            fs::remove_file(path).expect("FIXTURE_FAILURE: remove file reparse point");
        }
        return;
    }
    if m.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            remove_fixture(&entry.unwrap().path());
        }
        fs::remove_dir(path).unwrap();
    } else {
        fs::remove_file(path).unwrap();
    }
}

struct Fixture {
    base: PathBuf,
    normal: PathBuf,
    trial: PathBuf,
    outside: PathBuf,
}
impl Fixture {
    fn child() -> Self {
        let base = PathBuf::from(std::env::var_os(BASE).expect("isolated child only"));
        assert!(base.is_absolute());
        let local = base.join("local");
        let normal = local.join("WordWeave5");
        let trial = local.join("WordWeave5-EarlyTrial");
        let outside = base.join("outside");
        fs::create_dir_all(&normal).unwrap();
        fs::create_dir(&outside).unwrap();
        for name in [
            "progress.json",
            "custom.tsv",
            "assets/audio.wav",
            "ink/ink.json",
            "codex-work/journal.json",
            "diagnostics/session.txt",
            "backups/old.json",
            "pending-material-commit.json",
        ] {
            let p = normal.join(name);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, format!("normal sentinel: {name}")).unwrap();
        }
        Self {
            base,
            normal,
            trial,
            outside,
        }
    }
    fn seed_trial(&self) {
        fs::create_dir(&self.trial).unwrap();
        fs::write(self.trial.join("progress.json"), V2).unwrap();
        fs::write(self.trial.join("custom.tsv"), "synthetic old deck").unwrap();
    }
    fn refuse_before_writes(&self) {
        let before = ledger(&self.base);
        let error = match Storage::open() {
            Ok(_) => panic!("BEHAVIOR_RED: unsafe trial opened"),
            Err(e) => e,
        };
        assert!(
            error.contains("起動前の検査で停止しました。元データは変更していません。"),
            "{error}"
        );
        assert_eq!(
            ledger(&self.base),
            before,
            "preflight must not create lock/probe/recovery/diagnostics"
        );
    }
}

fn isolated(name: &str, body: impl FnOnce(Fixture)) {
    if std::env::var(CHILD).as_deref() == Ok(name) {
        body(Fixture::child());
        println!("ET1_CASE_COMPLETE:{name}");
        return;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("ww5-et1-{}-{stamp}", std::process::id()));
    fs::create_dir(&base).unwrap();
    for name in ["local", "temp", "cwd"] {
        fs::create_dir(base.join(name)).unwrap();
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([name, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, name)
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
    let deadline = Instant::now() + Duration::from_secs(30);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!(
        "ET1_CHILD:{name}:{}:timeout={timed_out}\n{stdout}\n{stderr}",
        output.status
    );
    remove_fixture(&base);
    assert!(
        !timed_out,
        "FIXTURE_FAILURE: child killed/reaped at deadline"
    );
    assert!(
        output.status.success(),
        "inspect child failure: behavior versus fixture"
    );
    assert!(
        stdout.contains("running 1 test") && stdout.contains(&format!("ET1_CASE_COMPLETE:{name}"))
    );
}

#[test]
fn et01_resolver_is_pure_and_selects_build_namespace() {
    isolated("et01_resolver_is_pure_and_selects_build_namespace", |f| {
        let before = ledger(&f.base);
        let expected = if cfg!(feature = "early-trial") {
            &f.trial
        } else {
            &f.normal
        };
        assert_eq!(&resolve_storage_root().unwrap(), expected);
        std::env::set_current_dir(&f.outside).unwrap();
        assert_eq!(&resolve_storage_root().unwrap(), expected);
        assert_eq!(ledger(&f.base), before);
    });
}

#[test]
fn et05_unregistered_generic_and_storage_at_keep_caller_contract() {
    isolated(
        "et05_unregistered_generic_and_storage_at_keep_caller_contract",
        |f| {
            std::env::remove_var("LOCALAPPDATA");
            let storage = Storage::at(f.base.join("caller")).unwrap();
            let normal_before = ledger(&f.normal);
            let dest = f.normal.join("explicit-external.txt");
            write_external_file(&storage.dir, &dest, PAYLOAD).unwrap();
            assert_eq!(fs::read(&dest).unwrap(), PAYLOAD);
            fs::remove_file(dest).unwrap();
            assert_eq!(ledger(&f.normal), normal_before);
            let before = ledger(&f.base);
            assert!(
                write_external_file(&storage.dir, &storage.dir.join("blocked.txt"), PAYLOAD)
                    .is_err()
            );
            assert_eq!(ledger(&f.base), before);
        },
    );
}

#[cfg(not(feature = "early-trial"))]
#[test]
fn et01_default_open_still_uses_normal_namespace() {
    isolated("et01_default_open_still_uses_normal_namespace", |f| {
        let before = ledger(&f.normal);
        let storage = Storage::open().unwrap();
        assert_eq!(storage.dir, f.normal);
        assert!(!f.trial.exists());
        drop(storage);
        let mut expected = before;
        expected.insert("app.lock".into(), (0, 0_u64.to_le_bytes().to_vec()));
        assert_eq!(ledger(&f.normal), expected);
        assert_eq!(fs::read(f.normal.join("app.lock")).unwrap(), b"");
    });
}

#[cfg(feature = "early-trial")]
#[test]
fn et01_trial_open_save_reopen_and_environment_change_keep_registered_root() {
    isolated(
        "et01_trial_open_save_reopen_and_environment_change_keep_registered_root",
        |f| {
            let normal = ledger(&f.normal);
            let env_before = (
                std::env::var_os("LOCALAPPDATA"),
                std::env::var_os("CODEX_HOME"),
            );
            let storage = Storage::open().expect("trial open");
            assert_eq!(storage.dir, f.trial);
            let mut p = storage.load().unwrap();
            assert!(p.audio_history.is_none());
            p.settings.topic = "synthetic saved settings".into();
            storage.save(&p).unwrap();
            assert_eq!(
                (
                    std::env::var_os("LOCALAPPDATA"),
                    std::env::var_os("CODEX_HOME")
                ),
                env_before
            );
            drop(storage);
            std::env::set_current_dir(&f.outside).unwrap();
            std::env::set_var("LOCALAPPDATA", f.base.join("changed-environment"));
            assert_eq!(resolve_storage_root().unwrap(), f.trial);
            let reopened = Storage::open().unwrap();
            assert_eq!(reopened.dir, f.trial);
            let p = reopened.load().unwrap();
            assert_eq!(p.settings.topic, "synthetic saved settings");
            assert!(p.audio_history.is_none());
            assert!(!f.base.join("changed-environment").exists());
            assert_eq!(ledger(&f.normal), normal);
            assert!(fs::read_dir(&f.trial).unwrap().all(|e| !e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("probe")));
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et01_missing_or_relative_localappdata_refuses_without_fallback() {
    isolated(
        "et01_missing_or_relative_localappdata_refuses_without_fallback",
        |f| {
            std::env::remove_var("LOCALAPPDATA");
            f.refuse_before_writes();
            std::env::set_var("LOCALAPPDATA", "relative-local");
            f.refuse_before_writes();
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et01_normal_content_is_not_read_and_normal_absence_is_not_created() {
    isolated(
        "et01_normal_content_is_not_read_and_normal_absence_is_not_created",
        |f| {
            let before = ledger(&f.normal);
            // Sharing denial proves the malformed normal progress cannot be read.
            let held = OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(f.normal.join("progress.json"))
                .unwrap();
            assert_eq!(
                fs::read(f.normal.join("progress.json"))
                    .unwrap_err()
                    .raw_os_error(),
                Some(32)
            );
            let storage = Storage::open().unwrap();
            assert!(storage.load().unwrap().audio_history.is_none());
            drop(storage);
            drop(held);
            assert_eq!(ledger(&f.normal), before);
            remove_fixture(&f.normal);
            let storage = Storage::open().unwrap();
            assert!(!f.normal.exists(), "normal root must not be created");
            assert_eq!(storage.dir, f.trial);
        },
    );
}

#[cfg(feature = "early-trial")]
fn history_progress() -> String {
    V2.replace(
        "\"audio_history\":null",
        &format!(
            "\"audio_history\":{{\"sha256\":\"{}\",\"bytes\":12}}",
            "a".repeat(64)
        ),
    )
}

#[cfg(feature = "early-trial")]
#[test]
fn et04_current_history_unknown_and_corrupt_progress_refuse_unchanged() {
    isolated(
        "et04_current_history_unknown_and_corrupt_progress_refuse_unchanged",
        |f| {
            f.seed_trial();
            let history = history_progress();
            // Validate the independently authored wire fixture before exercising rejection.
            let p: wordweave5::store::Progress =
                serde_json::from_str(&history).expect("FIXTURE_FAILURE: structural history");
            assert!(p.audio_history.is_some());
            for raw in [
                history,
                V2.replace("\"version\":2", "\"version\":999"),
                "broken JSON".into(),
                V2.replacen('{', "{\"version\":2,", 1),
            ] {
                fs::write(f.trial.join("progress.json"), raw).unwrap();
                f.refuse_before_writes();
            }
            fs::write(f.trial.join("progress.json"), V2).unwrap();
            let retry = Storage::open().unwrap();
            assert!(retry.load().unwrap().audio_history.is_none());
        },
    );
}

#[cfg(feature = "early-trial")]
fn pending(old: Option<&str>, next: &str) -> Vec<u8> {
    // Serialize string escaping only; never use the product Intent writer as oracle.
    serde_json::to_vec(&serde_json::json!({"old_deck":"synthetic old deck", "old_progress":old, "deck":wordweave5::model::BUILTIN_DECK, "progress":next})).unwrap()
}

#[cfg(feature = "early-trial")]
#[test]
fn et04_pending_old_and_new_history_or_invalid_documents_refuse_before_recovery() {
    isolated(
        "et04_pending_old_and_new_history_or_invalid_documents_refuse_before_recovery",
        |f| {
            f.seed_trial();
            let history = history_progress();
            let p: wordweave5::store::Progress =
                serde_json::from_str(&history).expect("FIXTURE_FAILURE: history wire");
            assert!(p.audio_history.is_some());
            for (old, next) in [
                (history.as_str(), V2),
                (V2, history.as_str()),
                ("broken", V2),
                (V2, "broken"),
            ] {
                fs::write(
                    f.trial.join("pending-material-commit.json"),
                    pending(Some(old), next),
                )
                .unwrap();
                f.refuse_before_writes();
            }
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et04_nonhistory_pending_recovers_exact_bytes_and_keeps_orphan_history() {
    isolated(
        "et04_nonhistory_pending_recovers_exact_bytes_and_keeps_orphan_history",
        |f| {
            f.seed_trial();
            let next = V2.replace("synthetic old", "synthetic recovered");
            let raw = pending(Some(V2), &next);
            fs::write(f.trial.join("pending-material-commit.json"), &raw).unwrap();
            fs::create_dir_all(f.trial.join("assets/audio-history")).unwrap();
            let orphan = f.trial.join("assets/audio-history/unreferenced.bin");
            fs::write(&orphan, b"unreferenced synthetic object").unwrap();
            let normal = ledger(&f.normal);
            let storage = Storage::open().unwrap();
            let p = storage.load().unwrap();
            assert_eq!(p.settings.topic, "synthetic recovered");
            assert!(p.audio_history.is_none());
            assert_eq!(
                fs::read(f.trial.join("progress.json")).unwrap(),
                next.as_bytes()
            );
            assert_eq!(
                fs::read(f.trial.join("custom.tsv")).unwrap(),
                wordweave5::model::BUILTIN_DECK.as_bytes()
            );
            assert!(!f.trial.join("pending-material-commit.json").exists());
            let archives: Vec<_> = fs::read_dir(f.trial.join("backups"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| {
                    p.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("material-commit-")
                })
                .collect();
            assert_eq!(archives.len(), 1);
            assert_eq!(fs::read(&archives[0]).unwrap(), raw);
            assert_eq!(fs::read(orphan).unwrap(), b"unreferenced synthetic object");
            assert_eq!(ledger(&f.normal), normal);
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et04_load_rechecks_current_and_pending_after_open_without_recovery_writes() {
    isolated(
        "et04_load_rechecks_current_and_pending_after_open_without_recovery_writes",
        |f| {
            f.seed_trial();
            let storage = Storage::open().unwrap();
            assert_eq!(
                storage.dir, f.trial,
                "load must exercise the admitted trial root"
            );
            assert!(storage.load().unwrap().audio_history.is_none());
            let history = history_progress();
            fs::write(f.trial.join("progress.json"), &history).unwrap();
            let before = ledger(&f.base);
            assert!(storage.load().is_err());
            assert_eq!(ledger(&f.base), before);
            fs::write(f.trial.join("progress.json"), V2).unwrap();
            fs::write(
                f.trial.join("pending-material-commit.json"),
                pending(Some(&history), V2),
            )
            .unwrap();
            let before = ledger(&f.base);
            assert!(storage.load().is_err());
            assert_eq!(ledger(&f.base), before);
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et01_trial_hardlinked_file_refuses_and_preserves_both_names() {
    isolated(
        "et01_trial_hardlinked_file_refuses_and_preserves_both_names",
        |f| {
            f.seed_trial();
            fs::hard_link(f.normal.join("custom.tsv"), f.trial.join("linked.tsv"))
                .expect("FIXTURE_FAILURE: ordinary hard link");
            f.refuse_before_writes();
            fs::remove_file(f.trial.join("linked.tsv")).unwrap();
            Storage::open().unwrap();
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et01_trial_identity_unavailable_is_not_treated_as_absent() {
    isolated(
        "et01_trial_identity_unavailable_is_not_treated_as_absent",
        |f| {
            f.seed_trial();
            let before = ledger(&f.base);
            let held = OpenOptions::new()
                .read(true)
                .share_mode(0)
                .custom_flags(0x02000000)
                .open(&f.trial)
                .expect("FIXTURE_FAILURE: directory sharing lock");
            let probe = OpenOptions::new()
                .read(true)
                .custom_flags(0x02000000)
                .open(&f.trial);
            assert_eq!(
                probe.unwrap_err().raw_os_error(),
                Some(32),
                "independent identity open denial"
            );
            let result = Storage::open();
            drop(held);
            assert!(result.is_err());
            assert_eq!(ledger(&f.base), before);
            Storage::open().unwrap();
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et05_registered_caller_normal_and_trial_roots_reject_files_directories_and_retry_outside() {
    isolated(
        "et05_registered_caller_normal_and_trial_roots_reject_files_directories_and_retry_outside",
        |f| {
            let storage = Storage::open().unwrap();
            assert_eq!(
                storage.dir, f.trial,
                "register the trial root before destination checks"
            );
            let caller = f.base.join("caller");
            fs::create_dir(&caller).unwrap();
            for root in [&caller, &f.normal, &f.trial] {
                for dest in [
                    root.to_owned(),
                    root.join("new.txt"),
                    root.join("progress.json"),
                ] {
                    let before = ledger(&f.base);
                    assert!(
                        write_external_file(&caller, &dest, PAYLOAD).is_err(),
                        "protected file: {dest:?}"
                    );
                    assert!(
                        create_external_directory(&caller, &dest).is_err(),
                        "protected directory: {dest:?}"
                    );
                    assert_eq!(ledger(&f.base), before);
                }
            }
            let before_normal = ledger(&f.normal);
            let before_trial = ledger(&f.trial);
            let dest = f.outside.join("export.txt");
            write_external_file(&caller, &dest, PAYLOAD).unwrap();
            assert_eq!(fs::read(&dest).unwrap(), PAYLOAD);
            create_external_directory(&caller, &f.outside.join("bundle")).unwrap();
            assert_eq!(ledger(&f.normal), before_normal);
            assert_eq!(ledger(&storage.dir), before_trial);
        },
    );
}

#[cfg(feature = "early-trial")]
fn junction(link: &Path, target: &Path) {
    #[link(name = "kernel32")]
    extern "system" {
        fn DeviceIoControl(
            h: *mut c_void,
            code: u32,
            input: *const c_void,
            len: u32,
            output: *mut c_void,
            size: u32,
            returned: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;
    }
    fs::create_dir(link).unwrap();
    let canonical = fs::canonicalize(target).unwrap();
    let wide: Vec<u16> = canonical.as_os_str().encode_wide().collect();
    assert!(wide.starts_with(&r"\\?\".encode_utf16().collect::<Vec<_>>()));
    let print = &wide[4..];
    let sub: Vec<u16> = r"\??\"
        .encode_utf16()
        .chain(print.iter().copied())
        .collect();
    let sl = u16::try_from(sub.len() * 2).unwrap();
    let pl = u16::try_from(print.len() * 2).unwrap();
    let mut data = vec![];
    data.extend_from_slice(&0xa0000003_u32.to_le_bytes());
    data.extend_from_slice(&(8 + sl + 2 + pl + 2).to_le_bytes());
    for n in [0_u16, 0, sl, sl + 2, pl] {
        data.extend_from_slice(&n.to_le_bytes());
    }
    for n in sub
        .into_iter()
        .chain([0])
        .chain(print.iter().copied())
        .chain([0])
    {
        data.extend_from_slice(&n.to_le_bytes());
    }
    let handle = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(0x02000000 | 0x00200000)
        .open(link)
        .unwrap();
    let mut returned = 0;
    // SAFETY: valid live handle/buffers, synchronous call, no overlapped pointer.
    let ok = unsafe {
        DeviceIoControl(
            handle.as_raw_handle(),
            0x000900a4,
            data.as_ptr().cast(),
            data.len() as u32,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(
        ok,
        0,
        "FIXTURE_FAILURE: junction {}",
        std::io::Error::last_os_error()
    );
    assert_ne!(
        fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
        0
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et01_trial_root_and_nested_junction_refuse_without_following_normal_tree() {
    isolated(
        "et01_trial_root_and_nested_junction_refuse_without_following_normal_tree",
        |f| {
            junction(&f.trial, &f.normal);
            f.refuse_before_writes();
            fs::remove_dir(&f.trial).unwrap();
            f.seed_trial();
            junction(&f.trial.join("assets"), &f.normal.join("assets"));
            f.refuse_before_writes();
        },
    );
}

#[cfg(feature = "early-trial")]
#[test]
fn et05_registered_root_aliases_and_hardlinks_reject_unchanged() {
    isolated(
        "et05_registered_root_aliases_and_hardlinks_reject_unchanged",
        |f| {
            let storage = Storage::open().unwrap();
            assert_eq!(
                storage.dir, f.trial,
                "register the trial root before alias fixtures"
            );
            let caller = f.base.join("caller");
            fs::create_dir(&caller).unwrap();
            for (index, root) in [&f.normal, &f.trial].into_iter().enumerate() {
                let alias = f.outside.join(format!("alias-{index}"));
                junction(&alias, root);
                let before = ledger(&f.base);
                assert!(write_external_file(&caller, &alias.join("new.txt"), PAYLOAD).is_err());
                assert!(create_external_directory(&caller, &alias.join("bundle")).is_err());
                assert_eq!(ledger(&f.base), before);
                let source = root.join("sentinel.txt");
                fs::write(&source, b"synthetic hardlink sentinel").unwrap();
                let alias_file = f.outside.join(format!("linked-{index}.txt"));
                fs::hard_link(&source, &alias_file).expect("FIXTURE_FAILURE: ordinary hard link");
                let before = ledger(&f.base);
                assert!(write_external_file(&caller, &alias_file, PAYLOAD).is_err());
                assert_eq!(ledger(&f.base), before);
            }
            drop(storage);
        },
    );
}
