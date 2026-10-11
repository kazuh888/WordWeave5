//! Host-owned reading of immutable audio history; publication is not a commit.
//!
//! The shared DTOs have no storage dependency. This adapter borrows an already
//! locked application store and never opens the user's default directory.

use acoustic_analysis::AnalysisReport;
pub use practice_history::*;

use crate::store::Storage;

mod closure;
mod mutation;
mod recovery;
mod results;
mod store;

pub use recovery::{RecoveryCandidate, RecoveryState, RecoveryView};
pub use store::*;

use closure::{check_trial, error, ReadSession, ORIGINAL_CAP};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryLimits {
    pub max_nodes: usize,
    pub max_total_bytes: u64,
}

impl HistoryLimits {
    pub fn v1() -> Self {
        Self {
            max_nodes: 100_000,
            max_total_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryErrorKind {
    InvalidDocument,
    UnsupportedVersion,
    MissingObject,
    DamagedObject,
    RelationshipMismatch,
    LimitExceeded,
    UnsafePath,
    Io,
    Conflict,
    Busy,
    PendingMaterial,
    QuotaExceeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryError {
    pub kind: HistoryErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedClosure {
    pub objects: Vec<BlobRef>,
    pub total_bytes: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryView {
    pub root: Option<HistoryRoot>,
    pub trials: Vec<TrialView>,
    pub next_after: Option<TrialId>,
    pub diagnostics: Vec<HistoryDiagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrialView {
    pub document: TrialRevision,
    pub original_available: bool,
    pub diagnostics: Vec<HistoryDiagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisView {
    pub run: AnalysisRun,
    pub result: Option<AnalysisReport>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdviceView {
    pub run: AdviceRun,
    pub result: Option<AdviceResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryDiagnostic {
    pub trial: Option<TrialId>,
    pub error: HistoryError,
}

pub struct HistoryReader<'a> {
    root: &'a std::path::Path,
    limits: HistoryLimits,
}

impl<'a> HistoryReader<'a> {
    /// Borrows a host root without acquiring a lock or changing stored state.
    pub(crate) fn for_root(
        root: &'a std::path::Path,
        limits: HistoryLimits,
    ) -> Result<Self, HistoryError> {
        if limits.max_nodes == 0 || limits.max_nodes > 100_000 || limits.max_total_bytes == 0 {
            return Err(error(HistoryErrorKind::InvalidDocument));
        }
        Ok(Self { root, limits })
    }

    pub fn new(storage: &'a Storage) -> Self {
        Self {
            root: &storage.dir,
            limits: HistoryLimits::v1(),
        }
    }

    pub fn with_limits(storage: &'a Storage, limits: HistoryLimits) -> Result<Self, HistoryError> {
        Self::for_root(&storage.dir, limits)
    }

    /// A partial browsing view, never authority for a write or saved-state claim.
    pub fn read_snapshot(&self, root: Option<&RootRef>) -> HistoryView {
        self.read_page(root, None)
    }

    /// The cursor counts examined heads, including unreadable ones.
    pub fn read_page(&self, root: Option<&RootRef>, after: Option<&TrialId>) -> HistoryView {
        closure::read_page(self, root, after)
    }

    pub fn read_analysis(
        &self,
        trial: &TrialRevision,
        run: &RunId,
    ) -> Result<AnalysisView, HistoryError> {
        check_trial(trial)?;
        ReadSession::new(self).analysis(trial, run)
    }

    pub fn read_advice(
        &self,
        trial: &TrialRevision,
        run: &RunId,
    ) -> Result<AdviceView, HistoryError> {
        check_trial(trial)?;
        ReadSession::new(self).advice(trial, run)
    }

    pub fn read_reflection(
        &self,
        trial: &TrialRevision,
        reference: &BlobRef,
    ) -> Result<Reflection, HistoryError> {
        check_trial(trial)?;
        ReadSession::new(self).reflection(trial, reference)
    }

    /// Validates every reachable object, retaining references rather than bodies.
    pub fn verify_closure(&self, root: &RootRef) -> Result<VerifiedClosure, HistoryError> {
        closure::verify(self, root)
    }

    pub fn read_original(&self, reference: &BlobRef) -> Result<Vec<u8>, HistoryError> {
        ReadSession::new(self).bytes(reference, ORIGINAL_CAP)
    }
}

pub fn hash_reference(text: &str) -> Result<Sha256, HistoryError> {
    hash_bytes(text.as_bytes())
}

pub fn hash_goal(text: &str) -> Result<Sha256, HistoryError> {
    hash_bytes(text.as_bytes())
}

pub fn hash_capture(capture: &CaptureProfile) -> Result<Sha256, HistoryError> {
    // A struct fixes serialization order independently of serde_json map features.
    #[derive(serde::Serialize)]
    struct Conditions<'a> {
        device: &'a Option<String>,
        distance: &'a Option<String>,
        gain: &'a Option<String>,
        environment: &'a Option<String>,
    }
    let bytes = serde_json::to_vec(&Conditions {
        device: &capture.device,
        distance: &capture.distance,
        gain: &capture.gain,
        environment: &capture.environment,
    })
    .map_err(|_| error(HistoryErrorKind::InvalidDocument))?;
    hash_bytes(&bytes)
}

fn hash_bytes(bytes: &[u8]) -> Result<Sha256, HistoryError> {
    let hash = crate::assets::sha256(bytes).map_err(|_| error(HistoryErrorKind::Io))?;
    Sha256::parse(&hash).map_err(closure::contract_error)
}

#[cfg(all(test, windows))]
mod i2_path_adapter_tests {
    use super::*;
    use std::{
        collections::BTreeMap,
        fs,
        os::windows::{ffi::OsStrExt, fs::MetadataExt, fs::OpenOptionsExt, io::AsRawHandle},
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    // I2a-0 / AH-AC04/16: handwritten data, no history writer or analyzer.
    // Reduced from tests/audio_history_closure.rs, without copying its suite.
    const RAW: &[u8] = b"I2 isolated synthetic original";
    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        temp: PathBuf,
        host: PathBuf,
        links: Vec<PathBuf>,
        trial: TrialRevision,
        original: BlobRef,
        root: RootRef,
        expected: VerifiedClosure,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let temp = std::env::temp_dir().join(format!(
                "ww5-i2-path-{}-{nonce}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&temp).expect("create unique isolated fixture");
            let host = temp.join("parent/host");
            fs::create_dir_all(host.join("audio-history/assets")).unwrap();
            fs::write(
                temp.join("neighbor-host-sentinel"),
                b"preserve neighbor bytes",
            )
            .unwrap();
            let original = put(&host, RAW);
            let text = "Read the exact synthetic sentence.";
            let trial = TrialRevision {
                schema_version: 1,
                id: TrialId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
                revision: 1,
                previous: None,
                created_at: RecordedAt {
                    utc_millis: 1_700_000_000_000,
                    offset_minutes: 540,
                },
                input: FrozenInput {
                    audio: AudioSnapshot {
                        original: original.clone(),
                        format: AudioFormat::Wav,
                        rate: 8_000,
                        channels: 1,
                        frames: 1,
                        origin: Origin::Imported,
                        warnings: vec![],
                    },
                    reference: ReferenceSnapshot {
                        text: text.into(),
                        sha256: digest(text.as_bytes()),
                        source: None,
                    },
                    goal: None,
                    capture: CaptureProfile {
                        device: None,
                        distance: None,
                        gain: None,
                        environment: None,
                        confirmed_at: None,
                        version: digest(
                            br#"{"device":null,"distance":null,"gain":null,"environment":null}"#,
                        ),
                    },
                },
                claim: PracticeClaim::Reference,
                match_evidence: vec![],
                analysis: BTreeMap::new(),
                advice: BTreeMap::new(),
                reflections: vec![],
                trashed_at: None,
            };
            trial
                .validate()
                .expect("valid handwritten trial before invoking reader");
            let trial_ref = put(&host, &serde_json::to_vec(&trial).unwrap());
            let document = HistoryRoot {
                schema_version: 1,
                revision: 1,
                next_save_sequence: 2,
                trials: BTreeMap::from([(
                    trial.id.clone(),
                    TrialHead {
                        document: trial_ref.clone(),
                        admitted_sequence: 1,
                    },
                )]),
                receipts: BTreeMap::new(),
            };
            document
                .validate()
                .expect("valid handwritten root before invoking reader");
            let root = RootRef::new(put(&host, &serde_json::to_vec(&document).unwrap()));
            let mut objects = vec![root.as_blob().clone(), trial_ref, original.clone()];
            objects.sort_by(|a, b| a.sha256.cmp(&b.sha256));
            let expected = VerifiedClosure {
                total_bytes: objects.iter().map(|r| r.bytes).sum(),
                objects,
            };
            Self {
                temp,
                host,
                links: vec![],
                trial,
                original,
                root,
                expected,
            }
        }

        fn junction(&mut self, link: &Path, target: &Path) {
            assert!(link.starts_with(&self.temp) && target.starts_with(&self.temp));
            assert!(!link.exists());
            self.links.push(link.to_owned());
            fs::create_dir(link).unwrap();
            create_junction(link, target).expect("junction setup failure is not a reader RED");
            assert_ne!(
                fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
                0
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // Unlink entries before recursive cleanup; never follow a junction.
            for link in &self.links {
                if let Err(error) = fs::remove_dir(link) {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        eprintln!(
                            "preserving fixture after unlink failure: {}: {error}",
                            self.temp.display()
                        );
                        return;
                    }
                }
            }
            if let Err(error) = fs::remove_dir_all(&self.temp) {
                eprintln!("fixture cleanup failed: {}: {error}", self.temp.display());
            }
        }
    }

    fn digest(bytes: &[u8]) -> Sha256 {
        Sha256::parse(&crate::assets::sha256(bytes).unwrap()).unwrap()
    }

    fn blob_path(host: &Path, reference: &BlobRef) -> PathBuf {
        host.join("audio-history/assets")
            .join(format!("{}.bin", reference.sha256.as_str()))
    }

    fn put(host: &Path, bytes: &[u8]) -> BlobRef {
        let reference = BlobRef {
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
        };
        fs::write(blob_path(host, &reference), bytes).unwrap();
        reference
    }

    fn tree(base: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(base: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                let metadata = fs::symlink_metadata(&path).unwrap();
                let relative = path.strip_prefix(base).unwrap().to_owned();
                if metadata.file_attributes() & 0x400 != 0 {
                    out.insert(relative, b"REPARSE: do not traverse".to_vec());
                } else if metadata.is_dir() {
                    out.insert(relative, vec![]);
                    visit(base, &path, out);
                } else {
                    out.insert(relative, fs::read(path).unwrap());
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(base, base, &mut result);
        result
    }

    fn assert_kind<T>(result: Result<T, HistoryError>, expected: HistoryErrorKind) {
        match result {
            Err(error) => assert_eq!(error.kind, expected),
            Ok(_) => panic!("expected {expected:?}, unexpectedly succeeded"),
        }
    }

    fn assert_storage_fixture(f: &Fixture, storage: &Storage) {
        let reader = HistoryReader::new(storage);
        assert_eq!(reader.verify_closure(&f.root).unwrap(), f.expected);
        assert_eq!(reader.read_original(&f.original).unwrap(), RAW);
        assert_eq!(
            reader.read_snapshot(Some(&f.root)).trials[0].document,
            f.trial
        );
    }

    #[test]
    fn handwritten_three_object_fixture_passes_existing_storage_entry() {
        let f = Fixture::new();
        let storage = Storage::at(f.host.clone()).unwrap();
        assert_storage_fixture(&f, &storage);
    }

    #[test]
    fn path_and_storage_entries_match_independent_closure_while_lock_is_held() {
        let f = Fixture::new();
        {
            Storage::at(f.host.clone()).unwrap();
        }
        let before = tree(&f.temp);
        let result = {
            let storage = Storage::at(f.host.clone()).unwrap();
            assert_storage_fixture(&f, &storage);
            eprintln!("Storage baseline passed: exact 3-object closure, original and trial; now Path entry");
            let existing = HistoryReader::with_limits(&storage, HistoryLimits::v1()).unwrap();
            assert_eq!(existing.verify_closure(&f.root).unwrap(), f.expected);
            HistoryReader::for_root(&f.host, HistoryLimits::v1()).map(|reader| {
                assert_eq!(reader.verify_closure(&f.root).unwrap(), f.expected);
                assert_eq!(reader.read_original(&f.original).unwrap(), RAW);
                assert_eq!(
                    reader.read_snapshot(Some(&f.root)),
                    existing.read_snapshot(Some(&f.root))
                );
                assert_eq!(
                    reader.read_page(Some(&f.root), Some(&f.trial.id)),
                    existing.read_page(Some(&f.root), Some(&f.trial.id))
                );
            })
        };
        assert_eq!(tree(&f.temp), before);
        result.expect("Path constructor must accept valid limits after Storage baseline succeeded");
    }

    #[test]
    fn path_limits_accept_exact_closure_and_reject_one_less() {
        let f = Fixture::new();
        let storage = Storage::at(f.host.clone()).unwrap();
        assert_storage_fixture(&f, &storage);
        let exact = HistoryLimits {
            max_nodes: 3,
            max_total_bytes: f.expected.total_bytes,
        };
        assert_eq!(
            HistoryReader::with_limits(&storage, exact)
                .unwrap()
                .verify_closure(&f.root)
                .unwrap(),
            f.expected
        );
        let reader = HistoryReader::for_root(&f.host, exact).expect("valid exact Path limits");
        assert_eq!(reader.verify_closure(&f.root).unwrap(), f.expected);
        for limits in [
            HistoryLimits {
                max_nodes: 2,
                ..exact
            },
            HistoryLimits {
                max_total_bytes: exact.max_total_bytes - 1,
                ..exact
            },
        ] {
            assert_kind(
                HistoryReader::for_root(&f.host, limits)
                    .unwrap()
                    .verify_closure(&f.root),
                HistoryErrorKind::LimitExceeded,
            );
            assert_kind(
                HistoryReader::with_limits(&storage, limits)
                    .unwrap()
                    .verify_closure(&f.root),
                HistoryErrorKind::LimitExceeded,
            );
        }
        assert!(HistoryReader::for_root(
            &f.host,
            HistoryLimits {
                max_nodes: 100_000,
                max_total_bytes: 3 * 1024 * 1024 * 1024
            }
        )
        .is_ok());
    }

    #[test]
    fn invalid_path_limits_fail_without_creating_the_absent_root() {
        let f = Fixture::new();
        let absent = f.temp.join("absent-host");
        let before = tree(&f.temp);
        for limits in [
            HistoryLimits {
                max_nodes: 0,
                max_total_bytes: 1,
            },
            HistoryLimits {
                max_nodes: 100_001,
                max_total_bytes: u64::MAX,
            },
            HistoryLimits {
                max_nodes: 1,
                max_total_bytes: 0,
            },
        ] {
            assert_kind(
                HistoryReader::for_root(&absent, limits),
                HistoryErrorKind::InvalidDocument,
            );
        }
        assert_eq!(tree(&f.temp), before);
        assert!(!absent.exists());
    }

    #[test]
    fn missing_root_is_repeatable_diagnostic_failure_without_repair() {
        let f = Fixture::new();
        {
            let storage = Storage::at(f.host.clone()).unwrap();
            assert_storage_fixture(&f, &storage);
        }
        fs::remove_file(blob_path(&f.host, f.root.as_blob())).unwrap();
        let before = tree(&f.temp);
        let result = HistoryReader::for_root(&f.host, HistoryLimits::v1()).map(|reader| {
            for _ in 0..2 {
                assert_kind(
                    reader.verify_closure(&f.root),
                    HistoryErrorKind::MissingObject,
                );
                let view = reader.read_snapshot(Some(&f.root));
                assert!(view.root.is_none() && view.trials.is_empty());
                assert_eq!(
                    view.diagnostics,
                    vec![HistoryDiagnostic {
                        trial: None,
                        error: HistoryError {
                            kind: HistoryErrorKind::MissingObject
                        }
                    }]
                );
            }
        });
        assert_eq!(tree(&f.temp), before);
        result.expect("Path constructor must not replace a read-time missing-root diagnostic");
    }

    #[test]
    fn missing_original_preserves_trial_and_repeated_partial_view() {
        let f = Fixture::new();
        {
            let storage = Storage::at(f.host.clone()).unwrap();
            assert_storage_fixture(&f, &storage);
        }
        fs::remove_file(blob_path(&f.host, &f.original)).unwrap();
        let before = tree(&f.temp);
        let result = {
            let storage = Storage::at(f.host.clone()).unwrap();
            let existing = HistoryReader::new(&storage).read_snapshot(Some(&f.root));
            HistoryReader::for_root(&f.host, HistoryLimits::v1()).map(|reader| {
                for _ in 0..2 {
                    assert_kind(
                        reader.verify_closure(&f.root),
                        HistoryErrorKind::MissingObject,
                    );
                    let view = reader.read_snapshot(Some(&f.root));
                    assert_eq!(view, existing);
                    assert!(view.root.is_some());
                    assert_eq!(view.trials.len(), 1);
                    assert_eq!(view.trials[0].document, f.trial);
                    assert!(!view.trials[0].original_available);
                    assert_eq!(
                        view.trials[0].diagnostics,
                        vec![HistoryDiagnostic {
                            trial: Some(f.trial.id.clone()),
                            error: HistoryError {
                                kind: HistoryErrorKind::MissingObject
                            }
                        }]
                    );
                }
            })
        };
        assert_eq!(tree(&f.temp), before);
        result.expect("Path constructor must allow partial browsing");
    }

    #[test]
    fn path_only_read_never_creates_storage_lock_or_missing_directories() {
        let f = Fixture::new();
        let before = tree(&f.temp);
        let result = HistoryReader::for_root(&f.host, HistoryLimits::v1()).map(|reader| {
            assert_eq!(reader.verify_closure(&f.root).unwrap(), f.expected);
        });
        assert_eq!(tree(&f.temp), before);
        assert!(!f.host.join("app.lock").exists());
        result.expect("Path reading needs no newly acquired Storage");
        let absent = f.temp.join("missing-parent/host");
        let reader = HistoryReader::for_root(&absent, HistoryLimits::v1()).unwrap();
        assert!(reader.read_snapshot(None).diagnostics.is_empty());
        for _ in 0..2 {
            assert_kind(
                reader.verify_closure(&f.root),
                HistoryErrorKind::MissingObject,
            );
        }
        assert_eq!(tree(&f.temp), before);
        assert!(!absent.exists());
    }

    // Same ordinary-permission junction mechanism as audio_history_closure.
    // No file symlinks, elevation, shell, credentials, or Windows setting changes.
    fn create_junction(link: &Path, target: &Path) -> std::io::Result<()> {
        use std::{ffi::c_void, ptr};
        #[link(name = "kernel32")]
        extern "system" {
            fn DeviceIoControl(
                device: *mut c_void,
                code: u32,
                input: *const c_void,
                input_len: u32,
                output: *mut c_void,
                output_len: u32,
                returned: *mut u32,
                overlapped: *mut c_void,
            ) -> i32;
        }
        let canonical = fs::canonicalize(target)?;
        let canonical: Vec<u16> = canonical.as_os_str().encode_wide().collect();
        let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
        assert!(canonical.starts_with(&prefix));
        let print_name = &canonical[prefix.len()..];
        let substitute: Vec<u16> = r"\??\"
            .encode_utf16()
            .chain(print_name.iter().copied())
            .collect();
        let sub_bytes = u16::try_from(substitute.len() * 2).unwrap();
        let print_bytes = u16::try_from(print_name.len() * 2).unwrap();
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&0xa000_0003_u32.to_le_bytes());
        buffer.extend_from_slice(&(8 + sub_bytes + 2 + print_bytes + 2).to_le_bytes());
        for field in [0_u16, 0, sub_bytes, sub_bytes + 2, print_bytes] {
            buffer.extend_from_slice(&field.to_le_bytes());
        }
        for value in substitute
            .iter()
            .copied()
            .chain([0])
            .chain(print_name.iter().copied())
            .chain([0])
        {
            buffer.extend_from_slice(&value.to_le_bytes());
        }
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(0x0200_0000 | 0x0020_0000)
            .open(link)?;
        let mut returned = 0;
        // SAFETY: the owned handle and input buffer remain valid for this
        // synchronous call; no output buffer or OVERLAPPED pointer is used.
        let success = unsafe {
            DeviceIoControl(
                file.as_raw_handle(),
                0x0009_00a4,
                buffer.as_ptr().cast(),
                u32::try_from(buffer.len()).unwrap(),
                ptr::null_mut(),
                0,
                &mut returned,
                ptr::null_mut(),
            )
        };
        if success == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    fn junction_case(placement: u8) {
        let mut f = Fixture::new();
        {
            let storage = Storage::at(f.host.clone()).unwrap();
            assert_storage_fixture(&f, &storage);
        }
        let target = f.temp.join("junction-target");
        let exposed_host = match placement {
            0 => {
                let link = f.temp.join("linked-parent");
                let parent = f.temp.join("parent");
                f.junction(&link, &parent);
                link.join("host")
            }
            1 => {
                let link = f.temp.join("linked-host");
                let host = f.host.clone();
                f.junction(&link, &host);
                link
            }
            2 | 3 => {
                let link = if placement == 2 {
                    f.host.join("audio-history")
                } else {
                    f.host.join("audio-history/assets")
                };
                fs::rename(&link, &target).unwrap();
                f.junction(&link, &target);
                f.host.clone()
            }
            _ => unreachable!(),
        };
        assert_eq!(
            fs::read(blob_path(&exposed_host, &f.original)).unwrap(),
            RAW,
            "junction resolves to valid fixture bytes"
        );
        let before = tree(&f.temp);
        let result = HistoryReader::for_root(&exposed_host, HistoryLimits::v1()).map(|reader| {
            for _ in 0..2 {
                assert_kind(
                    reader.read_original(&f.original),
                    HistoryErrorKind::UnsafePath,
                );
                assert_kind(reader.verify_closure(&f.root), HistoryErrorKind::UnsafePath);
            }
        });
        assert_eq!(tree(&f.temp), before);
        result.expect("valid Path constructor precedes unsafe read-time rejection");
    }

    #[test]
    fn path_ancestor_junction_rejected_without_target_mutation() {
        junction_case(0);
    }
    #[test]
    fn path_root_junction_rejected_without_target_mutation() {
        junction_case(1);
    }
    #[test]
    fn path_history_junction_rejected_without_target_mutation() {
        junction_case(2);
    }
    #[test]
    fn path_assets_junction_rejected_without_target_mutation() {
        junction_case(3);
    }
}
