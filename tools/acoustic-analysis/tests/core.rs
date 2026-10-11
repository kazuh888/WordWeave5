//! Independent acceptance oracles for ACOUSTIC-HISTORY-001 I4.
//! Expectations come from spec AH-AC01/03/05/06/09/10/18 and design §7.
use acoustic_analysis::*;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct FixedPitch {
    result: Result<PitchTrack, AnalysisError>,
}

impl PitchBackend for FixedPitch {
    fn identity(&self) -> Option<EngineIdentity> {
        Some(EngineIdentity {
            name: "acceptance-fixture".into(),
            version: "1".into(),
            binary_sha256: None,
            script_sha256: None,
        })
    }

    fn extract(
        &self,
        _pcm: MonoPcm<'_>,
        _profile: &PitchProfile,
        _deadline: Instant,
        _cancel: &CancellationToken,
    ) -> Result<PitchTrack, AnalysisError> {
        self.result.clone()
    }
}

fn pitch(values: &[(f64, SampleValue)]) -> FixedPitch {
    FixedPitch {
        result: Ok(PitchTrack {
            frames: values
                .iter()
                .map(|(time, value)| TimedValue {
                    time_seconds: *time,
                    value: value.clone(),
                })
                .collect(),
        }),
    }
}

fn error_backend(kind: AnalysisErrorKind) -> FixedPitch {
    FixedPitch {
        result: Err(AnalysisError { kind }),
    }
}

fn run(samples: &[i16], rate: u32, channels: u16, backend: &dyn PitchBackend) -> AnalysisReport {
    analyze(
        AnalysisRequest {
            pcm: PcmView::new(samples, rate, channels).expect("valid synthetic PCM fixture"),
            profile: &AnalysisProfile::v1(),
            limits: &AnalysisLimits::v1(),
        },
        backend,
        &CancellationToken::new(),
    )
    .expect("valid input must produce a report")
}

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-10,
        "actual={actual:.15}, expected={expected:.15}"
    );
}

fn finite(actual: &SampleValue, expected: f64) {
    match actual {
        SampleValue::Finite(value) => near(*value, expected),
        other => panic!("expected finite {expected}, got {other:?}"),
    }
}

// Deliberately uses PCM samples as the oracle, never ideal unquantized amplitudes.
fn reference_level(samples: &[f64]) -> SampleValue {
    let square_sum: f64 = samples.iter().map(|x| x * x).sum();
    if square_sum == 0.0 {
        SampleValue::DigitalSilence
    } else {
        SampleValue::Finite(10.0 * (square_sum / samples.len() as f64).log10())
    }
}

fn level_near(actual: &SampleValue, expected: &SampleValue) {
    match expected {
        SampleValue::Finite(value) => finite(actual, *value),
        _ => assert_eq!(actual, expected),
    }
}

fn mono_reference(samples: &[i16], channels: usize) -> Vec<f64> {
    samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().map(|s| f64::from(*s) / 32768.0).sum::<f64>() / channels as f64)
        .collect()
}

fn assert_energy_oracle(report: &AnalysisReport, samples: &[i16], rate: u32, channels: usize) {
    let mono = mono_reference(samples, channels);
    assert_eq!(report.sample_rate, rate);
    assert_eq!(report.channels as usize, channels);
    assert_eq!(report.input_frames, mono.len() as u64);
    let duration = mono.len() as f64 / f64::from(rate);
    near(report.measures.duration_seconds, duration);
    level_near(&report.measures.rms_dbfs, &reference_level(&mono));
    let width = (f64::from(rate) * 0.025).round() as usize;
    let mut expected_frames = Vec::new();
    let mut low_hops = Vec::new();
    for k in 0_usize.. {
        let start = (k as f64 * f64::from(rate) * 0.010).round() as usize;
        if start >= mono.len() {
            break;
        }
        let end = (start + width).min(mono.len());
        let level = reference_level(&mono[start..end]);
        expected_frames.push((
            (start + end) as f64 / (2.0 * f64::from(rate)),
            level.clone(),
        ));
        if matches!(level, SampleValue::DigitalSilence)
            || matches!(level, SampleValue::Finite(db) if db <= -40.0)
        {
            low_hops.push((k as f64 * 0.010, ((k + 1) as f64 * 0.010).min(duration)));
        }
    }
    assert_eq!(report.measures.rms_frames.len(), expected_frames.len());
    for (actual, (time, value)) in report.measures.rms_frames.iter().zip(expected_frames) {
        near(actual.time_seconds, time);
        level_near(&actual.value, &value);
    }
    let mut joined: Vec<(f64, f64)> = Vec::new();
    for (start, end) in low_hops {
        if let Some(last) = joined.last_mut() {
            if (last.1 - start).abs() < 1e-12 {
                last.1 = end;
                continue;
            }
        }
        joined.push((start, end));
    }
    // Integer hop count avoids treating binary floating point error as 199.999ms.
    joined.retain(|(start, end)| end - start >= 0.200 - 1e-12);
    assert_eq!(report.measures.low_intervals.len(), joined.len());
    for (actual, (start, end)) in report.measures.low_intervals.iter().zip(&joined) {
        near(actual.start_seconds, *start);
        near(actual.end_seconds, *end);
    }
    let total: f64 = joined.iter().map(|(start, end)| end - start).sum();
    near(report.measures.low_total_seconds, total);
    near(report.measures.low_fraction, total / duration);
}

fn assert_pitch_missing(report: &AnalysisReport, reason: MissingReason) {
    let expected = SampleValue::Missing(reason);
    assert_eq!(report.status, AnalysisStatus::Partial);
    assert_eq!(report.measures.pitch.median_hz, expected);
    assert_eq!(report.measures.pitch.p10_hz, expected);
    assert_eq!(report.measures.pitch.p90_hz, expected);
}

#[test]
fn ac03_pcm_view_rejects_invalid_shape_rate_channels_and_empty_data() {
    for (samples, rate, channels) in [
        (&[][..], 16000, 1),
        (&[1][..], 16000, 2),
        (&[1, 2][..], 7999, 1),
        (&[1, 2][..], 48001, 1),
        (&[1, 2][..], 16000, 0),
        (&[1, 2, 3][..], 16000, 3),
    ] {
        let error = PcmView::new(samples, rate, channels)
            .err()
            .expect("must reject invalid input");
        assert_eq!(error.kind, AnalysisErrorKind::InvalidInput);
    }
}

#[test]
fn ac03_pcm_view_accepts_exact_sixty_seconds_and_preserves_borrowed_samples() {
    for rate in [8000, 48000] {
        for channels in [1, 2] {
            let mut samples = vec![17; rate as usize * 60 * channels as usize];
            let view = PcmView::new(&samples, rate, channels).unwrap();
            assert_eq!(view.frames(), rate as usize * 60);
            assert_eq!(view.sample_rate(), rate);
            assert_eq!(view.channels(), channels);
            assert_eq!(view.samples().as_ptr(), samples.as_ptr());
            assert_eq!(view.samples(), samples);
            samples.extend(std::iter::repeat_n(17, channels as usize));
            assert_eq!(
                PcmView::new(&samples, rate, channels).err().unwrap().kind,
                AnalysisErrorKind::InvalidInput
            );
        }
    }
}

#[test]
fn ac05_zero_is_digital_silence_and_missing_backend_leaves_local_measures() {
    let samples = vec![0; 16000];
    let report = run(&samples, 16000, 1, &MissingPitchBackend);
    assert_energy_oracle(&report, &samples, 16000, 1);
    assert_eq!(report.measures.rms_dbfs, SampleValue::DigitalSilence);
    assert!(report
        .measures
        .rms_frames
        .iter()
        .all(|f| f.value == SampleValue::DigitalSilence));
    assert_pitch_missing(&report, MissingReason::BackendUnavailable);
    assert_eq!(
        report.measures.pitch.valid_frame_fraction,
        SampleValue::Missing(MissingReason::BackendUnavailable)
    );
    assert!(report.measures.pitch.frames.is_empty());
    assert!(report.pitch_engine.is_none());
    assert_eq!(report.measures.low_intervals.len(), 1);
    near(report.measures.low_total_seconds, 1.0);
    assert!(report.validate().is_ok());
}

#[test]
fn ac05_quantized_sine_rms_and_partial_tail_windows_match_sample_oracle() {
    let samples: Vec<i16> = (0..16237)
        .map(|i| {
            (16383.0 * (std::f64::consts::TAU * 200.0 * i as f64 / 16000.0).sin()).round() as i16
        })
        .collect();
    let original = samples.clone();
    let report = run(&samples, 16000, 1, &MissingPitchBackend);
    assert_energy_oracle(&report, &samples, 16000, 1);
    assert_eq!(samples, original, "analysis preserves PCM exactly");
    let final_frame = report.measures.rms_frames.last().unwrap();
    near(final_frame.time_seconds, (16160.0 + 16237.0) / 32000.0);
}

#[test]
fn ac05_dc_is_not_removed_or_normalized() {
    let samples = vec![8192; 8000];
    let report = run(&samples, 8000, 1, &MissingPitchBackend);
    finite(&report.measures.rms_dbfs, 20.0 * 0.25_f64.log10());
    assert_energy_oracle(&report, &samples, 8000, 1);
}

#[test]
fn ac06_stereo_antiphase_is_mean_mixed_to_digital_silence_with_warning() {
    let samples: Vec<i16> = (0..8000).flat_map(|_| [12345, -12345]).collect();
    let original = samples.clone();
    let report = run(&samples, 8000, 2, &MissingPitchBackend);
    assert_energy_oracle(&report, &samples, 8000, 2);
    assert_eq!(report.measures.rms_dbfs, SampleValue::DigitalSilence);
    assert!(report
        .quality
        .iter()
        .any(|q| q.kind == QualityKind::StereoMixed));
    assert_eq!(samples, original);
}

#[test]
fn ac06_stereo_uses_arithmetic_mean_and_original_endpoint_clip_fraction() {
    let samples: Vec<i16> = (0..8000)
        .flat_map(|i| {
            if i == 0 {
                [i16::MIN, i16::MAX]
            } else {
                [16384, 0]
            }
        })
        .collect();
    let report = run(&samples, 8000, 2, &MissingPitchBackend);
    assert_energy_oracle(&report, &samples, 8000, 2);
    let clip = report
        .quality
        .iter()
        .find(|q| q.kind == QualityKind::ClipCandidate)
        .expect("source endpoint warning");
    near(clip.fraction.unwrap(), 2.0 / 16000.0);
}

#[test]
fn ac05_exact_minus_forty_dbfs_is_included_as_low_energy() {
    // 625 samples at 25kHz: one 8192 sample => RMS=(8192/25)/32768=0.01.
    for (impulse, expected_low) in [(8191, 1.0), (8192, 1.0), (8193, 0.0)] {
        let mut samples = vec![0; 25000];
        for i in (0..samples.len()).step_by(625) {
            samples[i] = impulse;
        }
        let report = run(&samples, 25000, 1, &MissingPitchBackend);
        assert_energy_oracle(&report, &samples, 25000, 1);
        near(report.measures.low_total_seconds, expected_low);
        if impulse == 8192 {
            finite(&report.measures.rms_frames[0].value, -40.0);
        }
    }
}

#[test]
fn ac05_low_energy_interval_minimum_is_inclusive_at_two_hundred_ms() {
    for (frames, expected) in [(1599, 0.0), (1600, 0.2), (1601, 1601.0 / 8000.0)] {
        let samples = vec![0; frames];
        let report = run(&samples, 8000, 1, &MissingPitchBackend);
        assert_energy_oracle(&report, &samples, 8000, 1);
        near(report.measures.low_total_seconds, expected);
    }
}

#[test]
fn ac05_low_hop_intervals_merge_at_edges_without_counting_overlapping_windows() {
    let mut samples = vec![0; 8000];
    samples[2400..4800].fill(16384);
    let report = run(&samples, 8000, 1, &MissingPitchBackend);
    assert_energy_oracle(&report, &samples, 8000, 1);
    assert_eq!(report.measures.low_intervals.len(), 2);
    near(report.measures.low_intervals[0].start_seconds, 0.0);
    near(report.measures.low_intervals[0].end_seconds, 0.28);
    near(report.measures.low_intervals[1].start_seconds, 0.60);
    near(report.measures.low_intervals[1].end_seconds, 1.0);
    near(report.measures.low_total_seconds, 0.68);
}

#[test]
fn ac05_rms_window_sample_rounding_is_defined_for_non_integral_hops() {
    let samples: Vec<i16> = (0..22071)
        .map(|i| ((i * 37) % 23000) as i16 - 11500)
        .collect();
    let report = run(&samples, 22050, 1, &MissingPitchBackend);
    assert_energy_oracle(&report, &samples, 22050, 1);
}

#[test]
fn ac06_one_sample_keeps_local_measure_and_marks_pitch_too_short() {
    let report = run(&[1234], 48000, 1, &pitch(&[]));
    assert_energy_oracle(&report, &[1234], 48000, 1);
    assert_pitch_missing(&report, MissingReason::TooShort);
    assert!(report
        .quality
        .iter()
        .any(|q| q.kind == QualityKind::ShortInput));
    assert!(report.validate().is_ok());
}

#[test]
fn ac06_fixed_floor_short_input_cutoff_is_below_sixty_ms_not_inclusive() {
    // I0 probe: minimum Praat duration is 3 / floor(50Hz) = 60ms.
    // A fatal backend fixture proves 59ms was handled before unsafe extraction.
    let short_samples = vec![1234; 472]; // 59ms at 8kHz
    let short = run(
        &short_samples,
        8000,
        1,
        &error_backend(AnalysisErrorKind::InvalidInput),
    );
    assert_pitch_missing(&short, MissingReason::TooShort);
    assert_energy_oracle(&short, &short_samples, 8000, 1);
    for frames in [480, 488] {
        // 60ms and 61ms
        let samples = vec![1234; frames];
        let report = run(
            &samples,
            8000,
            1,
            &pitch(&[(0.03, SampleValue::Finite(200.0))]),
        );
        assert_eq!(report.status, AnalysisStatus::Complete);
        finite(&report.measures.pitch.median_hz, 200.0);
        assert_energy_oracle(&report, &samples, 8000, 1);
    }
}

#[test]
fn ac05_pitch_quantiles_use_linear_interpolation_and_all_frames_denominator() {
    let backend = pitch(&[
        (0.1, SampleValue::Finite(800.0)),
        (0.2, SampleValue::Missing(MissingReason::Unvoiced)),
        (0.3, SampleValue::Finite(50.0)),
        (0.4, SampleValue::Finite(300.0)),
        (0.5, SampleValue::Finite(100.0)),
    ]);
    let report = run(&vec![9000; 16000], 16000, 1, &backend);
    assert_eq!(report.status, AnalysisStatus::Complete);
    finite(&report.measures.pitch.median_hz, 200.0);
    finite(&report.measures.pitch.p10_hz, 65.0);
    finite(&report.measures.pitch.p90_hz, 650.0);
    finite(&report.measures.pitch.valid_frame_fraction, 0.8);
    assert_eq!(report.measures.pitch.frames, backend.result.unwrap().frames);
    assert_eq!(
        report.pitch_engine.as_ref().unwrap().name,
        "acceptance-fixture"
    );
    assert!(report.validate().is_ok());
}

#[test]
fn ac05_single_voiced_pitch_frame_has_identical_quantiles() {
    let report = run(
        &vec![9000; 8000],
        8000,
        1,
        &pitch(&[(0.125, SampleValue::Finite(177.0))]),
    );
    finite(&report.measures.pitch.median_hz, 177.0);
    finite(&report.measures.pitch.p10_hz, 177.0);
    finite(&report.measures.pitch.p90_hz, 177.0);
    finite(&report.measures.pitch.valid_frame_fraction, 1.0);
}

#[test]
fn ac05_all_unvoiced_is_missing_hz_and_zero_valid_fraction_not_zero_hz() {
    let report = run(
        &vec![0; 8000],
        8000,
        1,
        &pitch(&[
            (0.1, SampleValue::Missing(MissingReason::Unvoiced)),
            (0.2, SampleValue::Missing(MissingReason::Unvoiced)),
        ]),
    );
    assert_pitch_missing(&report, MissingReason::Unvoiced);
    finite(&report.measures.pitch.valid_frame_fraction, 0.0);
    assert_eq!(report.measures.pitch.frames.len(), 2);
    assert!(report
        .quality
        .iter()
        .any(|q| q.kind == QualityKind::NoVoicedFrames));
    assert!(report.validate().is_ok());
}

#[test]
fn ac01_recoverable_backend_errors_are_partial_and_preserve_rust_measures() {
    for (kind, reason) in [
        (
            AnalysisErrorKind::BackendUnavailable,
            MissingReason::BackendUnavailable,
        ),
        (
            AnalysisErrorKind::BackendFailed,
            MissingReason::BackendFailed,
        ),
        (
            AnalysisErrorKind::InvalidOutput,
            MissingReason::InvalidOutput,
        ),
    ] {
        let samples = vec![8192; 8000];
        let report = run(&samples, 8000, 1, &error_backend(kind));
        assert_energy_oracle(&report, &samples, 8000, 1);
        assert_pitch_missing(&report, reason);
        assert!(report
            .quality
            .iter()
            .any(|q| q.kind == QualityKind::BackendWarning));
        assert!(report.validate().is_ok());
    }
}

#[test]
fn ac01_fatal_backend_errors_return_no_partial_report() {
    let samples = vec![1234; 8000];
    for kind in [
        AnalysisErrorKind::InvalidInput,
        AnalysisErrorKind::UnsupportedProfile,
        AnalysisErrorKind::TimedOut,
        AnalysisErrorKind::Cancelled,
        AnalysisErrorKind::ResourceLimit,
    ] {
        let result = analyze(
            AnalysisRequest {
                pcm: PcmView::new(&samples, 8000, 1).unwrap(),
                profile: &AnalysisProfile::v1(),
                limits: &AnalysisLimits::v1(),
            },
            &error_backend(kind),
            &CancellationToken::new(),
        );
        assert_eq!(result.err().expect("no report on fatal error").kind, kind);
    }
}

#[test]
fn ac05_invalid_pitch_values_times_and_order_are_invalid_output_missing() {
    let bad_tracks = vec![
        vec![(0.1, SampleValue::Finite(49.99))],
        vec![(0.1, SampleValue::Finite(800.01))],
        vec![(0.1, SampleValue::Finite(0.0))],
        vec![(0.1, SampleValue::Finite(f64::NAN))],
        vec![(0.1, SampleValue::Finite(f64::INFINITY))],
        vec![(0.1, SampleValue::DigitalSilence)],
        vec![(-0.01, SampleValue::Finite(100.0))],
        vec![(1.01, SampleValue::Finite(100.0))],
        vec![(f64::NAN, SampleValue::Finite(100.0))],
        vec![
            (0.2, SampleValue::Finite(100.0)),
            (0.1, SampleValue::Finite(200.0)),
        ],
        vec![
            (0.1, SampleValue::Finite(100.0)),
            (0.1, SampleValue::Finite(200.0)),
        ],
    ];
    for frames in bad_tracks {
        let samples = vec![1234; 8000];
        let report = run(&samples, 8000, 1, &pitch(&frames));
        assert_pitch_missing(&report, MissingReason::InvalidOutput);
        assert!(
            report.measures.pitch.frames.is_empty(),
            "discard invalid track as a whole"
        );
        assert_energy_oracle(&report, &samples, 8000, 1);
    }
}

#[test]
fn ac09_precancelled_request_returns_cancelled_without_mutating_pcm() {
    let samples = vec![1234; 8000];
    let token = CancellationToken::new();
    token.cancel();
    let result = analyze(
        AnalysisRequest {
            pcm: PcmView::new(&samples, 8000, 1).unwrap(),
            profile: &AnalysisProfile::v1(),
            limits: &AnalysisLimits::v1(),
        },
        &MissingPitchBackend,
        &token,
    );
    assert_eq!(result.err().unwrap().kind, AnalysisErrorKind::Cancelled);
    assert!(samples.iter().all(|s| *s == 1234));
    let retry = run(&samples, 8000, 1, &MissingPitchBackend);
    assert_energy_oracle(&retry, &samples, 8000, 1);
}

struct CancelOnExtract;
impl PitchBackend for CancelOnExtract {
    fn identity(&self) -> Option<EngineIdentity> {
        None
    }
    fn extract(
        &self,
        _pcm: MonoPcm<'_>,
        _profile: &PitchProfile,
        _deadline: Instant,
        token: &CancellationToken,
    ) -> Result<PitchTrack, AnalysisError> {
        token.cancel();
        Ok(PitchTrack {
            frames: vec![TimedValue {
                time_seconds: 0.1,
                value: SampleValue::Finite(100.0),
            }],
        })
    }
}

#[test]
fn ac09_cancellation_during_backend_discards_late_success() {
    let samples = vec![1234; 8000];
    let result = analyze(
        AnalysisRequest {
            pcm: PcmView::new(&samples, 8000, 1).unwrap(),
            profile: &AnalysisProfile::v1(),
            limits: &AnalysisLimits::v1(),
        },
        &CancelOnExtract,
        &CancellationToken::new(),
    );
    assert_eq!(
        result
            .err()
            .expect("late success must not publish a report")
            .kind,
        AnalysisErrorKind::Cancelled
    );
}

struct CompleteAfterDeadline;
impl PitchBackend for CompleteAfterDeadline {
    fn identity(&self) -> Option<EngineIdentity> {
        None
    }
    fn extract(
        &self,
        _pcm: MonoPcm<'_>,
        _profile: &PitchProfile,
        deadline: Instant,
        _token: &CancellationToken,
    ) -> Result<PitchTrack, AnalysisError> {
        // Wait for the supplied deadline, avoiding a guessed wall-clock duration.
        while Instant::now() <= deadline {
            std::thread::yield_now();
        }
        Ok(PitchTrack {
            frames: vec![TimedValue {
                time_seconds: 0.1,
                value: SampleValue::Finite(100.0),
            }],
        })
    }
}

#[test]
fn ac09_deadline_expiry_before_or_after_backend_returns_timeout() {
    let samples = vec![1234; 8000];
    for wall_time in [Duration::ZERO, Duration::from_millis(20)] {
        let mut limits = AnalysisLimits::v1();
        limits.wall_time = wall_time;
        let result = analyze(
            AnalysisRequest {
                pcm: PcmView::new(&samples, 8000, 1).unwrap(),
                profile: &AnalysisProfile::v1(),
                limits: &limits,
            },
            &CompleteAfterDeadline,
            &CancellationToken::new(),
        );
        assert_eq!(
            result.err().expect("no report after common deadline").kind,
            AnalysisErrorKind::TimedOut
        );
    }
}

#[test]
fn ac03_core_limits_are_fixed_and_shortages_are_resource_errors() {
    let default = AnalysisLimits::v1();
    assert_eq!(default.wall_time, Duration::from_secs(60));
    assert_eq!(default.max_frames, 6001);
    assert_eq!(default.max_buffer_bytes, 64 * 1024 * 1024);
    let samples = vec![1234; 8000];
    for (max_frames, max_buffer_bytes) in [(99, default.max_buffer_bytes), (6001, 0)] {
        let limits = AnalysisLimits {
            wall_time: default.wall_time,
            max_frames,
            max_buffer_bytes,
        };
        let result = analyze(
            AnalysisRequest {
                pcm: PcmView::new(&samples, 8000, 1).unwrap(),
                profile: &AnalysisProfile::v1(),
                limits: &limits,
            },
            &MissingPitchBackend,
            &CancellationToken::new(),
        );
        assert_eq!(
            result.err().expect("no truncation to fit limit").kind,
            AnalysisErrorKind::ResourceLimit
        );
    }
    let exact = AnalysisLimits {
        wall_time: default.wall_time,
        max_frames: 100,
        max_buffer_bytes: default.max_buffer_bytes,
    };
    assert!(analyze(
        AnalysisRequest {
            pcm: PcmView::new(&samples, 8000, 1).unwrap(),
            profile: &AnalysisProfile::v1(),
            limits: &exact,
        },
        &MissingPitchBackend,
        &CancellationToken::new()
    )
    .is_ok());
}

#[test]
fn ac03_pitch_frame_limit_is_resource_error_not_recoverable_invalid_output() {
    let backend = pitch(
        &(0..101)
            .map(|i| (i as f64 / 101.0, SampleValue::Finite(100.0)))
            .collect::<Vec<_>>(),
    );
    let samples = vec![1234; 8000];
    let mut limits = AnalysisLimits::v1();
    limits.max_frames = 100;
    let result = analyze(
        AnalysisRequest {
            pcm: PcmView::new(&samples, 8000, 1).unwrap(),
            profile: &AnalysisProfile::v1(),
            limits: &limits,
        },
        &backend,
        &CancellationToken::new(),
    );
    assert_eq!(result.err().unwrap().kind, AnalysisErrorKind::ResourceLimit);
}

#[test]
fn ac10_v1_profile_records_every_fixed_setting_and_rejects_reinterpretation() {
    let p = AnalysisProfile::v1();
    assert_eq!(p.schema_version, 1);
    assert_eq!(p.profile_id, "reading-v1");
    assert_eq!(p.preprocessing, PreprocessProfile::Pcm16MeanChannelsV1);
    assert_eq!(p.energy.window_ms, 25);
    assert_eq!(p.energy.hop_ms, 10);
    near(p.energy.low_threshold_dbfs, -40.0);
    assert_eq!(p.energy.min_low_interval_ms, 200);
    assert_eq!(p.pitch.max_candidates, 15);
    assert!(!p.pitch.very_accurate);
    for (actual, expected) in [
        (p.pitch.floor_hz, 50.0),
        (p.pitch.top_hz, 800.0),
        (p.pitch.step_seconds, 0.01),
        (p.pitch.attenuation_at_top, 0.03),
        (p.pitch.silence_threshold, 0.09),
        (p.pitch.voicing_threshold, 0.50),
        (p.pitch.octave_cost, 0.055),
        (p.pitch.octave_jump_cost, 0.35),
        (p.pitch.voiced_unvoiced_cost, 0.14),
    ] {
        near(actual, expected);
    }
    assert!(p.validate().is_ok());
    let mut changed = Vec::new();
    let mut q = p.clone();
    q.schema_version = 2;
    changed.push(q);
    let mut q = p.clone();
    q.profile_id = "reading-v2".into();
    changed.push(q);
    let mut q = p.clone();
    q.energy.hop_ms = 20;
    changed.push(q);
    let mut q = p.clone();
    q.energy.low_threshold_dbfs = -39.99;
    changed.push(q);
    let mut q = p.clone();
    q.pitch.floor_hz = 51.0;
    changed.push(q);
    let mut q = p.clone();
    q.pitch.voicing_threshold = f64::NAN;
    changed.push(q);
    for q in changed {
        assert_eq!(
            q.validate().err().unwrap().kind,
            AnalysisErrorKind::UnsupportedProfile
        );
        let result = analyze(
            AnalysisRequest {
                pcm: PcmView::new(&[0; 8000], 8000, 1).unwrap(),
                profile: &q,
                limits: &AnalysisLimits::v1(),
            },
            &MissingPitchBackend,
            &CancellationToken::new(),
        );
        assert_eq!(
            result.err().unwrap().kind,
            AnalysisErrorKind::UnsupportedProfile
        );
    }
}

#[test]
fn ac10_ac18_same_input_profile_is_reproducible_and_roundtrip_preserves_missing() {
    let samples = vec![17; 8000];
    let first = run(&samples, 8000, 1, &MissingPitchBackend);
    let second = run(&samples, 8000, 1, &MissingPitchBackend);
    assert_eq!(first, second);
    assert_eq!(first.schema_version, 1);
    assert_eq!(first.metrics_version, "acoustic-metrics-v1");
    assert_eq!(first.library_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(first.profile, AnalysisProfile::v1());
    let bytes = serde_json::to_vec(&first).unwrap();
    let decoded: AnalysisReport = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, first);
    assert!(decoded.validate().is_ok());
    let value =
        serde_json::to_value(SampleValue::Missing(MissingReason::BackendUnavailable)).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"kind": "missing", "value": "backend_unavailable"})
    );
    assert_eq!(
        serde_json::to_value(SampleValue::DigitalSilence).unwrap(),
        serde_json::json!({"kind": "digital_silence"})
    );
}

// Hand-built valid report is independent of analyze(), so validation tests cannot
// inherit a parser/analysis defect by treating its output as an oracle.
fn saved_report() -> AnalysisReport {
    let level = 20.0 * 0.5_f64.log10();
    AnalysisReport {
        schema_version: 1,
        sample_rate: 16000,
        channels: 1,
        input_frames: 16000,
        metrics_version: "acoustic-metrics-v1".into(),
        library_version: env!("CARGO_PKG_VERSION").into(),
        profile: AnalysisProfile::v1(),
        pitch_engine: Some(EngineIdentity {
            name: "acceptance-fixture".into(),
            version: "1".into(),
            binary_sha256: None,
            script_sha256: None,
        }),
        status: AnalysisStatus::Complete,
        measures: Measures {
            duration_seconds: 1.0,
            rms_dbfs: SampleValue::Finite(level),
            rms_frames: (0..100)
                .map(|k| {
                    let start = k as f64 * 0.01;
                    TimedValue {
                        time_seconds: (start + (start + 0.025).min(1.0)) / 2.0,
                        value: SampleValue::Finite(level),
                    }
                })
                .collect(),
            low_intervals: vec![],
            low_total_seconds: 0.0,
            low_fraction: 0.0,
            pitch: PitchMeasure {
                frames: vec![
                    TimedValue {
                        time_seconds: 0.1,
                        value: SampleValue::Finite(100.0),
                    },
                    TimedValue {
                        time_seconds: 0.2,
                        value: SampleValue::Finite(200.0),
                    },
                ],
                median_hz: SampleValue::Finite(150.0),
                p10_hz: SampleValue::Finite(110.0),
                p90_hz: SampleValue::Finite(190.0),
                valid_frame_fraction: SampleValue::Finite(1.0),
            },
        },
        quality: vec![],
    }
}

fn reject_saved(report: AnalysisReport) {
    // Prove malformed externally decoded JSON still needs semantic validation.
    let bytes = serde_json::to_vec(&report).unwrap();
    let decoded: AnalysisReport = serde_json::from_slice(&bytes).unwrap();
    assert!(
        decoded.validate().is_err(),
        "semantically invalid report was accepted: {decoded:?}"
    );
}

#[test]
fn ac10_saved_report_valid_fixture_passes_before_negative_cases() {
    let report = saved_report();
    assert!(
        report.validate().is_ok(),
        "independent valid fixture must be accepted"
    );
    let decoded: AnalysisReport =
        serde_json::from_slice(&serde_json::to_vec(&report).unwrap()).unwrap();
    assert_eq!(report, decoded);
    assert!(decoded.validate().is_ok());
}

#[test]
fn ac10_saved_report_rejects_unknown_schema_and_profile() {
    let mut report = saved_report();
    report.schema_version = 2;
    reject_saved(report);
    let mut report = saved_report();
    report.profile.energy.window_ms = 24;
    reject_saved(report);
}

#[test]
fn ac03_saved_report_rejects_input_metadata_mismatch_and_unsupported_pcm_shape() {
    for rate in [7999, 48001, 8000] {
        let mut report = saved_report();
        report.sample_rate = rate;
        reject_saved(report);
    }
    for channels in [0, 3] {
        let mut report = saved_report();
        report.channels = channels;
        reject_saved(report);
    }
    for frames in [0, 15999, 16001, 960001] {
        let mut report = saved_report();
        report.input_frames = frames;
        reject_saved(report);
    }
}

#[test]
fn ac05_saved_report_rejects_invalid_units_ranges_and_nonfinite_values() {
    for duration in [-1.0, 0.0, 60.0001] {
        let mut report = saved_report();
        report.measures.duration_seconds = duration;
        reject_saved(report);
    }
    for db in [0.001, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut report = saved_report();
        report.measures.rms_dbfs = SampleValue::Finite(db);
        // JSON cannot faithfully represent nonfinite f64: test semantic in-memory boundary directly.
        assert!(report.validate().is_err());
        if db.is_finite() {
            reject_saved(report);
        }
    }
    for fraction in [-0.001, 1.001] {
        let mut report = saved_report();
        report.measures.low_fraction = fraction;
        reject_saved(report);
        let mut report = saved_report();
        report.measures.pitch.valid_frame_fraction = SampleValue::Finite(fraction);
        reject_saved(report);
        let mut report = saved_report();
        report.quality.push(Quality {
            kind: QualityKind::ClipCandidate,
            fraction: Some(fraction),
        });
        reject_saved(report);
    }
    let mut report = saved_report();
    report.measures.pitch.median_hz = SampleValue::DigitalSilence;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames[0].value = SampleValue::DigitalSilence;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames[0].value = SampleValue::Finite(801.0);
    reject_saved(report);
}

#[test]
fn ac05_saved_report_rejects_nonmonotonic_outside_and_wrong_hop_times() {
    let mut report = saved_report();
    report.measures.rms_frames[0].time_seconds = 0.011;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.rms_frames.pop();
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames[0].time_seconds = -0.01;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames[1].time_seconds = 1.01;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames[1].time_seconds = 0.1;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames.swap(0, 1);
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.frames[0].time_seconds = f64::NAN;
    assert!(report.validate().is_err());
}

#[test]
fn ac05_saved_report_rejects_overlapping_unsorted_short_and_inconsistent_intervals() {
    for intervals in [
        vec![(0.3, 0.7), (0.5, 0.9)],
        vec![(0.6, 0.9), (0.1, 0.4)],
        vec![(0.1, 0.2)],
        vec![(-0.1, 0.3)],
        vec![(0.8, 1.1)],
        vec![(0.7, 0.3)],
    ] {
        let mut report = saved_report();
        report.measures.low_total_seconds = intervals.iter().map(|(a, b)| b - a).sum();
        report.measures.low_fraction = report.measures.low_total_seconds;
        report.measures.low_intervals = intervals
            .into_iter()
            .map(|(a, b)| TimeInterval {
                start_seconds: a,
                end_seconds: b,
            })
            .collect();
        reject_saved(report);
    }
    let mut report = saved_report();
    report.measures.low_total_seconds = 0.4;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.low_fraction = 0.4;
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.valid_frame_fraction = SampleValue::Finite(0.5);
    reject_saved(report);
    let mut report = saved_report();
    report.measures.pitch.p10_hz = SampleValue::Finite(160.0);
    reject_saved(report);
}
