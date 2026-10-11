//! I5c-1 independent public-API acceptance tests (AH-AC01/05/09/10/18).
//! Oracles: approved spec and design-i5.md; never captured Praat output.
//! Windows actual-Praat gate: missing/wrong fixture FAILS, never silently skips.
#![cfg(all(feature = "praat", windows))]

use acoustic_analysis::{praat::*, *};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock},
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tokio_util::sync::CancellationToken;

const EXE_BYTES: u64 = 96_780_624;
const EXE_HASH: &str = "22c8994fd9767a2a5c1f03f21b35e41bb5ac716c628965e5d437033eeda8fff4";
const SCRIPT_HASH: &str = "3616a00415149e0ef5791f5ea497cbf523468695faa44fb059d911b6bca0876e";
const ORIGINAL: &[u8] = b"synthetic original sentinel: preserve bytes, even after errors";
static SERIAL: Mutex<()> = Mutex::new(());

// The product's process-wide lease intentionally rejects overlapping calls.
// Serialize these ordinary acceptance tests; concurrency belongs to I5c-2.
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(60)
}

fn fixture_executable() -> PathBuf {
    static VERIFIED: OnceLock<PathBuf> = OnceLock::new();
    VERIFIED
        .get_or_init(|| {
            // Explicit test-only fixture location supplied in the task handoff.
            // Never searches PATH, downloads, or changes the production pin.
            let path = std::env::var_os("WW_TEST_PRAAT_EXE")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe")
                });
            assert!(path.is_absolute(), "actual-Praat gate requires an absolute fixture path");
            let mut file = File::open(&path).expect(
                "actual-Praat gate UNVERIFIED: supply pinned local EXE with WW_TEST_PRAAT_EXE",
            );
            assert_eq!(file.metadata().unwrap().len(), EXE_BYTES, "wrong actual-Praat fixture size");
            let mut hash = Sha256::new();
            let mut chunk = [0_u8; 64 * 1024];
            loop {
                let n = file.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                hash.update(&chunk[..n]);
            }
            assert_eq!(format!("{:x}", hash.finalize()), EXE_HASH, "wrong actual-Praat fixture hash");
            path
        })
        .clone()
}

struct Fixture {
    root: TempDir,
    work: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let work = root.path().join("解析 作業 日本語");
        fs::create_dir(&work).unwrap();
        // Unknown existing work is not owned by this invocation.
        fs::create_dir(work.join("prior owner")).unwrap();
        fs::write(work.join("prior owner/keep.bin"), ORIGINAL).unwrap();
        fs::write(root.path().join("original.bin"), ORIGINAL).unwrap();
        Self { root, work }
    }

    fn config(&self, executable: PathBuf) -> PraatConfig {
        PraatConfig {
            executable,
            work_root: self.work.clone(),
        }
    }

    fn backend(&self) -> PraatBackend {
        PraatBackend::new(
            self.config(fixture_executable()),
            deadline(),
            &CancellationToken::new(),
        )
        .expect("verified fixture constructor must complete")
    }

    fn assert_preserved_and_clean(&self) {
        assert_eq!(
            fs::read(self.root.path().join("original.bin")).unwrap(),
            ORIGINAL
        );
        assert_eq!(
            fs::read(self.work.join("prior owner/keep.bin")).unwrap(),
            ORIGINAL
        );
        let entries: Vec<_> = fs::read_dir(&self.work)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(
            entries,
            vec![std::ffi::OsString::from("prior owner")],
            "owned work leaked or pre-existing work removed"
        );
        assert!(
            !take_praat_cleanup_warning(),
            "successful cleanup must not raise a warning"
        );
        assert!(
            !take_praat_cleanup_warning(),
            "warning accessor remains clear"
        );
    }
}

fn assert_error<T>(result: Result<T, AnalysisError>, expected: AnalysisErrorKind) {
    match result {
        Err(error) => assert_eq!(error.kind, expected),
        Ok(_) => panic!("expected {expected:?}, received success"),
    }
}

fn tone(rate: u32, milliseconds: usize, amplitude: f64) -> Vec<f64> {
    (0..rate as usize * milliseconds / 1000)
        .map(|i| amplitude * (std::f64::consts::TAU * 200.0 * i as f64 / f64::from(rate)).sin())
        .collect()
}

fn extract(backend: &PraatBackend, samples: &[f64], rate: u32) -> PitchTrack {
    backend
        .extract(
            MonoPcm {
                samples,
                sample_rate: rate,
            },
            &AnalysisProfile::v1().pitch,
            deadline(),
            &CancellationToken::new(),
        )
        .expect("valid synthetic float64 input must produce a pitch track")
}

fn assert_times(track: &PitchTrack, duration: f64) {
    assert!(!track.frames.is_empty());
    assert!(track.frames.len() <= 6001);
    let mut previous = -1.0;
    for frame in &track.frames {
        assert!(frame.time_seconds.is_finite());
        assert!((0.0..=duration).contains(&frame.time_seconds));
        assert!(frame.time_seconds > previous);
        previous = frame.time_seconds;
    }
}

fn assert_200_hz(track: &PitchTrack, duration: f64) {
    assert_times(track, duration);
    // Analytic oracle, deliberately excludes 100/400 Hz octave errors.
    let mut voiced = 0;
    for frame in &track.frames {
        match frame.value {
            SampleValue::Finite(hz) => {
                voiced += 1;
                assert!((195.0..=205.0).contains(&hz), "expected 200 Hz, got {hz}")
            }
            SampleValue::Missing(MissingReason::Unvoiced) => {}
            ref other => panic!("unexpected pitch value: {other:?}"),
        }
    }
    assert!(voiced > 0, "synthetic 200 Hz tone must yield voiced frames");
}

fn analyze_pcm(
    backend: &dyn PitchBackend,
    samples: &[i16],
    rate: u32,
    channels: u16,
) -> AnalysisReport {
    analyze(
        AnalysisRequest {
            pcm: PcmView::new(samples, rate, channels).unwrap(),
            profile: &AnalysisProfile::v1(),
            limits: &AnalysisLimits::v1(),
        },
        backend,
        &CancellationToken::new(),
    )
    .expect("valid core request must return a report")
}

#[test]
fn actual_fixture_gate_verifies_explicit_local_binary_and_exact_script_bytes() {
    let _serial = serial();
    fixture_executable();
    let script =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/pitch.praat")).unwrap();
    assert_eq!(format!("{:x}", Sha256::digest(&script)), SCRIPT_HASH);
    assert!(
        !script.contains(&b'\r'),
        "pinned script is LF, including on Windows"
    );
}

#[test]
fn ac01_constructor_rejects_relative_executable_without_path_fallback() {
    let _serial = serial();
    let f = Fixture::new();
    assert_error(
        PraatBackend::new(
            f.config("Praat.exe".into()),
            deadline(),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::BackendUnavailable,
    );
    f.assert_preserved_and_clean();
}

#[test]
fn ac01_constructor_rejects_absent_executable() {
    let _serial = serial();
    let f = Fixture::new();
    assert_error(
        PraatBackend::new(
            f.config(f.root.path().join("missing.exe")),
            deadline(),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::BackendUnavailable,
    );
    f.assert_preserved_and_clean();
}

#[test]
fn ac01_constructor_rejects_directory_as_executable() {
    let _serial = serial();
    let f = Fixture::new();
    assert_error(
        PraatBackend::new(
            f.config(f.work.clone()),
            deadline(),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::BackendUnavailable,
    );
    f.assert_preserved_and_clean();
}

#[test]
fn ac01_constructor_rejects_wrong_size_and_same_size_wrong_hash() {
    let _serial = serial();
    let f = Fixture::new();
    let wrong = f.root.path().join("untrusted.exe");
    fs::write(&wrong, b"MZ not the pinned executable").unwrap();
    assert_error(
        PraatBackend::new(
            f.config(wrong.clone()),
            deadline(),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::BackendUnavailable,
    );
    File::options()
        .write(true)
        .open(&wrong)
        .unwrap()
        .set_len(EXE_BYTES)
        .unwrap();
    assert_eq!(fs::metadata(&wrong).unwrap().len(), EXE_BYTES);
    assert_error(
        PraatBackend::new(f.config(wrong), deadline(), &CancellationToken::new()),
        AnalysisErrorKind::BackendUnavailable,
    );
    f.assert_preserved_and_clean();
}

#[test]
fn ac09_constructor_precancel_beats_expired_deadline_and_recovers_for_retry() {
    let _serial = serial();
    let f = Fixture::new();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert_error(
        PraatBackend::new(
            f.config(fixture_executable()),
            Instant::now() - Duration::from_secs(1),
            &cancel,
        ),
        AnalysisErrorKind::Cancelled,
    );
    f.assert_preserved_and_clean();
    let backend = f.backend();
    assert_200_hz(&extract(&backend, &tone(8000, 60, 0.4), 8000), 0.06);
    f.assert_preserved_and_clean();
}

#[test]
fn ac09_constructor_expired_deadline_is_global_error_and_recovers_for_retry() {
    let _serial = serial();
    let f = Fixture::new();
    assert_error(
        PraatBackend::new(
            f.config(fixture_executable()),
            Instant::now() - Duration::from_secs(1),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::TimedOut,
    );
    f.assert_preserved_and_clean();
    let backend = f.backend();
    assert_200_hz(&extract(&backend, &tone(8000, 60, 0.4), 8000), 0.06);
    f.assert_preserved_and_clean();
}

#[test]
fn ac10_identity_contains_only_verified_fixed_engine_and_script() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let expected = Some(EngineIdentity {
        name: "Praat".into(),
        version: "7.0.02".into(),
        binary_sha256: Some(EXE_HASH.into()),
        script_sha256: Some(SCRIPT_HASH.into()),
    });
    assert_eq!(backend.identity(), expected);
    assert_eq!(backend.identity(), expected);
    f.assert_preserved_and_clean();
}

#[test]
fn ac01_verified_executable_pin_denies_write_and_delete_until_drop() {
    let _serial = serial();
    let f = Fixture::new();
    let installation = f.root.path().join("Praat 固定 install");
    fs::create_dir(&installation).unwrap();
    let executable = installation.join("Praat.exe");
    fs::copy(fixture_executable(), &executable).unwrap();
    let backend = PraatBackend::new(
        f.config(executable.clone()),
        deadline(),
        &CancellationToken::new(),
    )
    .unwrap();
    assert!(
        OpenOptions::new().write(true).open(&executable).is_err(),
        "verified executable must remain write-pinned"
    );
    assert!(
        fs::remove_file(&executable).is_err(),
        "verified executable must remain delete-pinned"
    );
    assert_200_hz(&extract(&backend, &tone(8000, 60, 0.4), 8000), 0.06);
    drop(backend);
    assert!(
        OpenOptions::new().write(true).open(&executable).is_ok(),
        "backend drop must release the pin"
    );
    f.assert_preserved_and_clean();
}

#[test]
fn ac05_direct_extract_rejects_malformed_mono_without_mutation_then_retries() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let cases = [
        (vec![], 8000),
        (vec![0.0; 480], 7999),
        (vec![0.0; 2881], 48001),
        (vec![0.0; 8000 * 60 + 1], 8000),
        (vec![f64::NAN; 480], 8000),
        (vec![f64::INFINITY; 480], 8000),
        (vec![f64::NEG_INFINITY; 480], 8000),
        (vec![1.00001; 480], 8000),
        (vec![-1.00001; 480], 8000),
    ];
    for (samples, sample_rate) in cases {
        let bits: Vec<_> = samples.iter().map(|v| v.to_bits()).collect();
        assert_error(
            backend.extract(
                MonoPcm {
                    samples: &samples,
                    sample_rate,
                },
                &AnalysisProfile::v1().pitch,
                deadline(),
                &CancellationToken::new(),
            ),
            AnalysisErrorKind::InvalidInput,
        );
        assert_eq!(
            samples.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            bits
        );
        f.assert_preserved_and_clean();
    }
    assert_200_hz(&extract(&backend, &tone(8000, 60, 0.4), 8000), 0.06);
    f.assert_preserved_and_clean();
}

#[test]
fn ac10_direct_extract_rejects_every_changed_pitch_profile_field() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let samples = tone(8000, 60, 0.4);
    let base = AnalysisProfile::v1().pitch;
    let mut variants = Vec::new();
    macro_rules! changed {
        ($field:ident, $value:expr) => {{
            let mut p = base.clone();
            p.$field = $value;
            variants.push(p);
        }};
    }
    changed!(floor_hz, 51.0);
    changed!(top_hz, 799.0);
    changed!(step_seconds, 0.011);
    changed!(max_candidates, 14);
    changed!(very_accurate, true);
    changed!(attenuation_at_top, 0.031);
    changed!(silence_threshold, 0.091);
    changed!(voicing_threshold, 0.51);
    changed!(octave_cost, 0.056);
    changed!(octave_jump_cost, 0.36);
    changed!(voiced_unvoiced_cost, 0.15);
    changed!(floor_hz, f64::NAN);
    for profile in variants {
        assert_error(
            backend.extract(
                MonoPcm {
                    samples: &samples,
                    sample_rate: 8000,
                },
                &profile,
                deadline(),
                &CancellationToken::new(),
            ),
            AnalysisErrorKind::UnsupportedProfile,
        );
        f.assert_preserved_and_clean();
    }
}

#[test]
fn ac05_actual_200hz_two_seconds_at_8_and_48khz_preserves_samples() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    for rate in [8000, 48000] {
        let samples = tone(rate, 2000, 0.4);
        let bits: Vec<_> = samples.iter().map(|v| v.to_bits()).collect();
        assert_200_hz(&extract(&backend, &samples, rate), 2.0);
        assert_eq!(
            samples.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            bits
        );
        f.assert_preserved_and_clean();
    }
}

#[test]
fn ac05_actual_maximum_60_seconds_48khz_completes_within_same_deadline() {
    let _serial = serial();
    let f = Fixture::new();
    let samples = tone(48000, 60_000, 0.4);
    let started = Instant::now();
    let common_deadline = started + Duration::from_secs(60);
    let cancel = CancellationToken::new();
    let backend =
        PraatBackend::new(f.config(fixture_executable()), common_deadline, &cancel).unwrap();
    let track = backend
        .extract(
            MonoPcm {
                samples: &samples,
                sample_rate: 48000,
            },
            &AnalysisProfile::v1().pitch,
            common_deadline,
            &cancel,
        )
        .unwrap();
    assert!(
        Instant::now() < common_deadline,
        "constructor, extraction and cleanup share the 60 s budget"
    );
    assert_200_hz(&track, 60.0);
    f.assert_preserved_and_clean();
    eprintln!(
        "I5c actual maximum fixture elapsed_ms={}",
        started.elapsed().as_millis()
    );
    // Peak child commit, transient work bytes and descendant exit proof need
    // I5c-2/private process telemetry; a returned track does not prove them.
}

#[test]
fn ac05_actual_zero_input_is_unvoiced_not_zero_hz() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let track = extract(&backend, &[0.0; 16000], 8000);
    assert_times(&track, 2.0);
    assert!(track
        .frames
        .iter()
        .all(|frame| frame.value == SampleValue::Missing(MissingReason::Unvoiced)));
    f.assert_preserved_and_clean();
}

#[test]
fn ac05_actual_59ms_direct_rejects_core_reports_too_short_and_60_61ms_are_voiced() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    for rate in [8000, 48000] {
        let short = tone(rate, 59, 0.4);
        assert_error(
            backend.extract(
                MonoPcm {
                    samples: &short,
                    sample_rate: rate,
                },
                &AnalysisProfile::v1().pitch,
                deadline(),
                &CancellationToken::new(),
            ),
            AnalysisErrorKind::InvalidInput,
        );
        let pcm: Vec<_> = short.iter().map(|s| (s * 32768.0).round() as i16).collect();
        let report = analyze_pcm(&backend, &pcm, rate, 1);
        assert_eq!(report.status, AnalysisStatus::Partial);
        assert_eq!(
            report.measures.pitch.median_hz,
            SampleValue::Missing(MissingReason::TooShort)
        );
        assert!(report.measures.pitch.frames.is_empty());
        assert_eq!(report.measures.duration_seconds, 0.059);
        assert!(report
            .quality
            .iter()
            .any(|q| q.kind == QualityKind::ShortInput));
        for ms in [60, 61] {
            assert_200_hz(
                &extract(&backend, &tone(rate, ms, 0.4), rate),
                ms as f64 / 1000.0,
            );
        }
        f.assert_preserved_and_clean();
    }
}

#[test]
fn ac05_actual_sub_pcm16_half_lsb_tone_survives_float64_transport() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    // Strictly below the half-LSB rounding tie: any PCM16 re-quantization
    // erases the entire signal, including samples exactly at sine extrema.
    let samples = tone(48000, 2000, 0.49 / 32768.0);
    assert!(samples.iter().all(|x| x.abs() <= 0.5 / 32768.0));
    assert!(samples.iter().all(|x| (x * 32768.0).round() == 0.0));
    assert_200_hz(&extract(&backend, &samples, 48000), 2.0);
    f.assert_preserved_and_clean();
}

#[test]
fn ac05_actual_core_antiphase_stereo_remains_digital_silence_and_unvoiced() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let mut samples = Vec::new();
    for sample in tone(8000, 2000, 0.4) {
        let v = (sample * 32768.0).round() as i16;
        samples.extend_from_slice(&[v, -v]);
    }
    let before = samples.clone();
    let report = analyze_pcm(&backend, &samples, 8000, 2);
    assert_eq!(report.measures.rms_dbfs, SampleValue::DigitalSilence);
    assert_eq!(
        report.measures.pitch.median_hz,
        SampleValue::Missing(MissingReason::Unvoiced)
    );
    assert_eq!(
        report.measures.pitch.valid_frame_fraction,
        SampleValue::Finite(0.0)
    );
    assert_eq!(report.status, AnalysisStatus::Partial);
    assert!(!report.measures.pitch.frames.is_empty());
    assert!(report
        .measures
        .pitch
        .frames
        .iter()
        .all(|f| f.value == SampleValue::Missing(MissingReason::Unvoiced)));
    assert!(report
        .quality
        .iter()
        .any(|q| q.kind == QualityKind::StereoMixed));
    assert_eq!(samples, before);
    report.validate().unwrap();
    f.assert_preserved_and_clean();
}

#[test]
fn ac09_extract_precancel_beats_expired_deadline_then_retry_succeeds() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let samples = tone(8000, 60, 0.4);
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert_error(
        backend.extract(
            MonoPcm {
                samples: &samples,
                sample_rate: 8000,
            },
            &AnalysisProfile::v1().pitch,
            Instant::now() - Duration::from_secs(1),
            &cancel,
        ),
        AnalysisErrorKind::Cancelled,
    );
    f.assert_preserved_and_clean();
    assert_200_hz(&extract(&backend, &samples, 8000), 0.06);
    f.assert_preserved_and_clean();
}

#[test]
fn ac09_extract_expired_deadline_is_global_error_then_retry_succeeds() {
    let _serial = serial();
    let f = Fixture::new();
    let backend = f.backend();
    let samples = tone(8000, 60, 0.4);
    assert_error(
        backend.extract(
            MonoPcm {
                samples: &samples,
                sample_rate: 8000,
            },
            &AnalysisProfile::v1().pitch,
            Instant::now() - Duration::from_secs(1),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::TimedOut,
    );
    f.assert_preserved_and_clean();
    assert_200_hz(&extract(&backend, &samples, 8000), 0.06);
    f.assert_preserved_and_clean();
}

#[test]
fn ac01_missing_backend_fallback_preserves_rust_measurements_and_original() {
    let _serial = serial();
    let f = Fixture::new();
    assert_error(
        PraatBackend::new(
            f.config(f.root.path().join("missing.exe")),
            deadline(),
            &CancellationToken::new(),
        ),
        AnalysisErrorKind::BackendUnavailable,
    );
    let pcm = vec![16384_i16; 8000];
    let before = pcm.clone();
    // Explicit caller policy only after BackendUnavailable; never after timeout.
    let report = analyze_pcm(&MissingPitchBackend, &pcm, 8000, 1);
    assert_eq!(report.status, AnalysisStatus::Partial);
    assert_eq!(report.measures.duration_seconds, 1.0);
    match report.measures.rms_dbfs {
        SampleValue::Finite(db) => assert!((db - 20.0 * 0.5_f64.log10()).abs() < 1e-10),
        ref other => panic!("RMS must survive missing pitch: {other:?}"),
    }
    assert_eq!(
        report.measures.pitch.median_hz,
        SampleValue::Missing(MissingReason::BackendUnavailable)
    );
    assert!(report.pitch_engine.is_none());
    assert_eq!(pcm, before);
    report.validate().unwrap();
    f.assert_preserved_and_clean();
}
