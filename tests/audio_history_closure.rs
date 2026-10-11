//! I1b-1 independent reader contract. Objects are handwritten, never emitted by
//! a history writer/analyzer. No default root, credentials, device or network.
#![cfg(windows)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use acoustic_analysis::{
    AnalysisProfile, AnalysisReport, AnalysisStatus, Measures, MissingReason, PitchMeasure,
    SampleValue, TimedValue,
};
use serde::Serialize;
use serde_json::json;
use wordweave5::{
    assets::{self, AssetKind, AssetStore},
    audio_history::*,
    store::Storage,
};

const MIB: u64 = 1024 * 1024;
const RAW: &[u8] = b"synthetic original bytes, deliberately not decoded by the reader";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "ww5-history-reader-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("unique isolated test root");
        Self(dir)
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        // Only the uniquely created test root is removed, after Storage's lock.
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Fixture {
    storage: Storage,
    temp: TempRoot,
    original: BlobRef,
}
impl Fixture {
    fn new() -> Self {
        let temp = TempRoot::new();
        let storage = Storage::at(temp.0.join("host")).unwrap();
        let reference = AssetStore::new(storage.dir.join("audio-history"))
            .put(AssetKind::FileBlob, RAW)
            .unwrap();
        let original = BlobRef {
            sha256: Sha256::parse(&reference.id).unwrap(),
            bytes: reference.bytes,
        };
        Self {
            storage,
            temp,
            original,
        }
    }
    fn put_bytes(&self, bytes: &[u8]) -> BlobRef {
        let reference = AssetStore::new(self.storage.dir.join("audio-history"))
            .put(AssetKind::FileBlob, bytes)
            .unwrap();
        BlobRef {
            sha256: Sha256::parse(&reference.id).unwrap(),
            bytes: reference.bytes,
        }
    }
    fn put(&self, value: &impl Serialize) -> BlobRef {
        self.put_bytes(&serde_json::to_vec(value).unwrap())
    }
    fn path(&self, reference: &BlobRef) -> PathBuf {
        let path = self
            .storage
            .dir
            .join("audio-history/assets")
            .join(format!("{}.bin", reference.sha256.as_str()));
        assert!(path.starts_with(&self.temp.0));
        path
    }
    fn remove(&self, reference: &BlobRef) {
        fs::remove_file(self.path(reference)).unwrap();
    }
    fn trial(&self, n: u64) -> TrialRevision {
        let reference = "Read this sentence.\nKeep the exact text.";
        let capture = CaptureProfile {
            device: None,
            distance: None,
            gain: None,
            environment: None,
            confirmed_at: None,
            version: digest(br#"{"device":null,"distance":null,"gain":null,"environment":null}"#),
        };
        TrialRevision {
            schema_version: 1,
            id: trial_id(n),
            revision: 1,
            previous: None,
            created_at: at(),
            input: FrozenInput {
                audio: AudioSnapshot {
                    original: self.original.clone(),
                    format: AudioFormat::Wav,
                    rate: 8_000,
                    channels: 1,
                    frames: 1,
                    origin: Origin::Imported,
                    warnings: vec![CaptureWarning::PartialRecording],
                },
                reference: ReferenceSnapshot {
                    text: reference.into(),
                    sha256: digest(reference.as_bytes()),
                    source: None,
                },
                goal: Some(GoalSnapshot {
                    text: "Synthetic goal".into(),
                    version: digest(b"Synthetic goal"),
                }),
                capture,
            },
            claim: PracticeClaim::Reference,
            match_evidence: vec![],
            analysis: BTreeMap::new(),
            advice: BTreeMap::new(),
            reflections: vec![],
            trashed_at: None,
        }
    }
    fn root_document(&self, heads: &[(TrialId, BlobRef)]) -> HistoryRoot {
        HistoryRoot {
            schema_version: 1,
            revision: if heads.is_empty() { 0 } else { 1 },
            next_save_sequence: heads.len() as u64 + 1,
            trials: heads
                .iter()
                .enumerate()
                .map(|(i, (id, document))| {
                    (
                        id.clone(),
                        TrialHead {
                            document: document.clone(),
                            admitted_sequence: i as u64 + 1,
                        },
                    )
                })
                .collect(),
            receipts: BTreeMap::new(),
        }
    }
    fn root(&self, heads: &[(TrialId, BlobRef)]) -> RootRef {
        RootRef::new(self.put(&self.root_document(heads)))
    }
    fn single(&self, trial: &TrialRevision) -> (RootRef, BlobRef) {
        let document = self.put(trial);
        (self.root(&[(trial.id.clone(), document.clone())]), document)
    }
}
fn digest(bytes: &[u8]) -> Sha256 {
    Sha256::parse(&assets::sha256(bytes).unwrap()).unwrap()
}
fn trial_id(n: u64) -> TrialId {
    TrialId::parse(&format!("00000000-0000-4000-8000-{n:012x}")).unwrap()
}
fn run_id(n: u64) -> RunId {
    RunId::parse(&format!("11111111-1111-4111-8111-{n:012x}")).unwrap()
}
fn at() -> RecordedAt {
    RecordedAt {
        utc_millis: 1_700_000_000_000,
        offset_minutes: 540,
    }
}
fn claim() -> PracticeClaim {
    PracticeClaim::Practice {
        performed_at: at(),
        self_speaker_confirmed_at: at(),
        time_confirmed_at: at(),
        duplicate_confirmed: false,
    }
}
fn report(rate: u32, channels: u16, frames: u64) -> AnalysisReport {
    // Hand-derived short-input report: a constant -6dBFS signal, no low-energy
    // interval, F0 unavailable because the duration is below 60ms.
    let duration = frames as f64 / f64::from(rate);
    assert!(duration < 0.06);
    let mut rms_frames = Vec::new();
    let mut k = 0_u64;
    loop {
        let start = k * u64::from(rate) / 100;
        if start >= frames {
            break;
        }
        let end = (start + u64::from(rate) * 25 / 1000).min(frames);
        rms_frames.push(TimedValue {
            time_seconds: (start + end) as f64 / (2.0 * f64::from(rate)),
            value: SampleValue::Finite(-6.0),
        });
        k += 1;
    }
    AnalysisReport {
        schema_version: 1,
        metrics_version: "acoustic-metrics-v1".into(),
        library_version: "0.1.0".into(),
        profile: AnalysisProfile::v1(),
        sample_rate: rate,
        channels,
        input_frames: frames,
        pitch_engine: None,
        status: AnalysisStatus::Partial,
        measures: Measures {
            duration_seconds: duration,
            rms_dbfs: SampleValue::Finite(-6.0),
            rms_frames,
            low_intervals: vec![],
            low_total_seconds: 0.0,
            low_fraction: 0.0,
            pitch: PitchMeasure {
                frames: vec![],
                median_hz: SampleValue::Missing(MissingReason::TooShort),
                p10_hz: SampleValue::Missing(MissingReason::TooShort),
                p90_hz: SampleValue::Missing(MissingReason::TooShort),
                valid_frame_fraction: SampleValue::Missing(MissingReason::TooShort),
            },
        },
        quality: vec![],
    }
}
fn analysis(trial: &TrialRevision, n: u64, result: Option<BlobRef>) -> AnalysisRun {
    let terminal = result.is_some();
    AnalysisRun {
        schema_version: 1,
        id: run_id(n),
        trial: trial.id.clone(),
        audio: trial.input.audio.original.sha256.clone(),
        pcm: digest(b"synthetic PCM identity"),
        decoder: DecoderIdentity {
            name: "fixture".into(),
            version: "1".into(),
            binary_sha256: None,
        },
        profile: AnalysisProfile::v1(),
        state: if terminal {
            AnalysisRunState::Partial
        } else {
            AnalysisRunState::Running
        },
        started_at: at(),
        ended_at: terminal.then(at),
        result,
        failure: None,
    }
}
fn feedback() -> AdviceResult {
    AdviceResult {
        schema_version: 1,
        assessment: AdviceAssessment::Assessed,
        heard_text: Some("Read this sentence.".into()),
        summary: "Synthetic feedback".into(),
        strengths: vec![],
        improvements: vec![AdviceImprovement {
            reference_excerpt: "this sentence".into(),
            observation: "Synthetic observation".into(),
            practice: "Repeat once".into(),
        }],
        unassessable_reason: None,
        requested_model: "fixture".into(),
        requested_effort: "fixture".into(),
        actual_model: None,
        actual_effort: None,
        usage: None,
    }
}
fn advice(trial: &TrialRevision, n: u64, result: BlobRef) -> AdviceRun {
    AdviceRun {
        schema_version: 1,
        id: run_id(n),
        trial: trial.id.clone(),
        audio: trial.input.audio.original.sha256.clone(),
        reference: trial.input.reference.sha256.clone(),
        confirmed_at: at(),
        confirmation: digest(b"synthetic confirmation"),
        destination: "https://fixture.invalid".into(),
        versions: AdviceVersions {
            provider: "fixture".into(),
            prompt: "1".into(),
            parser: "1".into(),
        },
        state: AdviceRunState::Assessed,
        send: SendCertainty::ResponseReceived,
        started_at: at(),
        ended_at: Some(at()),
        result: Some(result),
        failure: None,
    }
}
fn expect_kind<T: std::fmt::Debug>(value: Result<T, HistoryError>, kind: HistoryErrorKind) {
    assert_eq!(value.unwrap_err().kind, kind);
}
fn expected(refs: &[BlobRef]) -> (Vec<BlobRef>, u64) {
    let sorted: BTreeMap<_, _> = refs.iter().map(|r| (r.sha256.clone(), r.clone())).collect();
    let objects: Vec<_> = sorted.into_values().collect();
    let bytes = objects.iter().map(|r| r.bytes).sum();
    (objects, bytes)
}
fn tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.insert(path.strip_prefix(base).unwrap().to_owned(), vec![]);
                visit(base, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(path, path, &mut result);
    result
}

#[test]
fn handwritten_fixture_is_valid_before_invoking_reader() {
    let f = Fixture::new();
    let t = f.trial(1);
    t.validate().unwrap();
    report(8_000, 1, 1).validate().unwrap();
    feedback().validate().unwrap();
    analysis(&t, 1, None).validate().unwrap();
    let result = f.put(&feedback());
    advice(&t, 2, result).validate().unwrap();
    f.root_document(&[(t.id.clone(), f.put(&t))])
        .validate()
        .unwrap();
    assert_eq!(assets::sha256(RAW).unwrap(), f.original.sha256.as_str());
}

#[test]
fn hash_helpers_use_exact_utf8_and_fixed_capture_order_without_confirmation_time() {
    let f = Fixture::new();
    let t = f.trial(1);
    let text = "  A\nＢ🙂  ";
    assert_eq!(hash_reference(text).unwrap(), digest(text.as_bytes()));
    assert_eq!(hash_goal(text).unwrap(), digest(text.as_bytes()));
    assert_eq!(
        hash_capture(&t.input.capture).unwrap(),
        t.input.capture.version
    );
    let mut capture = t.input.capture;
    capture.confirmed_at = Some(at());
    capture.version = digest(b"not part of hash");
    assert_eq!(
        hash_capture(&capture).unwrap(),
        digest(br#"{"device":null,"distance":null,"gain":null,"environment":null}"#)
    );
    capture.device = Some("synthetic device".into());
    assert_eq!(
        hash_capture(&capture).unwrap(),
        digest(br#"{"device":"synthetic device","distance":null,"gain":null,"environment":null}"#)
    );
}

#[test]
fn no_root_is_empty_without_creating_history_or_mutating_storage() {
    let temp = TempRoot::new();
    let storage = Storage::at(temp.0.join("host")).unwrap();
    // Windows locks prevent reading app.lock itself. Compare all bytes with
    // the lock released on both sides; retain it throughout reader operations.
    drop(storage);
    let before = tree(&temp.0);
    {
        let storage = Storage::at(temp.0.join("host")).unwrap();
        let reader = HistoryReader::new(&storage);
        let view = reader.read_snapshot(None);
        assert_eq!(view, reader.read_page(None, None));
        assert!(
            view.root.is_none()
                && view.trials.is_empty()
                && view.diagnostics.is_empty()
                && view.next_after.is_none()
        );
    }
    assert_eq!(tree(&temp.0), before);
}

#[test]
fn strict_closure_includes_old_revision_results_root_and_exact_unique_byte_total() {
    let f = Fixture::new();
    let mut old = f.trial(1);
    let old_result = f.put(&report(8_000, 1, 1));
    let old_run = analysis(&old, 1, Some(old_result.clone()));
    let old_run_ref = f.put(&old_run);
    old.analysis.insert(old_run.id.clone(), old_run_ref.clone());
    let old_ref = f.put(&old);
    let mut current = old.clone();
    current.revision = 2;
    current.previous = Some(old_ref.clone());
    current.analysis.clear();
    let mut second_result = report(8_000, 1, 1);
    second_result.library_version = "0.1.1".into();
    let second_result_ref = f.put(&second_result);
    let second_run = analysis(&current, 2, Some(second_result_ref.clone()));
    let second_run_ref = f.put(&second_run);
    current
        .analysis
        .insert(second_run.id.clone(), second_run_ref.clone());
    let advice_result = f.put(&feedback());
    let advice_run = advice(&current, 3, advice_result.clone());
    let advice_ref = f.put(&advice_run);
    current
        .advice
        .insert(advice_run.id.clone(), advice_ref.clone());
    current.match_evidence.push(MatchEvidence::AiAssessed {
        run: advice_run.id.clone(),
    });
    let note = Reflection {
        schema_version: 1,
        trial: current.id.clone(),
        at: at(),
        text: "Synthetic reflection".into(),
    };
    let note_ref = f.put(&note);
    current.reflections.push(note_ref.clone());
    let (root, current_ref) = f.single(&current);
    f.put_bytes(b"unreferenced orphan must not enter the closure");
    let (objects, total_bytes) = expected(&[
        root.as_blob().clone(),
        f.original.clone(),
        old_ref,
        old_run_ref,
        old_result,
        current_ref,
        second_run_ref,
        second_result_ref,
        advice_ref,
        advice_result,
        note_ref,
    ]);
    let reader = HistoryReader::new(&f.storage);
    assert_eq!(
        reader.verify_closure(&root).unwrap(),
        VerifiedClosure {
            objects,
            total_bytes
        }
    );
    assert_eq!(
        reader.read_analysis(&current, &second_run.id).unwrap(),
        AnalysisView {
            run: second_run,
            result: Some(second_result)
        }
    );
    assert_eq!(
        reader.read_advice(&current, &advice_run.id).unwrap(),
        AdviceView {
            run: advice_run,
            result: Some(feedback())
        }
    );
    assert_eq!(
        reader
            .read_reflection(&current, &current.reflections[0])
            .unwrap(),
        note
    );
}

#[test]
fn shared_original_is_counted_once_across_two_trial_heads() {
    let f = Fixture::new();
    let first = f.trial(1);
    let second = f.trial(2);
    let a = f.put(&first);
    let b = f.put(&second);
    let root = f.root(&[(first.id, a.clone()), (second.id, b.clone())]);
    let (objects, total_bytes) = expected(&[root.as_blob().clone(), a, b, f.original.clone()]);
    assert_eq!(
        HistoryReader::new(&f.storage)
            .verify_closure(&root)
            .unwrap(),
        VerifiedClosure {
            objects,
            total_bytes
        }
    );
}

#[test]
fn missing_root_is_a_diagnostic_failure_not_an_empty_saved_history() {
    let f = Fixture::new();
    let (root, _) = f.single(&f.trial(1));
    f.remove(root.as_blob());
    let reader = HistoryReader::new(&f.storage);
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::MissingObject,
    );
    let view = reader.read_snapshot(Some(&root));
    assert!(view.root.is_none() && view.trials.is_empty());
    assert!(!view.diagnostics.is_empty());
    assert!(view
        .diagnostics
        .iter()
        .any(|d| d.error.kind == HistoryErrorKind::MissingObject));
}

#[test]
fn original_reads_return_exact_bytes_and_detect_same_length_hash_damage() {
    let f = Fixture::new();
    let reader = HistoryReader::new(&f.storage);
    assert_eq!(reader.read_original(&f.original).unwrap(), RAW);
    fs::write(f.path(&f.original), vec![b'x'; RAW.len()]).unwrap();
    expect_kind(
        reader.read_original(&f.original),
        HistoryErrorKind::DamagedObject,
    );
}

#[test]
fn original_length_damage_is_not_hidden_by_reference_metadata() {
    let f = Fixture::new();
    fs::write(f.path(&f.original), b"truncated").unwrap();
    expect_kind(
        HistoryReader::new(&f.storage).read_original(&f.original),
        HistoryErrorKind::DamagedObject,
    );
}

#[test]
fn missing_original_preserves_trial_text_and_readable_individual_results() {
    let f = Fixture::new();
    let mut t = f.trial(1);
    let result = report(8_000, 1, 1);
    let run = analysis(&t, 1, Some(f.put(&result)));
    t.analysis.insert(run.id.clone(), f.put(&run));
    let ar = advice(&t, 2, f.put(&feedback()));
    t.advice.insert(ar.id.clone(), f.put(&ar));
    let note = Reflection {
        schema_version: 1,
        trial: t.id.clone(),
        at: at(),
        text: "Preserved note".into(),
    };
    let note_ref = f.put(&note);
    t.reflections.push(note_ref.clone());
    let (root, _) = f.single(&t);
    f.remove(&f.original);
    let reader = HistoryReader::new(&f.storage);
    let view = reader.read_snapshot(Some(&root));
    assert_eq!(view.trials.len(), 1);
    assert_eq!(view.trials[0].document, t);
    assert!(!view.trials[0].original_available && !view.trials[0].diagnostics.is_empty());
    assert_eq!(
        reader.read_analysis(&t, &run.id).unwrap().result,
        Some(result)
    );
    assert_eq!(
        reader.read_advice(&t, &ar.id).unwrap().result,
        Some(feedback())
    );
    assert_eq!(reader.read_reflection(&t, &note_ref).unwrap(), note);
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::MissingObject,
    );
}

#[test]
fn one_broken_trial_does_not_hide_a_healthy_neighbor() {
    let f = Fixture::new();
    let bad = f.trial(1);
    let good = f.trial(2);
    let bad_ref = f.put(&bad);
    let good_ref = f.put(&good);
    let root = f.root(&[
        (bad.id.clone(), bad_ref.clone()),
        (good.id.clone(), good_ref),
    ]);
    f.remove(&bad_ref);
    let view = HistoryReader::new(&f.storage).read_snapshot(Some(&root));
    assert_eq!(view.trials.len(), 1);
    assert_eq!(view.trials[0].document, good);
    assert!(view
        .diagnostics
        .iter()
        .any(|d| d.trial.as_ref() == Some(&bad.id)));
}

#[test]
fn damaged_previous_revision_does_not_discard_the_newest_trial_body() {
    let f = Fixture::new();
    let old = f.trial(1);
    let old_ref = f.put(&old);
    let mut current = old;
    current.revision = 2;
    current.previous = Some(old_ref.clone());
    let (root, _) = f.single(&current);
    f.remove(&old_ref);
    let reader = HistoryReader::new(&f.storage);
    let view = reader.read_snapshot(Some(&root));
    assert_eq!(view.trials.len(), 1);
    assert_eq!(view.trials[0].document, current);
    assert!(!view.trials[0].diagnostics.is_empty());
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::MissingObject,
    );
}

#[test]
fn node_and_total_byte_limits_accept_exact_closure_and_reject_one_less() {
    let f = Fixture::new();
    let (root, t_ref) = f.single(&f.trial(1));
    let (_, bytes) = expected(&[root.as_blob().clone(), t_ref, f.original.clone()]);
    let exact = HistoryReader::with_limits(
        &f.storage,
        HistoryLimits {
            max_nodes: 3,
            max_total_bytes: bytes,
        },
    )
    .unwrap();
    assert_eq!(exact.verify_closure(&root).unwrap().total_bytes, bytes);
    for limits in [
        HistoryLimits {
            max_nodes: 2,
            max_total_bytes: bytes,
        },
        HistoryLimits {
            max_nodes: 3,
            max_total_bytes: bytes - 1,
        },
    ] {
        expect_kind(
            HistoryReader::with_limits(&f.storage, limits)
                .unwrap()
                .verify_closure(&root),
            HistoryErrorKind::LimitExceeded,
        );
    }
}

#[test]
fn invalid_limits_are_rejected_and_node_cap_cannot_be_raised_by_quota() {
    let f = Fixture::new();
    assert_eq!(
        HistoryLimits::v1(),
        HistoryLimits {
            max_nodes: 100_000,
            max_total_bytes: 2 * 1024 * MIB
        }
    );
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
        assert!(HistoryReader::with_limits(&f.storage, limits).is_err());
    }
    assert!(HistoryReader::with_limits(
        &f.storage,
        HistoryLimits {
            max_nodes: 100_000,
            max_total_bytes: 3 * 1024 * MIB
        }
    )
    .is_ok());
}

#[test]
fn individual_caps_reject_oversized_references_before_missing_object_io() {
    let f = Fixture::new();
    let reader = HistoryReader::new(&f.storage);
    let missing = digest(b"not published");
    expect_kind(
        reader.read_original(&BlobRef {
            sha256: missing.clone(),
            bytes: 6 * MIB + 1,
        }),
        HistoryErrorKind::LimitExceeded,
    );
    expect_kind(
        reader.verify_closure(&RootRef::new(BlobRef {
            sha256: missing.clone(),
            bytes: 8 * MIB + 1,
        })),
        HistoryErrorKind::LimitExceeded,
    );
    let t = f.trial(1);
    let root = f.root(&[(
        t.id.clone(),
        BlobRef {
            sha256: missing.clone(),
            bytes: 256 * 1024 + 1,
        },
    )]);
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::LimitExceeded,
    );
    let mut t = t;
    let run = analysis(
        &t,
        1,
        Some(BlobRef {
            sha256: missing,
            bytes: 2 * MIB + 1,
        }),
    );
    t.analysis.insert(run.id.clone(), f.put(&run));
    expect_kind(
        reader.read_analysis(&t, &run.id),
        HistoryErrorKind::LimitExceeded,
    );
}

#[test]
fn original_cap_itself_is_accepted_without_attempting_audio_decode() {
    let f = Fixture::new();
    let bytes = vec![0x5a; (6 * MIB) as usize];
    let reference = f.put_bytes(&bytes);
    assert_eq!(
        HistoryReader::new(&f.storage)
            .read_original(&reference)
            .unwrap(),
        bytes
    );
}

#[test]
fn same_hash_references_with_conflicting_lengths_are_rejected() {
    let f = Fixture::new();
    let first = f.trial(1);
    let mut second = f.trial(2);
    second.input.audio.original.bytes += 1;
    let root = f.root(&[
        (first.id.clone(), f.put(&first)),
        (second.id.clone(), f.put(&second)),
    ]);
    assert!(HistoryReader::new(&f.storage)
        .verify_closure(&root)
        .is_err());
}

#[test]
fn unknown_root_and_trial_schema_versions_fail_closed() {
    let f = Fixture::new();
    let t = f.trial(1);
    let t_ref = f.put(&t);
    let mut r = f.root_document(&[(t.id.clone(), t_ref)]);
    r.schema_version = 2;
    let reader = HistoryReader::new(&f.storage);
    expect_kind(
        reader.verify_closure(&RootRef::new(f.put(&r))),
        HistoryErrorKind::UnsupportedVersion,
    );
    let mut t = t;
    t.schema_version = 2;
    let (root, _) = f.single(&t);
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::UnsupportedVersion,
    );
}

#[test]
fn unknown_fields_in_root_trial_and_analysis_report_are_not_dropped() {
    let f = Fixture::new();
    let t = f.trial(1);
    let reader = HistoryReader::new(&f.storage);
    let mut root = serde_json::to_value(f.root_document(&[(t.id.clone(), f.put(&t))])).unwrap();
    root["raw_response"] = json!("synthetic secret");
    expect_kind(
        reader.verify_closure(&RootRef::new(f.put(&root))),
        HistoryErrorKind::InvalidDocument,
    );
    let mut value = serde_json::to_value(&t).unwrap();
    value["future_field"] = json!(true);
    let root = f.root(&[(t.id.clone(), f.put(&value))]);
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::InvalidDocument,
    );
    let mut t = t;
    let mut report = serde_json::to_value(report(8_000, 1, 1)).unwrap();
    report["measures"]["future_field"] = json!(true);
    let run = analysis(&t, 1, Some(f.put(&report)));
    t.analysis.insert(run.id.clone(), f.put(&run));
    expect_kind(
        reader.read_analysis(&t, &run.id),
        HistoryErrorKind::InvalidDocument,
    );
}

#[test]
fn root_map_key_must_match_the_referenced_trial_id() {
    let f = Fixture::new();
    let t = f.trial(1);
    let root = f.root(&[(trial_id(2), f.put(&t))]);
    expect_kind(
        HistoryReader::new(&f.storage).verify_closure(&root),
        HistoryErrorKind::RelationshipMismatch,
    );
}

#[test]
fn frozen_reference_goal_and_capture_hashes_are_verified_against_content() {
    let f = Fixture::new();
    for which in 0..3 {
        let mut t = f.trial(1);
        match which {
            0 => t.input.reference.sha256 = digest(b"different"),
            1 => t.input.goal.as_mut().unwrap().version = digest(b"different"),
            _ => t.input.capture.version = digest(b"different"),
        }
        t.validate().unwrap();
        let (root, _) = f.single(&t);
        expect_kind(
            HistoryReader::new(&f.storage).verify_closure(&root),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn analysis_run_map_id_trial_and_audio_must_match_the_owning_trial() {
    let f = Fixture::new();
    for which in 0..3 {
        let mut t = f.trial(1);
        let key = run_id(1);
        let mut run = analysis(&t, 1, None);
        match which {
            0 => run.id = run_id(2),
            1 => run.trial = trial_id(2),
            _ => run.audio = digest(b"different"),
        }
        run.validate().unwrap();
        t.analysis.insert(key.clone(), f.put(&run));
        expect_kind(
            HistoryReader::new(&f.storage).read_analysis(&t, &key),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn individually_valid_report_metadata_must_match_trial_and_run_state() {
    let f = Fixture::new();
    for (r, state) in [
        (report(16_000, 1, 1), AnalysisRunState::Partial),
        (report(8_000, 2, 1), AnalysisRunState::Partial),
        (report(8_000, 1, 2), AnalysisRunState::Partial),
        (report(8_000, 1, 1), AnalysisRunState::Complete),
    ] {
        r.validate().unwrap();
        let mut t = f.trial(1);
        let mut run = analysis(&t, 1, Some(f.put(&r)));
        run.state = state;
        run.validate().unwrap();
        t.analysis.insert(run.id.clone(), f.put(&run));
        expect_kind(
            HistoryReader::new(&f.storage).read_analysis(&t, &run.id),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn advice_run_input_and_identity_links_are_checked_independently() {
    let f = Fixture::new();
    for which in 0..4 {
        let mut t = f.trial(1);
        let mut run = advice(&t, 1, f.put(&feedback()));
        let key = run.id.clone();
        match which {
            0 => run.id = run_id(2),
            1 => run.trial = trial_id(2),
            2 => run.audio = digest(b"different"),
            _ => run.reference = digest(b"different"),
        }
        run.validate().unwrap();
        t.advice.insert(key.clone(), f.put(&run));
        expect_kind(
            HistoryReader::new(&f.storage).read_advice(&t, &key),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn advice_assessment_and_reference_excerpt_must_match_saved_context() {
    let f = Fixture::new();
    for which in 0..2 {
        let mut t = f.trial(1);
        let mut body = feedback();
        if which == 0 {
            body.assessment = AdviceAssessment::ReferenceMismatch;
            body.improvements.clear();
        } else {
            body.improvements[0].reference_excerpt = "not contained in the fixed reference".into();
        }
        body.validate().unwrap();
        let run = advice(&t, 1, f.put(&body));
        t.advice.insert(run.id.clone(), f.put(&run));
        expect_kind(
            HistoryReader::new(&f.storage).read_advice(&t, &run.id),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn ai_match_evidence_cannot_mislabel_a_saved_assessment() {
    let f = Fixture::new();
    let mut t = f.trial(1);
    let run = advice(&t, 1, f.put(&feedback()));
    t.advice.insert(run.id.clone(), f.put(&run));
    t.match_evidence
        .push(MatchEvidence::AiMismatch { run: run.id });
    t.validate().unwrap();
    let (root, _) = f.single(&t);
    expect_kind(
        HistoryReader::new(&f.storage).verify_closure(&root),
        HistoryErrorKind::RelationshipMismatch,
    );
}

#[test]
fn individual_reads_reject_unowned_references_before_missing_file_access() {
    let f = Fixture::new();
    let t = f.trial(1);
    let reader = HistoryReader::new(&f.storage);
    expect_kind(
        reader.read_analysis(&t, &run_id(99)),
        HistoryErrorKind::RelationshipMismatch,
    );
    expect_kind(
        reader.read_advice(&t, &run_id(99)),
        HistoryErrorKind::RelationshipMismatch,
    );
    let missing = BlobRef {
        sha256: digest(b"absent note"),
        bytes: 100,
    };
    expect_kind(
        reader.read_reflection(&t, &missing),
        HistoryErrorKind::RelationshipMismatch,
    );
}

#[test]
fn reflection_must_belong_to_the_trial_even_if_its_blob_is_listed() {
    let f = Fixture::new();
    let mut t = f.trial(1);
    let note = Reflection {
        schema_version: 1,
        trial: trial_id(2),
        at: at(),
        text: "Other trial".into(),
    };
    let reference = f.put(&note);
    t.reflections.push(reference.clone());
    expect_kind(
        HistoryReader::new(&f.storage).read_reflection(&t, &reference),
        HistoryErrorKind::RelationshipMismatch,
    );
}

#[test]
fn reference_to_confirmed_practice_is_legal_and_keeps_previous_reference_revision() {
    let f = Fixture::new();
    let old = f.trial(1);
    let old_ref = f.put(&old);
    let mut current = old;
    current.revision = 2;
    current.previous = Some(old_ref.clone());
    current.claim = claim();
    let (root, current_ref) = f.single(&current);
    let (objects, total_bytes) = expected(&[
        root.as_blob().clone(),
        current_ref,
        old_ref,
        f.original.clone(),
    ]);
    assert_eq!(
        HistoryReader::new(&f.storage)
            .verify_closure(&root)
            .unwrap(),
        VerifiedClosure {
            objects,
            total_bytes
        }
    );
}

#[test]
fn previous_chain_rejects_reverse_claim_changed_practice_time_and_frozen_input() {
    let f = Fixture::new();
    for which in 0..6 {
        let mut old = f.trial(1);
        old.claim = claim();
        let old_ref = f.put(&old);
        let mut current = old;
        current.revision = 2;
        current.previous = Some(old_ref);
        match which {
            0 => current.claim = PracticeClaim::Reference,
            1 => {
                if let PracticeClaim::Practice { performed_at, .. } = &mut current.claim {
                    performed_at.utc_millis += 1;
                }
            }
            2 => {
                if let PracticeClaim::Practice {
                    time_confirmed_at, ..
                } = &mut current.claim
                {
                    time_confirmed_at.utc_millis += 1;
                }
            }
            3 => current.created_at.utc_millis += 1,
            4 => {
                current.input.reference.text.push('!');
                current.input.reference.sha256 = digest(current.input.reference.text.as_bytes());
            }
            _ => current.input.audio.frames = 2,
        }
        current.validate().unwrap();
        let (root, _) = f.single(&current);
        expect_kind(
            HistoryReader::new(&f.storage).verify_closure(&root),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn previous_chain_requires_same_id_and_exact_predecessor_revision() {
    let f = Fixture::new();
    for which in 0..2 {
        let old = f.trial(1);
        let old_ref = f.put(&old);
        let mut current = old;
        current.revision = if which == 0 { 2 } else { 3 };
        current.previous = Some(old_ref);
        if which == 0 {
            current.id = trial_id(2);
        }
        current.validate().unwrap();
        let (root, _) = f.single(&current);
        expect_kind(
            HistoryReader::new(&f.storage).verify_closure(&root),
            HistoryErrorKind::RelationshipMismatch,
        );
    }
}

#[test]
fn old_running_intent_and_new_terminal_run_can_share_run_id_across_revisions() {
    let f = Fixture::new();
    let mut old = f.trial(1);
    let running = analysis(&old, 1, None);
    let running_ref = f.put(&running);
    old.analysis.insert(running.id.clone(), running_ref.clone());
    let old_ref = f.put(&old);
    let mut current = old;
    current.revision = 2;
    current.previous = Some(old_ref.clone());
    let result = f.put(&report(8_000, 1, 1));
    let complete = analysis(&current, 1, Some(result.clone()));
    let complete_ref = f.put(&complete);
    current
        .analysis
        .insert(complete.id.clone(), complete_ref.clone());
    let (root, current_ref) = f.single(&current);
    let (objects, total_bytes) = expected(&[
        root.as_blob().clone(),
        current_ref,
        old_ref,
        running_ref,
        complete_ref,
        result,
        f.original.clone(),
    ]);
    assert_eq!(
        HistoryReader::new(&f.storage)
            .verify_closure(&root)
            .unwrap(),
        VerifiedClosure {
            objects,
            total_bytes
        }
    );
}

#[test]
fn pagination_counts_broken_heads_and_advances_after_exactly_32_ids() {
    let f = Fixture::new();
    let mut heads = Vec::new();
    for n in 1..=33 {
        let t = f.trial(n);
        heads.push((t.id.clone(), f.put(&t)));
    }
    let broken = heads[31].1.clone();
    let root = f.root(&heads);
    f.remove(&broken);
    let reader = HistoryReader::new(&f.storage);
    let first = reader.read_snapshot(Some(&root));
    assert_eq!(first.trials.len(), 31);
    assert_eq!(first.next_after, Some(trial_id(32)));
    assert_eq!(
        first
            .trials
            .iter()
            .map(|v| v.document.id.clone())
            .collect::<Vec<_>>(),
        (1..=31).map(trial_id).collect::<Vec<_>>()
    );
    assert!(first
        .diagnostics
        .iter()
        .any(|d| d.trial == Some(trial_id(32))));
    let next = reader.read_page(Some(&root), first.next_after.as_ref());
    assert_eq!(next.trials.len(), 1);
    assert_eq!(next.trials[0].document.id, trial_id(33));
    assert_eq!(next.next_after, None);
    let invalid = reader.read_page(Some(&root), Some(&trial_id(99)));
    assert!(invalid.trials.is_empty() && !invalid.diagnostics.is_empty());
}

#[test]
fn bounded_previous_chain_reports_limit_instead_of_partial_success() {
    let f = Fixture::new();
    let mut t = f.trial(1);
    for revision in 2..=40 {
        let previous = f.put(&t);
        t.revision = revision;
        t.previous = Some(previous);
    }
    let (root, _) = f.single(&t);
    let reader = HistoryReader::with_limits(
        &f.storage,
        HistoryLimits {
            max_nodes: 10,
            max_total_bytes: MIB,
        },
    )
    .unwrap();
    expect_kind(
        reader.verify_closure(&root),
        HistoryErrorKind::LimitExceeded,
    );
    let view = reader.read_snapshot(Some(&root));
    assert!(view
        .diagnostics
        .iter()
        .chain(view.trials.iter().flat_map(|t| t.diagnostics.iter()))
        .any(|d| d.error.kind == HistoryErrorKind::LimitExceeded));
}

#[test]
fn all_reader_operations_leave_history_and_neighbor_host_bytes_unchanged() {
    let f = Fixture::new();
    let mut t = f.trial(1);
    let ar = analysis(&t, 1, Some(f.put(&report(8_000, 1, 1))));
    t.analysis.insert(ar.id.clone(), f.put(&ar));
    let qr = advice(&t, 2, f.put(&feedback()));
    t.advice.insert(qr.id.clone(), f.put(&qr));
    let note = f.put(&Reflection {
        schema_version: 1,
        trial: t.id.clone(),
        at: at(),
        text: "Keep this".into(),
    });
    t.reflections.push(note.clone());
    let (root, _) = f.single(&t);
    fs::create_dir(f.temp.0.join("other-host")).unwrap();
    fs::write(f.temp.0.join("other-host/sentinel"), b"untouched").unwrap();
    let host = f.storage.dir.clone();
    drop(f.storage);
    let before = tree(&f.temp.0);
    {
        let storage = Storage::at(host).unwrap();
        let reader = HistoryReader::new(&storage);
        reader.verify_closure(&root).unwrap();
        reader.read_snapshot(Some(&root));
        reader.read_page(Some(&root), Some(&t.id));
        reader.read_original(&f.original).unwrap();
        reader.read_analysis(&t, &ar.id).unwrap();
        reader.read_advice(&t, &qr.id).unwrap();
        reader.read_reflection(&t, &note).unwrap();
    }
    assert_eq!(tree(&f.temp.0), before);
}

// AH-AC02/03/18: valid content cannot authorize crossing a host path boundary.
// This module uses actual Windows mount-point/file-symlink reparse points.
mod windows_path_safety {
    use super::*;
    use std::os::windows::{
        ffi::OsStrExt,
        fs::{symlink_file, MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    };

    struct LinkedTemp {
        temp: Option<TempRoot>,
        links: Vec<(PathBuf, bool)>,
    }

    impl LinkedTemp {
        fn new() -> Self {
            Self {
                temp: Some(TempRoot::new()),
                links: Vec::new(),
            }
        }

        fn root(&self) -> &Path {
            &self.temp.as_ref().unwrap().0
        }

        fn junction(&mut self, link: &Path, target: &Path) {
            assert!(link.starts_with(self.root()) && target.starts_with(self.root()));
            assert!(!link.exists());
            // Register before the native call, so even a setup panic removes
            // only this directory entry, never recursively through the link.
            self.links.push((link.to_owned(), true));
            fs::create_dir(link).expect("create isolated junction entry");
            create_junction(link, target).expect("create real Windows junction fixture");
            assert_ne!(
                fs::symlink_metadata(link).unwrap().file_attributes() & 0x400,
                0,
                "fixture must be a reparse point"
            );
        }

        fn file_symlink(&mut self, link: &Path, target: &Path) {
            assert!(link.starts_with(self.root()) && target.starts_with(self.root()));
            assert!(!link.exists());
            self.links.push((link.to_owned(), false));
            symlink_file(target, link).expect(
                "create real Windows file symlink; privilege/setup failure is not a reader RED",
            );
            assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
        }

        fn unlink(&mut self) {
            for (path, directory) in &self.links {
                let result = if *directory {
                    fs::remove_dir(path)
                } else {
                    fs::remove_file(path)
                };
                result.expect("remove only the isolated reparse entry, without recursion");
            }
            self.links.clear();
        }
    }

    impl Drop for LinkedTemp {
        fn drop(&mut self) {
            let mut safe_to_clean = true;
            for (path, directory) in &self.links {
                let result = if *directory {
                    fs::remove_dir(path)
                } else {
                    fs::remove_file(path)
                };
                if let Err(error) = result {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        safe_to_clean = false;
                        eprintln!("preserving isolated fixture after unlink failure: {error}");
                    }
                }
            }
            if !safe_to_clean {
                // Avoid TempRoot's recursive cleanup when unlinking failed.
                if let Some(temp) = self.temp.take() {
                    std::mem::forget(temp);
                }
            }
        }
    }

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

        // REPARSE_DATA_BUFFER / IO_REPARSE_TAG_MOUNT_POINT. Both names
        // designate the same private, existing target; no shell or settings.
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
            .custom_flags(0x0200_0000 | 0x0020_0000) // BACKUP_SEMANTICS | OPEN_REPARSE_POINT
            .open(link)?;
        let mut returned = 0;
        // SAFETY: file owns the handle; buffer/returned remain valid through
        // this synchronous call. No output or OVERLAPPED structure is used.
        let success = unsafe {
            DeviceIoControl(
                file.as_raw_handle(),
                0x0009_00a4, // FSCTL_SET_REPARSE_POINT
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

    enum Placement {
        StorageAncestor,
        HistoryDirectory,
        AssetsDirectory,
        OriginalLeaf,
    }

    fn rejects_link_and_preserves_bytes(placement: Placement) {
        let mut fixture = LinkedTemp::new();
        let real_parent = fixture.root().join("real-parent");
        let host = real_parent.join("host");
        let history = host.join("audio-history");
        let assets = history.join("assets");
        let reference = BlobRef {
            sha256: digest(RAW),
            bytes: RAW.len() as u64,
        };
        let name = format!("{}.bin", reference.sha256.as_str());
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join(&name), RAW).unwrap();
        fs::write(
            real_parent.join("sentinel"),
            b"neighbor host stays unchanged",
        )
        .unwrap();
        {
            let storage = Storage::at(host.clone()).unwrap();
            assert_eq!(
                HistoryReader::new(&storage)
                    .read_original(&reference)
                    .unwrap(),
                RAW,
                "the exact same content is valid through a normal path"
            );
        }
        let target = fixture.root().join("target");
        let exposed_host = match placement {
            Placement::StorageAncestor => {
                let link = fixture.root().join("linked-parent");
                fixture.junction(&link, &real_parent);
                link.join("host")
            }
            Placement::HistoryDirectory => {
                fs::rename(&history, &target).unwrap();
                fixture.junction(&history, &target);
                host.clone()
            }
            Placement::AssetsDirectory => {
                fs::rename(&assets, &target).unwrap();
                fixture.junction(&assets, &target);
                host.clone()
            }
            Placement::OriginalLeaf => {
                fs::create_dir(&target).unwrap();
                let destination = target.join("original.bin");
                fs::rename(assets.join(&name), &destination).unwrap();
                fixture.file_symlink(&assets.join(&name), &destination);
                host.clone()
            }
        };
        let exposed = exposed_host.join("audio-history/assets").join(name);
        let linked_bytes = fs::read(&exposed).expect("link must resolve to the valid original");
        assert_eq!(digest(&linked_bytes), reference.sha256);
        assert_eq!(linked_bytes.len() as u64, reference.bytes);
        // Storage's exclusive app.lock is closed during each full bytes snapshot.
        let before = tree(fixture.root());
        {
            let storage = Storage::at(exposed_host).unwrap();
            let reader = HistoryReader::new(&storage);
            for _ in 0..2 {
                expect_kind(
                    reader.read_original(&reference),
                    HistoryErrorKind::UnsafePath,
                );
            }
        }
        assert_eq!(tree(fixture.root()), before);
        let real_bytes = if target.exists() {
            tree(&target)
        } else {
            tree(&real_parent)
        };
        fixture.unlink();
        assert_eq!(
            fs::read(real_parent.join("sentinel")).unwrap(),
            b"neighbor host stays unchanged"
        );
        assert_eq!(
            tree(if target.exists() {
                &target
            } else {
                &real_parent
            }),
            real_bytes,
            "unlinking the fixture must preserve every target byte"
        );
    }

    #[test]
    fn storage_ancestor_junction_rejects_valid_original_and_preserves_target() {
        rejects_link_and_preserves_bytes(Placement::StorageAncestor);
    }

    #[test]
    fn audio_history_junction_rejects_valid_original_and_preserves_target() {
        rejects_link_and_preserves_bytes(Placement::HistoryDirectory);
    }

    #[test]
    fn assets_junction_rejects_valid_original_and_preserves_target() {
        rejects_link_and_preserves_bytes(Placement::AssetsDirectory);
    }

    #[test]
    #[ignore = "Requires Windows file-symlink privilege; run this exact test elevated"]
    fn leaf_file_symlink_rejects_valid_original_and_preserves_target() {
        rejects_link_and_preserves_bytes(Placement::OriginalLeaf);
    }
}
