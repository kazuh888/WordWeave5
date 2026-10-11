//! ACOUSTIC-HISTORY-001 I2d-H: specification-owned public destination tests.
//! FIXTURE_UNAVAILABLE is not acceptance evidence, even if libtest reports OK.
#![cfg(windows)]

use std::{
    collections::BTreeMap,
    ffi::{c_void, OsString},
    fs::{self, OpenOptions},
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wordweave5::store::{
    atomic_write, create_external_directory, resolve_storage_root, write_external_file, Storage,
};

const CHILD: &str = "WW5_EXPORT_DESTINATION_CHILD";
const CHILD_ROOT: &str = "WW5_EXPORT_DESTINATION_ROOT";
const PAYLOAD: &[u8] = "未保存の音声注釈と教材\n".as_bytes();
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    base: PathBuf,
    root: PathBuf,
    outside: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let parent = std::env::var_os(CHILD_ROOT)
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = parent.join(format!(
            "ww5-export-{}-{stamp}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let root = base.join("ProtectedHost");
        let outside = base.join("external");
        fs::create_dir_all(root.join("assets/audio-history")).unwrap();
        fs::create_dir(root.join("backups")).unwrap();
        fs::create_dir(&outside).unwrap();
        for (name, bytes) in [
            ("progress.json", b"exact synthetic progress".as_slice()),
            ("custom.tsv", b"exact synthetic deck".as_slice()),
            (
                "pending-material-commit.json",
                b"pending must not recover".as_slice(),
            ),
            ("app.lock", b"synthetic lock sentinel".as_slice()),
            ("backups/old.json", b"preserve backup".as_slice()),
            (
                "assets/audio-history/original.wav",
                b"synthetic original".as_slice(),
            ),
            ("日本語.txt", "既存の筆跡と会話".as_bytes()),
        ] {
            fs::write(root.join(name), bytes).unwrap();
        }
        Self {
            base,
            root,
            outside,
        }
    }
    fn snapshot(&self) -> BTreeMap<PathBuf, (u32, Vec<u8>)> {
        ledger(&self.base)
    }
    fn reject_file(&self, root: &Path, dest: &Path) {
        let before = self.snapshot();
        assert!(
            write_external_file(root, dest, PAYLOAD).is_err(),
            "H rejection expected: {dest:?}"
        );
        assert_eq!(
            self.snapshot(),
            before,
            "rejection changed bytes, directories, or temp set"
        );
    }
    fn reject_dir(&self, root: &Path, dest: &Path) {
        let before = self.snapshot();
        assert!(
            create_external_directory(root, dest).is_err(),
            "H directory rejection expected: {dest:?}"
        );
        assert_eq!(self.snapshot(), before);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Only visit this exclusively created fixture. Never recurse through a reparse point.
        fn remove(path: &Path) -> std::io::Result<()> {
            let m = fs::symlink_metadata(path)?;
            if m.file_attributes() & 0x400 != 0 {
                return if m.file_attributes() & 0x10 != 0 {
                    fs::remove_dir(path)
                } else {
                    fs::remove_file(path)
                };
            }
            if m.is_dir() {
                for e in fs::read_dir(path)? {
                    remove(&e?.path())?;
                }
                fs::remove_dir(path)
            } else {
                fs::remove_file(path)
            }
        }
        if let Err(e) = remove(&self.base) {
            eprintln!("WW5_H_CLEANUP_FAILED:{}:{e}", self.base.display());
        }
    }
}
fn ledger(base: &Path) -> BTreeMap<PathBuf, (u32, Vec<u8>)> {
    fn visit(base: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, (u32, Vec<u8>)>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            let m = fs::symlink_metadata(&p).unwrap();
            let attrs = m.file_attributes();
            let bytes = if attrs & 0x400 != 0 {
                b"REPARSE: not traversed".to_vec()
            } else if m.is_file() {
                fs::read(&p).unwrap()
            } else {
                vec![]
            };
            out.insert(
                p.strip_prefix(base).unwrap().to_owned(),
                (attrs & 0x410, bytes),
            );
            if m.is_dir() && attrs & 0x400 == 0 {
                visit(base, &p, out);
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(base, base, &mut out);
    out
}
fn unavailable(case: &str, why: impl std::fmt::Display) {
    eprintln!("WW5_H_FIXTURE_UNAVAILABLE:{case}:{why}; NOT BEHAVIOR PASS OR RED");
}
fn isolated(case: &str, body: impl FnOnce()) {
    if std::env::var(CHILD).as_deref() == Ok(case) {
        body();
        println!("WW5_H_CHILD_OK:{case}");
        return;
    }
    let f = Fixture::new();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([case, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, case)
        .env(CHILD_ROOT, &f.base)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut timeout = false;
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            timeout = true;
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!(
        "WW5_H_PARENT:{case}:exit={}:timeout={timeout}\n{stdout}\n{stderr}",
        output.status
    );
    assert!(!timeout, "child killed/reaped after 20 seconds");
    assert!(output.status.success(), "child behavior/fixture failure");
    assert!(
        stdout.contains("running 1 test")
            && stdout.contains("1 passed; 0 failed")
            && stdout.contains(&format!("WW5_H_CHILD_OK:{case}"))
    );
}

// Independent OS evidence uses an ordinary handle and the documented legacy
// file identity. It does not call the product path checker or provide its data.
#[repr(C)]
#[derive(Default)]
struct FileInfo {
    attrs: u32,
    created: [u32; 2],
    accessed: [u32; 2],
    written: [u32; 2],
    volume: u32,
    size_hi: u32,
    size_lo: u32,
    links: u32,
    index_hi: u32,
    index_lo: u32,
}
#[link(name = "kernel32")]
extern "system" {
    fn GetFileInformationByHandle(handle: *mut c_void, info: *mut FileInfo) -> i32;
    fn GetShortPathNameW(long: *const u16, short: *mut u16, size: u32) -> u32;
    fn GetLogicalDrives() -> u32;
    fn DeviceIoControl(
        handle: *mut c_void,
        code: u32,
        input: *const c_void,
        input_len: u32,
        output: *mut c_void,
        output_len: u32,
        returned: *mut u32,
        overlapped: *mut c_void,
    ) -> i32;
}
fn identity(path: &Path) -> ((u32, u32, u32), u32) {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0x0200_0000)
        .open(path)
        .unwrap();
    let mut info = FileInfo::default();
    // SAFETY: the handle and complete repr(C) output struct live through this call.
    assert_ne!(
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
        0,
        "independent identity probe failed"
    );
    ((info.volume, info.index_hi, info.index_lo), info.links)
}
fn junction(link: &Path, target: &Path) {
    fs::create_dir(link).unwrap();
    let canonical = fs::canonicalize(target).unwrap();
    let wide: Vec<u16> = canonical.as_os_str().encode_wide().collect();
    let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
    assert!(wide.starts_with(&prefix));
    let print = &wide[prefix.len()..];
    let sub: Vec<u16> = r"\??\"
        .encode_utf16()
        .chain(print.iter().copied())
        .collect();
    let sub_len = u16::try_from(sub.len() * 2).unwrap();
    let print_len = u16::try_from(print.len() * 2).unwrap();
    let mut buffer = vec![];
    buffer.extend_from_slice(&0xa000_0003_u32.to_le_bytes());
    buffer.extend_from_slice(&(8 + sub_len + 2 + print_len + 2).to_le_bytes());
    for n in [0_u16, 0, sub_len, sub_len + 2, print_len] {
        buffer.extend_from_slice(&n.to_le_bytes());
    }
    for n in sub
        .into_iter()
        .chain([0])
        .chain(print.iter().copied())
        .chain([0])
    {
        buffer.extend_from_slice(&n.to_le_bytes());
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(0x0200_0000 | 0x0020_0000)
        .open(link)
        .unwrap();
    let mut returned = 0;
    // SAFETY: synchronous call, owned handle, live input and output count; no overlapped I/O.
    let ok = unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            0x0009_00a4,
            buffer.as_ptr().cast(),
            buffer.len() as u32,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(
        ok,
        0,
        "FIXTURE_FAILURE junction: {}",
        std::io::Error::last_os_error()
    );
    assert_ne!(
        fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
        0
    );
    assert_eq!(
        identity(link).0,
        identity(target).0,
        "independent OS query confirms alias"
    );
}

#[test]
fn h01_every_protected_file_and_new_descendant_refuses_without_mutation() {
    let f = Fixture::new();
    for name in [
        "progress.json",
        "custom.tsv",
        "pending-material-commit.json",
        "app.lock",
        "backups/old.json",
        "assets/audio-history/original.wav",
        "日本語.txt",
        "new.json",
        "assets/audio-history/new.wav",
    ] {
        f.reject_file(&f.root, &f.root.join(name));
    }
}
#[test]
fn h01_root_and_descendant_directory_refuse_without_mutation() {
    let f = Fixture::new();
    for dest in [
        &f.root,
        &f.root.join("new-directory"),
        &f.root.join("assets/audio-history/new-directory"),
    ] {
        f.reject_dir(&f.root, dest);
    }
}
#[test]
fn h02_root_and_destination_case_aliases_are_protected() {
    let f = Fixture::new();
    let lower_root = f.base.join("protectedhost");
    assert_eq!(
        identity(&lower_root).0,
        identity(&f.root).0,
        "FIXTURE_FAILURE: filesystem is case sensitive"
    );
    f.reject_file(&lower_root, &f.root.join("progress.json"));
    f.reject_file(&f.root, &lower_root.join("PROGRESS.JSON"));
    f.reject_dir(&lower_root, &f.root.join("new-dir"));
}
#[test]
fn h02_existing_dot_and_dotdot_components_do_not_bypass_protection() {
    let f = Fixture::new();
    f.reject_file(&f.root, &f.root.join("assets/.././progress.json"));
    f.reject_file(&f.root.join("assets/.."), &f.root.join("progress.json"));
    f.reject_dir(&f.root.join("."), &f.root.join("assets/../new-dir"));
}
#[test]
fn h02_ads_device_and_ambiguous_names_refuse_without_writes() {
    let f = Fixture::new();
    for name in [
        "note.txt:stream",
        "CON",
        "con.txt",
        "NUL.bin",
        "COM1",
        "LPT9.txt",
        "COM¹.txt",
        "LPT²",
        "LPT³.txt",
        "trailing.",
        "trailing ",
    ] {
        f.reject_file(&f.root, &f.outside.join(name));
        f.reject_dir(&f.root, &f.outside.join(name));
    }
    for dest in [
        Path::new(r"C:relative.txt"),
        Path::new(r"\\.\NUL"),
        Path::new(r"\\?\GLOBALROOT\Device\Null"),
        Path::new("nul\0name"),
    ] {
        f.reject_file(&f.root, dest);
    }
}
#[test]
fn h02_wrong_types_and_missing_parent_refuse_without_mkdir() {
    let f = Fixture::new();
    fs::write(f.outside.join("ordinary"), b"old").unwrap();
    f.reject_file(&f.root, &f.outside);
    f.reject_dir(&f.root, &f.outside);
    f.reject_dir(&f.root, &f.outside.join("ordinary"));
    f.reject_file(&f.root, &f.outside.join("missing/file.txt"));
    f.reject_dir(&f.root, &f.outside.join("missing/directory"));
    f.reject_file(&f.root, &f.outside.join("ordinary/file.txt"));
    f.reject_file(&f.outside.join("ordinary"), &f.outside.join("output.txt"));
}
#[test]
fn h02_external_hardlink_to_protected_file_is_rejected() {
    let f = Fixture::new();
    let dest = f.outside.join("alias.json");
    let source = f.root.join("progress.json");
    fs::hard_link(&source, &dest).unwrap();
    assert_eq!(identity(&source).0, identity(&dest).0);
    assert_eq!(identity(&dest).1, 2);
    f.reject_file(&f.root, &dest);
}
#[test]
fn h02_external_hardlinks_are_rejected_even_when_both_names_are_outside() {
    let f = Fixture::new();
    let first = f.outside.join("first.txt");
    let second = f.outside.join("second.txt");
    fs::write(&first, b"outside sentinel").unwrap();
    fs::hard_link(&first, &second).unwrap();
    assert_eq!(identity(&first).0, identity(&second).0);
    assert_eq!(identity(&second).1, 2);
    f.reject_file(&f.root, &second);
}
#[test]
fn h02_junction_ancestors_and_junction_dotdot_are_rejected() {
    let f = Fixture::new();
    let alias = f.base.join("alias");
    junction(&alias, &f.root);
    f.reject_file(&f.root, &alias.join("progress.json"));
    f.reject_dir(&f.root, &alias.join("new-dir"));
    f.reject_file(&f.root, &alias.join("../external/new.txt"));
    f.reject_file(&alias, &f.outside.join("new.txt"));
}
#[test]
fn h02_junction_to_safe_outside_is_still_rejected() {
    let f = Fixture::new();
    let alias = f.base.join("outside-alias");
    junction(&alias, &f.outside);
    f.reject_file(&f.root, &alias.join("new.txt"));
    f.reject_dir(&f.root, &alias.join("new-dir"));
    f.reject_file(&f.root, &alias);
}
#[test]
fn h02_leaf_file_symlink_is_rejected_when_fixture_is_available() {
    let f = Fixture::new();
    let alias = f.outside.join("link.json");
    if let Err(e) = std::os::windows::fs::symlink_file(f.root.join("progress.json"), &alias) {
        unavailable("h02_leaf_file_symlink", e);
        return;
    }
    assert_ne!(
        fs::symlink_metadata(&alias).unwrap().file_attributes() & 0x400,
        0
    );
    assert_eq!(
        identity(&alias).0,
        identity(&f.root.join("progress.json")).0
    );
    f.reject_file(&f.root, &alias);
}
#[test]
fn h02_directory_symlink_is_rejected_when_fixture_is_available() {
    let f = Fixture::new();
    let alias = f.base.join("directory-link");
    if let Err(e) = std::os::windows::fs::symlink_dir(&f.outside, &alias) {
        unavailable("h02_directory_symlink", e);
        return;
    }
    assert_ne!(
        fs::symlink_metadata(&alias).unwrap().file_attributes() & 0x400,
        0
    );
    assert_eq!(identity(&alias).0, identity(&f.outside).0);
    f.reject_file(&f.root, &alias.join("new.txt"));
    f.reject_dir(&f.root, &alias.join("new-dir"));
}
#[test]
fn h02_short_name_alias_is_protected_when_os_provides_one() {
    let f = Fixture::new();
    let mut input: Vec<u16> = f.root.as_os_str().encode_wide().collect();
    input.push(0);
    let mut buffer = vec![0_u16; 32768];
    // SAFETY: terminated input and valid output capacity are supplied.
    let n = unsafe { GetShortPathNameW(input.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) }
        as usize;
    if n == 0 || n >= buffer.len() {
        unavailable("h02_short_name", std::io::Error::last_os_error());
        return;
    }
    let alias = PathBuf::from(OsString::from_wide(&buffer[..n]));
    if alias == f.root {
        unavailable("h02_short_name", "OS returned no distinct short name");
        return;
    }
    assert_eq!(identity(&alias).0, identity(&f.root).0);
    eprintln!("WW5_H_ALIAS_CONFIRMED:short-name");
    f.reject_file(&f.root, &alias.join("progress.json"));
    f.reject_file(&alias, &f.root.join("progress.json"));
}
#[test]
fn h02_existing_alternate_drive_alias_is_protected_when_available() {
    let f = Fixture::new();
    let expected = identity(&f.root).0;
    let suffix: PathBuf = f.root.components().skip(2).collect();
    // SAFETY: read-only process-independent OS drive inventory; no mapping changes.
    let drives = unsafe { GetLogicalDrives() };
    let mut found = false;
    for n in 0..26 {
        if drives & (1 << n) == 0 {
            continue;
        }
        let alias = PathBuf::from(format!("{}:\\", (b'A' + n) as char)).join(&suffix);
        if alias == f.root || !alias.is_dir() {
            continue;
        }
        if identity(&alias).0 != expected {
            continue;
        }
        found = true;
        eprintln!("WW5_H_ALIAS_CONFIRMED:existing-alternate-drive");
        f.reject_file(&f.root, &alias.join("progress.json"));
        f.reject_file(&alias, &f.root.join("progress.json"));
    }
    if !found {
        unavailable(
            "h02_alternate_drive",
            "no pre-existing same-volume drive alias for owned fixture",
        );
    }
}
#[test]
fn h03_planned_root_itself_is_protected_in_both_modes_and_case_spellings() {
    let f = Fixture::new();
    let planned = f.outside.join("PlannedHost");
    for dest in [&planned, &f.outside.join("plannedhost")] {
        f.reject_file(&planned, dest);
        f.reject_dir(&planned, dest);
    }
    assert!(!planned.exists());
    let unicode_planned = f.outside.join("Ångström");
    let unicode_alias = f.outside.join("ångström");
    f.reject_file(&unicode_planned, &unicode_alias);
    f.reject_dir(&unicode_planned, &unicode_alias);
}
#[test]
fn h03_uncreated_ancestor_of_multicomponent_root_is_rejected_in_both_modes() {
    let f = Fixture::new();
    let ancestor = f.outside.join("New");
    let planned = ancestor.join("assets/deeper");
    f.reject_file(&planned, &ancestor);
    f.reject_dir(&planned, &ancestor);
    f.reject_file(&planned, &f.outside.join("new"));
    f.reject_dir(&planned, &f.outside.join("new"));
    f.reject_file(&planned, &planned.join("new.json"));
    f.reject_dir(&planned, &planned.join("new-dir"));
    assert!(!ancestor.exists());
}
#[test]
fn h03_planned_root_with_junction_ancestor_refuses() {
    let f = Fixture::new();
    let alias = f.base.join("planned-alias");
    junction(&alias, &f.outside);
    let planned = alias.join("not-created/deeper");
    f.reject_file(&planned, &f.outside.join("export.txt"));
    f.reject_dir(&planned, &f.outside.join("export-dir"));
}
#[test]
fn h03_sibling_export_does_not_create_planned_root_or_parents() {
    let f = Fixture::new();
    let planned = f.outside.join("New/assets");
    write_external_file(&planned, &f.outside.join("sibling.txt"), PAYLOAD).unwrap();
    create_external_directory(&planned, &f.outside.join("sibling-dir")).unwrap();
    assert_eq!(fs::read(f.outside.join("sibling.txt")).unwrap(), PAYLOAD);
    assert!(f.outside.join("sibling-dir").is_dir());
    assert!(!f.outside.join("New").exists());
}
#[test]
fn h03_resolver_is_side_effect_free_and_tracks_environment_only_in_child() {
    isolated(
        "h03_resolver_is_side_effect_free_and_tracks_environment_only_in_child",
        || {
            let f = Fixture::new();
            let before = f.snapshot();
            let first = f.base.join("LocalAppDataMissing");
            std::env::set_var("LOCALAPPDATA", &first);
            assert_eq!(resolve_storage_root().unwrap(), first.join("WordWeave5"));
            assert_eq!(f.snapshot(), before);
            std::env::set_var("LOCALAPPDATA", &f.outside);
            assert_eq!(
                resolve_storage_root().unwrap(),
                f.outside.join("WordWeave5")
            );
            assert_eq!(f.snapshot(), before);
            std::env::remove_var("LOCALAPPDATA");
            assert_eq!(
                resolve_storage_root(),
                Err("LOCALAPPDATAが見つかりません。".into())
            );
            assert_eq!(f.snapshot(), before);
        },
    );
}
#[test]
fn h03_custom_locked_storage_root_remains_protected_without_opening_storage() {
    let f = Fixture::new();
    let custom = f.base.join("custom-storage");
    let storage = Storage::at(custom.clone()).unwrap();
    assert!(
        Storage::at(custom.clone()).is_err(),
        "independent fixture establishes Storage acquisition failure"
    );
    // Do not read bytes from the exclusively locked app.lock. Directory entries
    // prove no extra destination/temp was made while Storage acquisition fails.
    assert_eq!(fs::read_dir(&custom).unwrap().count(), 1);
    assert!(write_external_file(&custom, &custom.join("recovery.json"), PAYLOAD).is_err());
    assert!(create_external_directory(&custom, &custom.join("backup")).is_err());
    assert_eq!(fs::read_dir(&custom).unwrap().count(), 1);
    drop(storage);
    f.reject_file(&custom, &custom.join("recovery.json"));
    f.reject_dir(&custom, &custom.join("backup"));
}
#[test]
fn h04_external_new_replace_empty_and_japanese_bytes_succeed_exactly() {
    let f = Fixture::new();
    let protected = ledger(&f.root);
    let dest = f.outside.join("普通の外部保存.txt");
    write_external_file(&f.root, &dest, b"").unwrap();
    assert_eq!(fs::read(&dest).unwrap(), b"");
    assert_eq!(identity(&dest).1, 1);
    write_external_file(&f.root, &dest, PAYLOAD).unwrap();
    assert_eq!(fs::read(&dest).unwrap(), PAYLOAD);
    assert_eq!(ledger(&f.root), protected);
    assert_eq!(
        fs::read_dir(&f.outside).unwrap().count(),
        1,
        "no temporary files remain"
    );
}
#[test]
fn h04_new_directory_and_similarly_named_sibling_succeed() {
    let f = Fixture::new();
    let protected = ledger(&f.root);
    let dest = f.base.join("ProtectedHost-export");
    create_external_directory(&f.root, &dest).unwrap();
    assert!(dest.is_dir());
    assert_eq!(fs::read_dir(&dest).unwrap().count(), 0);
    write_external_file(&f.root, &dest.join("日本語.tsv"), PAYLOAD).unwrap();
    assert_eq!(fs::read(dest.join("日本語.tsv")).unwrap(), PAYLOAD);
    assert_eq!(ledger(&f.root), protected);
}
#[test]
fn h04_legacy_atomic_write_keeps_new_replace_and_io_failure_contract() {
    let f = Fixture::new();
    let dest = f.outside.join("legacy.txt");
    atomic_write(&dest, b"first").unwrap();
    assert_eq!(fs::read(&dest).unwrap(), b"first");
    atomic_write(&dest, PAYLOAD).unwrap();
    assert_eq!(fs::read(&dest).unwrap(), PAYLOAD);
    let before = f.snapshot();
    assert!(atomic_write(&f.outside.join("missing/file"), PAYLOAD).is_err());
    assert!(atomic_write(&f.outside, PAYLOAD).is_err());
    assert_eq!(
        f.snapshot(),
        before,
        "legacy error cleans only its generated temp"
    );
}

// Q-H-CWD: public regression packet; all process-wide cwd changes stay in children.
mod q_h_cwd_tests {
    use super::*;
    use std::path::{Component, Prefix};

    struct RestoreCwd(PathBuf);
    impl RestoreCwd {
        fn capture() -> Self {
            Self(std::env::current_dir().unwrap())
        }
    }
    impl Drop for RestoreCwd {
        fn drop(&mut self) {
            match std::env::set_current_dir(&self.0) {
                Ok(()) => eprintln!("WW5_Q_CWD_RESTORED"),
                Err(error) => eprintln!("WW5_Q_CWD_RESTORE_FAILED:{error}"),
            }
        }
    }

    fn set_fixture_cwd(f: &Fixture, verbatim: bool) {
        let requested = if verbatim {
            fs::canonicalize(&f.base).unwrap()
        } else {
            f.base.clone()
        };
        std::env::set_current_dir(&requested).unwrap();
        let observed = std::env::current_dir().unwrap();
        let prefix = match observed.components().next() {
            Some(Component::Prefix(prefix)) => prefix.kind(),
            other => panic!("FIXTURE_FAILURE: cwd has no Windows prefix: {other:?}"),
        };
        eprintln!("WW5_Q_CWD_OBSERVED:verbatim={verbatim}:prefix={prefix:?}:cwd={observed:?}");
        assert!(
            if verbatim {
                matches!(prefix, Prefix::VerbatimDisk(_))
            } else {
                matches!(prefix, Prefix::Disk(_))
            },
            "FIXTURE_FAILURE: actual cwd prefix differs from requested condition"
        );
        assert_eq!(identity(&observed).0, identity(&f.base).0);
    }

    fn reject_original_junction(root_input: bool, directory: bool) {
        let f = Fixture::new();
        // Declared after Fixture: on assertion panic, cwd restores before fixture removal.
        let restore = RestoreCwd::capture();
        junction(&f.base.join("cwd-junction"), &f.outside);
        eprintln!("WW5_Q_JUNCTION_IDENTITY_CONFIRMED");
        for verbatim in [false, true] {
            set_fixture_cwd(&f, verbatim);
            let root = if root_input {
                PathBuf::from(r"cwd-junction\..\ProtectedHost")
            } else {
                f.root.clone()
            };
            let dest = if root_input {
                f.outside.join("must-remain-absent")
            } else {
                PathBuf::from(r"cwd-junction\..\external\must-remain-absent")
            };
            let before = f.snapshot();
            let result = if directory {
                create_external_directory(&root, &dest)
            } else {
                write_external_file(&root, &dest, PAYLOAD)
            };
            let after = f.snapshot();
            eprintln!(
                "WW5_Q_REFUSAL:verbatim={verbatim}:root_input={root_input}:directory={directory}:is_err={}:ledger_unchanged={}:outside_entries={}",
                result.is_err(), after == before, fs::read_dir(&f.outside).unwrap().count()
            );
            assert!(
                result.is_err(),
                "Q-H-CWD must observe the original junction before resolving .."
            );
            assert_eq!(
                after, before,
                "refusal must preserve bytes and the complete directory/temp set"
            );
            assert!(!f.outside.join("must-remain-absent").try_exists().unwrap());
        }
        drop(restore);
    }

    fn separator_spelling(path: &Path, style: usize) -> PathBuf {
        let mut separators = 0;
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .map(|unit| {
                if unit != b'\\' as u16 {
                    return unit;
                }
                separators += 1;
                if style == 1 || (style == 2 && separators % 2 == 1) {
                    b'/' as u16
                } else {
                    unit
                }
            })
            .collect();
        PathBuf::from(OsString::from_wide(&wide))
    }

    fn normal_inputs(directory: bool) {
        let f = Fixture::new();
        let restore = RestoreCwd::capture();
        set_fixture_cwd(&f, true);
        let protected = ledger(&f.root);
        for form in ["relative", "rooted-relative", "absolute"] {
            for style in 0..3 {
                let leaf = format!("{form}-{style}");
                let expected = f.outside.join(&leaf);
                let (root, dest) = match form {
                    "relative" => (
                        PathBuf::from(r".\ProtectedHost"),
                        PathBuf::from(format!(r"external\.\{leaf}")),
                    ),
                    "rooted-relative" => (
                        f.root.components().skip(1).collect::<PathBuf>(),
                        expected.components().skip(1).collect::<PathBuf>(),
                    ),
                    "absolute" => (f.root.clone(), expected.clone()),
                    _ => unreachable!(),
                };
                let root = separator_spelling(&root, style);
                let dest = separator_spelling(&dest, style);
                let result = if directory {
                    create_external_directory(&root, &dest)
                } else {
                    write_external_file(&root, &dest, PAYLOAD)
                };
                eprintln!("WW5_Q_NORMAL:directory={directory}:form={form}:separator_style={style}:is_ok={}", result.is_ok());
                result.unwrap();
                if directory {
                    assert!(expected.is_dir());
                    assert_eq!(fs::read_dir(&expected).unwrap().count(), 0);
                } else {
                    assert_eq!(fs::read(&expected).unwrap(), PAYLOAD);
                }
                assert_eq!(ledger(&f.root), protected);
            }
        }
        assert_eq!(
            fs::read_dir(&f.outside).unwrap().count(),
            9,
            "only the nine requested destinations, no temps"
        );
        drop(restore);
    }

    #[test]
    fn relative_root_junction_dotdot_refuses_file_with_real_verbatim_cwd() {
        isolated(
            "q_h_cwd_tests::relative_root_junction_dotdot_refuses_file_with_real_verbatim_cwd",
            || reject_original_junction(true, false),
        );
    }
    #[test]
    fn relative_root_junction_dotdot_refuses_directory_with_real_verbatim_cwd() {
        isolated(
            "q_h_cwd_tests::relative_root_junction_dotdot_refuses_directory_with_real_verbatim_cwd",
            || reject_original_junction(true, true),
        );
    }
    #[test]
    fn relative_destination_junction_dotdot_refuses_file_with_real_verbatim_cwd() {
        isolated("q_h_cwd_tests::relative_destination_junction_dotdot_refuses_file_with_real_verbatim_cwd", || reject_original_junction(false, false));
    }
    #[test]
    fn relative_destination_junction_dotdot_refuses_directory_with_real_verbatim_cwd() {
        isolated("q_h_cwd_tests::relative_destination_junction_dotdot_refuses_directory_with_real_verbatim_cwd", || reject_original_junction(false, true));
    }
    #[test]
    fn normal_file_path_forms_and_separators_work_with_real_verbatim_cwd() {
        isolated(
            "q_h_cwd_tests::normal_file_path_forms_and_separators_work_with_real_verbatim_cwd",
            || normal_inputs(false),
        );
    }
    #[test]
    fn normal_directory_path_forms_and_separators_work_with_real_verbatim_cwd() {
        isolated(
            "q_h_cwd_tests::normal_directory_path_forms_and_separators_work_with_real_verbatim_cwd",
            || normal_inputs(true),
        );
    }
}
