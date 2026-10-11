//! Independent I5a acceptance tests (AH-AC01/05/06/09/10).
//! Synthetic data only; Windows supervision and real Praat are separate gates.

use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use tokio_util::sync::CancellationToken;

use crate::{
    praat_protocol::{classify_output, parse_pitch, write_mono_wav},
    process::{ProcessFailureReason, ProcessOutput},
    AnalysisErrorKind, AnalysisProfile, MissingReason, MonoPcm, SampleValue,
};

fn parse(bytes: &[u8], duration: f64) -> Result<crate::PitchTrack, crate::AnalysisError> {
    parse_pitch(bytes, duration, &AnalysisProfile::v1().pitch)
}

fn invalid_protocol(bytes: &[u8]) {
    assert_eq!(
        parse(bytes, 1.0)
            .expect_err("malformed protocol must fail atomically")
            .kind,
        AnalysisErrorKind::InvalidOutput,
        "fixture: {:?}",
        String::from_utf8_lossy(bytes)
    );
}

#[test]
fn parser_preserves_nine_decimal_values_without_rounding_to_hop() {
    let track = parse(
        b"WWPITCH\t1\t3\n0.000000000\t50.000000000\n0.013456789\t203.123456789\n1.000000000\t800.000000000\n",
        1.0,
    )
    .unwrap();
    assert_eq!(track.frames.len(), 3);
    for (frame, (time, hz)) in track.frames.iter().zip([
        (0.0_f64, 50.0_f64),
        (0.013456789, 203.123456789),
        (1.0, 800.0),
    ]) {
        assert_eq!(frame.time_seconds.to_bits(), time.to_bits());
        assert_eq!(frame.value, SampleValue::Finite(hz));
    }
}

#[test]
fn parser_accepts_crlf_and_optional_single_final_newline() {
    for bytes in [
        b"WWPITCH\t1\t1\n0.500000000\t200.000000000".as_slice(),
        b"WWPITCH\t1\t1\n0.500000000\t200.000000000\n",
        b"WWPITCH\t1\t1\r\n0.500000000\t200.000000000\r\n",
    ] {
        let track = parse(bytes, 1.0).unwrap();
        assert_eq!(track.frames.len(), 1);
        assert_eq!(track.frames[0].time_seconds, 0.5);
        assert_eq!(track.frames[0].value, SampleValue::Finite(200.0));
    }
}

#[test]
fn parser_maps_only_exact_unvoiced_to_missing() {
    let track = parse(b"WWPITCH\t1\t2\n0\tunvoiced\n1\tunvoiced\n", 1.0).unwrap();
    assert_eq!(track.frames.len(), 2);
    assert!(track
        .frames
        .iter()
        .all(|frame| { frame.value == SampleValue::Missing(MissingReason::Unvoiced) }));
    for token in [
        "Unvoiced",
        "undefined",
        "--undefined--",
        "missing",
        "",
        "unvoiced ",
    ] {
        invalid_protocol(format!("WWPITCH\t1\t1\n0\t{token}\n").as_bytes());
    }
}

#[test]
fn parser_rejects_zero_and_out_of_range_pitch_instead_of_treating_as_unvoiced() {
    for hz in ["0", "0.000000000", "49.999999999", "800.000000001", "801"] {
        invalid_protocol(format!("WWPITCH\t1\t1\n0\t{hz}\n").as_bytes());
    }
}

#[test]
fn parser_rejects_non_decimal_non_finite_signed_and_padded_numbers() {
    for value in [
        "NaN",
        "nan",
        "Inf",
        "inf",
        "Infinity",
        "+200",
        "-0",
        "2e2",
        "2E2",
        " 200",
        "200 ",
        "２００",
        "200,0",
        "0xC8",
    ] {
        invalid_protocol(format!("WWPITCH\t1\t1\n0\t{value}\n").as_bytes());
        invalid_protocol(format!("WWPITCH\t1\t1\n{value}\t200\n").as_bytes());
    }
    let overflow = "9".repeat(400);
    invalid_protocol(format!("WWPITCH\t1\t1\n0\t{overflow}\n").as_bytes());
}

#[test]
fn parser_rejects_duplicate_reversed_and_out_of_duration_times() {
    for bytes in [
        b"WWPITCH\t1\t2\n0.5\t200\n0.5\t201\n".as_slice(),
        b"WWPITCH\t1\t2\n0.5\t200\n0.4\t201\n",
        b"WWPITCH\t1\t1\n1.000000001\t200\n",
        b"WWPITCH\t1\t1\n-0.000000001\t200\n",
    ] {
        invalid_protocol(bytes);
    }
}

#[test]
fn parser_rejects_bad_headers_count_mismatch_and_extra_fields_or_lines() {
    for bytes in [
        b"".as_slice(),
        b"WWPITCH\t2\t1\n0\t200\n",
        b"wwpitch\t1\t1\n0\t200\n",
        b"WWPITCH\t1\t0\n",
        b"WWPITCH\t1\t+1\n0\t200\n",
        b"WWPITCH\t1\t1.0\n0\t200\n",
        b"WWPITCH\t1\t 1\n0\t200\n",
        b"WWPITCH\t1\t1\textra\n0\t200\n",
        b"WWPITCH\t1\t2\n0\t200\n",
        b"WWPITCH\t1\t1\n0\t200\n0.5\t201\n",
        b"WWPITCH\t1\t1\n0\t200\textra\n",
        b"WWPITCH\t1\t1\n0\n",
        b"WWPITCH\t1\t1\n\n0\t200\n",
        b"WWPITCH\t1\t1\n0\t200\n\n",
    ] {
        invalid_protocol(bytes);
    }
}

#[test]
fn parser_rejects_bom_invalid_utf8_and_bare_cr() {
    for bytes in [
        b"\xef\xbb\xbfWWPITCH\t1\t1\n0\t200\n".as_slice(),
        b"WWPITCH\t1\t1\n0\t\xff\n",
        b"WWPITCH\t1\t1\r0\t200\r",
        b"WWPITCH\t1\t1\n0\t200\r",
    ] {
        invalid_protocol(bytes);
    }
}

#[test]
fn parser_rejects_entire_track_when_a_later_frame_is_invalid() {
    invalid_protocol(b"WWPITCH\t1\t3\n0\t200\n0.5\t201\n0.75\tNaN\n");
}

#[test]
fn parser_accepts_6001_frames_and_rejects_6002_as_resource_limit() {
    let mut bytes = String::from("WWPITCH\t1\t6001\n");
    for i in 0..=6000 {
        bytes.push_str(&format!("{}.{:02}0000000\tunvoiced\n", i / 100, i % 100));
    }
    let track = parse(bytes.as_bytes(), 60.0).unwrap();
    assert_eq!(track.frames.len(), 6001);
    assert_eq!(track.frames.first().unwrap().time_seconds, 0.0);
    assert_eq!(track.frames.last().unwrap().time_seconds, 60.0);
    assert_eq!(
        parse(b"WWPITCH\t1\t6002\n", 60.0).unwrap_err().kind,
        AnalysisErrorKind::ResourceLimit
    );
}

#[test]
fn parser_stdout_limit_is_inclusive_and_checked_before_parsing() {
    // Invalid syntax at/below the byte cap is a protocol error, not a resource error.
    for len in [1_048_575, 1_048_576] {
        assert_eq!(
            parse(&vec![b'x'; len], 1.0).unwrap_err().kind,
            AnalysisErrorKind::InvalidOutput
        );
    }
    assert_eq!(
        parse(&vec![b'x'; 1_048_577], 1.0).unwrap_err().kind,
        AnalysisErrorKind::ResourceLimit
    );
}

#[test]
fn parser_failure_does_not_poison_a_later_independent_attempt() {
    invalid_protocol(b"WWPITCH\t1\t1\n0\t0\n");
    let bytes = b"WWPITCH\t1\t1\n0\t200\n";
    assert_eq!(parse(bytes, 1.0).unwrap(), parse(bytes, 1.0).unwrap());
}

fn future_deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

fn write_wav(out: &mut impl Write, samples: &[f64], rate: u32) -> Result<(), crate::AnalysisError> {
    write_mono_wav(
        out,
        MonoPcm {
            samples,
            sample_rate: rate,
        },
        future_deadline(),
        &CancellationToken::new(),
    )
}

fn u16_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes.try_into().unwrap())
}

fn u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().unwrap())
}

// Independent RIFF chunk walker: does not use any production codec or header helper.
fn assert_float64_wave(bytes: &[u8], rate: u32, samples: &[f64]) {
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(u32_le(&bytes[4..8]) as usize, bytes.len() - 8);
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(bytes.len(), 58 + samples.len() * 8);
    let mut cursor = 12;
    let mut chunks = Vec::new();
    while cursor < bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let len = u32_le(&bytes[cursor + 4..cursor + 8]) as usize;
        let payload = &bytes[cursor + 8..cursor + 8 + len];
        chunks.push((id, payload));
        cursor += 8 + len + len % 2;
    }
    assert_eq!(cursor, bytes.len());
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].0, b"fmt ");
    let fmt = chunks[0].1;
    assert_eq!(fmt.len(), 18);
    assert_eq!(u16_le(&fmt[0..2]), 3);
    assert_eq!(u16_le(&fmt[2..4]), 1);
    assert_eq!(u32_le(&fmt[4..8]), rate);
    assert_eq!(u32_le(&fmt[8..12]), rate * 8);
    assert_eq!(u16_le(&fmt[12..14]), 8);
    assert_eq!(u16_le(&fmt[14..16]), 64);
    assert_eq!(u16_le(&fmt[16..18]), 0);
    assert_eq!(chunks[1].0, b"fact");
    assert_eq!(chunks[1].1.len(), 4);
    assert_eq!(u32_le(chunks[1].1) as usize, samples.len());
    assert_eq!(chunks[2].0, b"data");
    assert_eq!(chunks[2].1.len(), samples.len() * 8);
    for (actual, expected) in chunks[2].1.chunks_exact(8).zip(samples) {
        assert_eq!(
            u64::from_le_bytes(actual.try_into().unwrap()),
            expected.to_bits()
        );
    }
}

#[test]
fn wav_preserves_float64_half_lsb_extrema_signed_zero_and_input_in_real_file() {
    let samples = [
        -1.0,
        -0.5,
        -1.0 / 65536.0,
        -0.0,
        0.0,
        1.0 / 65536.0,
        0.5,
        1.0,
    ];
    let before = samples.map(f64::to_bits);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("合成 mono.wav");
    let mut file = std::fs::File::create(&path).unwrap();
    write_wav(&mut file, &samples, 48000).unwrap();
    drop(file);
    let bytes = std::fs::read(path).unwrap();
    assert_float64_wave(&bytes, 48000, &samples);
    assert_eq!(samples.map(f64::to_bits), before);
}

#[test]
fn wav_accepts_short_nonempty_pcm_at_both_sample_rate_boundaries() {
    for rate in [8000, 48000] {
        let mut out = Vec::new();
        write_wav(&mut out, &[0.25], rate).unwrap();
        assert_float64_wave(&out, rate, &[0.25]);
    }
}

#[test]
fn wav_rejects_empty_pcm_and_invalid_rates_before_writing() {
    for (samples, rate) in [
        (&[][..], 8000),
        (&[0.0][..], 0),
        (&[0.0][..], 7999),
        (&[0.0][..], 48001),
        (&[0.0][..], u32::MAX),
    ] {
        let mut out = Vec::new();
        assert_eq!(
            write_wav(&mut out, samples, rate).unwrap_err().kind,
            AnalysisErrorKind::InvalidInput
        );
        assert!(out.is_empty(), "invalid PCM must not publish a WAV header");
    }
}

#[test]
fn wav_rejects_non_finite_and_out_of_range_pcm_without_mutating_input() {
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.000000001,
        1.000000001,
    ] {
        let samples = [0.0, invalid];
        let before = samples.map(f64::to_bits);
        let mut out = Vec::new();
        assert_eq!(
            write_wav(&mut out, &samples, 8000).unwrap_err().kind,
            AnalysisErrorKind::InvalidInput
        );
        assert!(out.is_empty());
        assert_eq!(samples.map(f64::to_bits), before);
    }
}

#[derive(Default)]
struct CountWrites {
    total: usize,
    largest: usize,
}

impl Write for CountWrites {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.total += bytes.len();
        self.largest = self.largest.max(bytes.len());
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn wav_accepts_exactly_sixty_seconds_with_58_byte_header_and_bounded_writes() {
    let samples = vec![0.0; 48000 * 60];
    let mut out = CountWrites::default();
    write_wav(&mut out, &samples, 48000).unwrap();
    assert_eq!(out.total, 23_040_058);
    assert!(out.largest <= 64 * 1024, "write buffer exceeds 64 KiB");
}

#[test]
fn wav_rejects_one_sample_beyond_sixty_seconds() {
    let samples = vec![0.0; 8000 * 60 + 1];
    let mut out = Vec::new();
    assert_eq!(
        write_wav(&mut out, &samples, 8000).unwrap_err().kind,
        AnalysisErrorKind::InvalidInput
    );
    assert!(out.is_empty());
}

#[test]
fn wav_cancelled_at_entry_writes_nothing_and_takes_precedence_over_deadline() {
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut out = Vec::new();
    let result = write_mono_wav(
        &mut out,
        MonoPcm {
            samples: &[0.0],
            sample_rate: 8000,
        },
        Instant::now(),
        &cancel,
    );
    assert_eq!(result.unwrap_err().kind, AnalysisErrorKind::Cancelled);
    assert!(out.is_empty());
}

#[test]
fn wav_expired_deadline_writes_nothing() {
    let mut out = Vec::new();
    let result = write_mono_wav(
        &mut out,
        MonoPcm {
            samples: &[0.0],
            sample_rate: 8000,
        },
        Instant::now(),
        &CancellationToken::new(),
    );
    assert_eq!(result.unwrap_err().kind, AnalysisErrorKind::TimedOut);
    assert!(out.is_empty());
}

// Write is the I/O boundary. Inject control changes without OS faults or real audio.
struct CancelOnFirstWrite {
    cancel: CancellationToken,
    bytes: Vec<u8>,
}
impl Write for CancelOnFirstWrite {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        self.cancel.cancel();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn wav_cancellation_during_writing_is_not_returned_as_success_and_retry_is_independent() {
    let samples = vec![0.25; 20000];
    let cancel = CancellationToken::new();
    let mut out = CancelOnFirstWrite {
        cancel: cancel.clone(),
        bytes: Vec::new(),
    };
    let result = write_mono_wav(
        &mut out,
        MonoPcm {
            samples: &samples,
            sample_rate: 8000,
        },
        future_deadline(),
        &cancel,
    );
    assert_eq!(result.unwrap_err().kind, AnalysisErrorKind::Cancelled);
    assert!(
        !out.bytes.is_empty(),
        "fixture must cancel after a real write"
    );
    assert!(
        out.bytes.len() < 58 + samples.len() * 8,
        "cancellation must stop further output"
    );
    let mut retry = Vec::new();
    write_wav(&mut retry, &samples, 8000).unwrap();
    assert_float64_wave(&retry, 8000, &samples);
}

struct DeadlineOnFirstWrite {
    deadline: Instant,
    bytes: usize,
}
impl Write for DeadlineOnFirstWrite {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes += bytes.len();
        std::thread::sleep(
            self.deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
        );
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn wav_deadline_crossed_during_write_is_not_returned_as_success() {
    let samples = vec![0.0; 20000];
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut out = DeadlineOnFirstWrite { deadline, bytes: 0 };
    let result = write_mono_wav(
        &mut out,
        MonoPcm {
            samples: &samples,
            sample_rate: 8000,
        },
        deadline,
        &CancellationToken::new(),
    );
    assert_eq!(result.unwrap_err().kind, AnalysisErrorKind::TimedOut);
    assert!(out.bytes > 0, "fixture must cross deadline inside write");
    assert!(out.bytes < 58 + samples.len() * 8);
}

struct FailingWriter {
    accepted: usize,
    fail_after: usize,
}
impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.accepted == self.fail_after {
            return Err(io::Error::other("PRIVATE_WRITE_SENTINEL"));
        }
        let len = bytes.len().min(self.fail_after - self.accepted);
        self.accepted += len;
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn wav_write_failure_is_backend_failed_without_raw_error_and_fresh_retry_succeeds() {
    for fail_after in [0, 12, 62] {
        let mut out = FailingWriter {
            accepted: 0,
            fail_after,
        };
        let error = write_wav(&mut out, &[0.25, 0.5], 8000).unwrap_err();
        assert_eq!(out.accepted, fail_after);
        assert_eq!(error.kind, AnalysisErrorKind::BackendFailed);
        assert!(!format!("{error:?}").contains("PRIVATE_WRITE_SENTINEL"));
    }
    let mut retry = Vec::new();
    write_wav(&mut retry, &[0.25, 0.5], 8000).unwrap();
    assert_float64_wave(&retry, 8000, &[0.25, 0.5]);
}

fn output(exit_code: u32, stderr: &[u8], memory_limit_observed: bool) -> ProcessOutput {
    ProcessOutput {
        stdout: b"WWPITCH\t1\t1\n0\t200\n".to_vec(),
        stderr: stderr.to_vec(),
        exit_code,
        peak_job_bytes: 1024,
        memory_limit_observed,
    }
}

#[test]
fn classifier_accepts_clean_exit_without_consuming_or_rewriting_output() {
    let out = output(0, b"", false);
    let before = out.stdout.clone();
    classify_output(&out).unwrap();
    assert_eq!(out.stdout, before);
    assert!(out.stderr.is_empty());
}

#[test]
fn classifier_leaves_protocol_validation_to_parser() {
    let mut out = output(0, b"", false);
    out.stdout = b"malformed protocol".to_vec();
    classify_output(&out).unwrap();
    assert_eq!(
        parse(&out.stdout, 1.0).unwrap_err().kind,
        AnalysisErrorKind::InvalidOutput
    );
}

#[test]
fn classifier_known_oom_is_resource_limit_even_on_exit_zero() {
    for code in [0, 1, u32::MAX] {
        let out = output(
            code,
            b"PRIVATE_STDERR_SENTINEL\nError: Out of memory: allocation failed\n",
            false,
        );
        let error = classify_output(&out).unwrap_err();
        assert_eq!(error.kind, AnalysisErrorKind::ResourceLimit);
        assert_eq!(error.reason, ProcessFailureReason::MemoryLimit);
        assert!(!format!("{error:?}").contains("PRIVATE_STDERR_SENTINEL"));
    }
}

#[test]
fn classifier_oom_signature_can_cross_a_pipe_read_chunk_boundary() {
    let mut stderr = vec![b'x'; 8192 - 5];
    stderr.extend_from_slice(b"Error: Out of memory: PRIVATE_BOUNDARY_SENTINEL");
    let error = classify_output(&output(0, &stderr, false)).unwrap_err();
    assert_eq!(error.kind, AnalysisErrorKind::ResourceLimit);
    assert_eq!(error.reason, ProcessFailureReason::MemoryLimit);
}

#[test]
fn classifier_memory_evidence_overrides_success_and_backend_diagnostic() {
    for stderr in [b"".as_slice(), b"PRIVATE_STDERR_SENTINEL"] {
        let error = classify_output(&output(0, stderr, true)).unwrap_err();
        assert_eq!(error.kind, AnalysisErrorKind::ResourceLimit);
        assert_eq!(error.reason, ProcessFailureReason::MemoryLimit);
    }
}

#[test]
fn classifier_unknown_nonzero_is_resource_limit_even_without_memory_notification() {
    for code in [1, 42, 0xc0000005, u32::MAX] {
        let error = classify_output(&output(code, b"PRIVATE_UNKNOWN_SENTINEL", false)).unwrap_err();
        assert_eq!(error.kind, AnalysisErrorKind::ResourceLimit);
        assert_eq!(error.reason, ProcessFailureReason::UnclassifiedAbnormalExit);
        assert!(!format!("{error:?}").contains("PRIVATE_UNKNOWN_SENTINEL"));
    }
}

#[test]
fn classifier_exit_zero_with_any_stderr_is_backend_diagnostic_not_pipe_failure() {
    for stderr in [b"PRIVATE_STDERR_SENTINEL".as_slice(), b"\n", b" ", b"\xff"] {
        let out = output(0, stderr, false);
        let error = classify_output(&out).unwrap_err();
        assert_eq!(error.kind, AnalysisErrorKind::BackendFailed);
        assert_eq!(error.reason, ProcessFailureReason::BackendDiagnostic);
        assert!(!format!("{error:?}").contains("PRIVATE_STDERR_SENTINEL"));
        assert_eq!(out.stderr, stderr);
    }
}

#[test]
fn classifier_diagnostic_does_not_poison_a_later_clean_attempt() {
    assert_eq!(
        classify_output(&output(0, b"warning", false))
            .unwrap_err()
            .kind,
        AnalysisErrorKind::BackendFailed
    );
    classify_output(&output(0, b"", false)).unwrap();
}

// M2 acceptance: approved praat-m2-design.md §10, AH-AC05/09/10.
// Kept separate from the existing protocol tests; hooks never bypass EXE pins.
#[cfg(windows)]
mod m2 {
    use super::*;
    use crate::{
        praat::{
            take_praat_cleanup_warning, DigestWriter, PraatBackend, PraatConfig, WavTestPoint,
        },
        AnalysisError, PitchTrack,
    };
    use std::{
        cell::{Cell, RefCell},
        fs::{self, File, OpenOptions},
        io::Read,
        path::{Path, PathBuf},
        rc::Rc,
        sync::Mutex,
    };

    static SERIAL: Mutex<()> = Mutex::new(());
    const SENTINEL: &[u8] = b"M2 synthetic original and prior-owner sentinel";
    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(60)
    }

    #[derive(Default)]
    struct Seen {
        closed: Cell<usize>,
        pin_attempted: Cell<usize>,
        pin_acquired: Cell<usize>,
        chunks: Cell<usize>,
        verified: Cell<usize>,
        analysis: Cell<usize>,
        path: RefCell<Option<PathBuf>>,
    }
    impl Seen {
        fn record(&self, point: &WavTestPoint, path: &Path) {
            let counter = match point {
                WavTestPoint::WavWriterClosed => &self.closed,
                WavTestPoint::WavPinAttempted { acquired } => {
                    if *acquired {
                        self.pin_acquired.set(self.pin_acquired.get() + 1);
                    }
                    &self.pin_attempted
                }
                WavTestPoint::WavHashChunk => &self.chunks,
                WavTestPoint::WavVerified => &self.verified,
                WavTestPoint::BeforeAnalysisRun => &self.analysis,
            };
            counter.set(counter.get() + 1);
            let mut recorded = self.path.borrow_mut();
            if let Some(previous) = recorded.as_ref() {
                assert_eq!(previous, path, "every hook must observe the same WAV");
            } else {
                *recorded = Some(path.to_owned());
            }
        }
        fn successful(&self) {
            assert_eq!(self.closed.get(), 1);
            assert_eq!(self.pin_attempted.get(), 1);
            assert_eq!(self.pin_acquired.get(), 1);
            assert!(
                self.chunks.get() >= 2,
                "128058 bytes require multiple <=64KiB chunks"
            );
            assert_eq!(self.verified.get(), 1);
            assert_eq!(self.analysis.get(), 1);
        }
    }

    struct Fixture {
        root: tempfile::TempDir,
        work: PathBuf,
        backend: PraatBackend,
        samples: Vec<f64>,
    }
    impl Fixture {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            let work = root.path().join("M2 解析 日本語");
            fs::create_dir(&work).unwrap();
            fs::create_dir(work.join("prior owner")).unwrap();
            fs::write(work.join("prior owner/keep.bin"), SENTINEL).unwrap();
            fs::write(root.path().join("original.bin"), SENTINEL).unwrap();
            let executable = std::env::var_os("WW_TEST_PRAAT_EXE")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent().unwrap().parent().unwrap()
                        .join("tasks/ACOUSTIC-HISTORY-001/artifacts/praat-probe/praat7002-x64v1/Praat.exe")
                });
            assert!(executable.is_absolute());
            assert_eq!(
                fs::metadata(&executable)
                    .expect("M2 requires the real pinned Praat fixture")
                    .len(),
                96_780_624
            );
            // The real constructor verifies the fixed SHA and version. A fixture
            // failure here is not a behavioral RED for WAV identity.
            let backend = PraatBackend::new(
                PraatConfig {
                    executable,
                    work_root: work.clone(),
                },
                deadline(),
                &CancellationToken::new(),
            )
            .expect("M2 fixture constructor must verify real pinned Praat");
            let samples = (0..16000)
                .map(|i| 0.4 * (std::f64::consts::TAU * 200.0 * i as f64 / 8000.0).sin())
                .collect();
            Self {
                root,
                work,
                backend,
                samples,
            }
        }
        fn call(
            &self,
            until: Instant,
            cancel: &CancellationToken,
            seen: &Seen,
            action: &dyn Fn(WavTestPoint, &Path) -> Result<(), AnalysisError>,
        ) -> Result<PitchTrack, AnalysisError> {
            let before: Vec<_> = self.samples.iter().map(|v| v.to_bits()).collect();
            let result = self.backend.extract_with_hook(
                MonoPcm {
                    samples: &self.samples,
                    sample_rate: 8000,
                },
                &AnalysisProfile::v1().pitch,
                until,
                cancel,
                &|point, path| {
                    seen.record(&point, path);
                    action(point, path)
                },
            );
            assert_eq!(
                self.samples.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                before
            );
            result
        }
        fn clean(&self, seen: &Seen) {
            assert_eq!(
                fs::read(self.root.path().join("original.bin")).unwrap(),
                SENTINEL
            );
            assert_eq!(
                fs::read(self.work.join("prior owner/keep.bin")).unwrap(),
                SENTINEL
            );
            let names: Vec<_> = fs::read_dir(&self.work)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            assert_eq!(names, vec![std::ffi::OsString::from("prior owner")]);
            if let Some(path) = seen.path.borrow().as_ref() {
                assert!(!path.exists(), "owned WAV must be removed");
                assert!(
                    !path.parent().unwrap().exists(),
                    "owned work directory must be removed"
                );
            }
            assert!(
                !take_praat_cleanup_warning(),
                "normal cleanup must not warn"
            );
        }
        fn retry(&self) {
            let seen = Seen::default();
            let track = self
                .call(deadline(), &CancellationToken::new(), &seen, &|_, _| Ok(()))
                .expect("same backend must recover for a fresh real-Praat call");
            assert_200_hz(&track);
            seen.successful();
            self.clean(&seen);
        }
        fn rejected(
            &self,
            result: Result<PitchTrack, AnalysisError>,
            seen: &Seen,
            kind: AnalysisErrorKind,
        ) {
            self.clean(seen);
            assert_eq!(seen.closed.get(), 1, "fixture must reach writer-close hook");
            assert_eq!(
                seen.analysis.get(),
                0,
                "rejected WAV must not enter analysis supervisor"
            );
            assert_eq!(
                result
                    .expect_err("rejected call must return no PitchTrack/F0")
                    .kind,
                kind
            );
            self.retry();
        }
    }
    fn assert_200_hz(track: &PitchTrack) {
        let mut voiced = 0;
        let mut previous = -1.0;
        for frame in &track.frames {
            assert!((0.0..=2.0).contains(&frame.time_seconds));
            assert!(frame.time_seconds > previous);
            previous = frame.time_seconds;
            match frame.value {
                SampleValue::Finite(hz) => {
                    assert!((195.0..=205.0).contains(&hz));
                    voiced += 1;
                }
                SampleValue::Missing(MissingReason::Unvoiced) => {}
                ref other => panic!("unexpected pitch: {other:?}"),
            }
        }
        assert!(voiced > 0, "synthetic 200Hz must produce voiced frames");
    }

    // Independent RIFF/IEEE-float64 encoding; never uses write_mono_wav.
    fn silence_wav(frames: usize) -> Vec<u8> {
        let payload = u32::try_from(frames * 8).unwrap();
        let mut bytes = b"RIFF".to_vec();
        bytes.extend((50 + payload).to_le_bytes());
        bytes.extend(
            b"WAVEfmt \x12\0\0\0\x03\0\x01\0\x40\x1f\0\0\0\xfa\0\0\x08\0\x40\0\0\0fact\x04\0\0\0",
        );
        bytes.extend((frames as u32).to_le_bytes());
        bytes.extend(b"data");
        bytes.extend(payload.to_le_bytes());
        assert_eq!(bytes.len(), 58);
        bytes.resize(58 + payload as usize, 0); // IEEE-float64 +0.0
        bytes
    }

    fn tamper(replace: bool) {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let f = Fixture::new();
        let seen = Seen::default();
        let changed = Cell::new(false);
        let result = f.call(
            deadline(),
            &CancellationToken::new(),
            &seen,
            &|point, path| {
                if matches!(point, WavTestPoint::WavWriterClosed) {
                    let before = fs::read(path).unwrap();
                    let replacement = silence_wav(f.samples.len());
                    assert_eq!(before.len(), replacement.len());
                    assert_ne!(before, replacement, "fixture must change actual bytes");
                    if replace {
                        let backup = path.with_file_name("owned-before.wav");
                        fs::rename(path, &backup).expect("writer must already be closed");
                        fs::write(path, &replacement).unwrap();
                        assert_eq!(fs::read(backup).unwrap(), before);
                    } else {
                        let mut writer = OpenOptions::new().write(true).open(path).unwrap();
                        writer.write_all(&replacement).unwrap();
                        writer.flush().unwrap();
                        drop(writer);
                    }
                    let actual = fs::read(path).unwrap();
                    assert_eq!(actual.len(), before.len());
                    assert_eq!(actual, replacement);
                    assert_ne!(actual, before);
                    changed.set(true);
                }
                Ok(())
            },
        );
        assert!(changed.get(), "tamper hook must run");
        assert_eq!(
            seen.analysis.get(),
            0,
            "changed WAV must be rejected before analysis (M2 behavioral RED oracle)"
        );
        assert_eq!(seen.pin_attempted.get(), 1);
        assert_eq!(seen.pin_acquired.get(), 1);
        assert!(seen.chunks.get() > 0);
        assert_eq!(seen.verified.get(), 0);
        f.rejected(result, &seen, AnalysisErrorKind::BackendFailed);
    }
    #[test]
    fn t1_same_length_in_place_overwrite_is_rejected_before_analysis_then_retry() {
        tamper(false);
    }
    #[test]
    fn t2_same_length_replacement_is_rejected_before_analysis_then_retry() {
        tamper(true);
    }

    #[test]
    fn t2_identical_byte_replacement_is_allowed() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let f = Fixture::new();
        let seen = Seen::default();
        let replaced = Cell::new(false);
        let track = f
            .call(
                deadline(),
                &CancellationToken::new(),
                &seen,
                &|point, path| {
                    if matches!(point, WavTestPoint::WavWriterClosed) {
                        let bytes = fs::read(path).unwrap();
                        fs::rename(path, path.with_file_name("owned-identical.wav")).unwrap();
                        fs::write(path, &bytes).unwrap();
                        assert_eq!(fs::read(path).unwrap(), bytes);
                        replaced.set(true);
                    }
                    Ok(())
                },
            )
            .unwrap();
        assert!(replaced.get());
        seen.successful();
        assert_200_hz(&track);
        f.clean(&seen);
    }

    #[test]
    fn t3a_verified_pin_allows_read_but_denies_write_rename_and_delete() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let f = Fixture::new();
        let seen = Seen::default();
        let checked = Cell::new(false);
        let track = f
            .call(
                deadline(),
                &CancellationToken::new(),
                &seen,
                &|point, path| {
                    if matches!(point, WavTestPoint::WavVerified) {
                        let mut reader =
                            File::open(path).expect("verified pin must allow a real reader");
                        let mut header = [0; 12];
                        reader.read_exact(&mut header).unwrap();
                        assert_eq!(&header[..4], b"RIFF");
                        assert_eq!(&header[8..], b"WAVE");
                        drop(reader);
                        // Closing our reader first ensures only the production pin can
                        // explain the write/delete/rename denials.
                        assert!(OpenOptions::new().write(true).open(path).is_err());
                        assert!(
                            fs::rename(path, path.with_file_name("must-not-exist.wav")).is_err()
                        );
                        assert!(fs::remove_file(path).is_err());
                        checked.set(true);
                    }
                    Ok(())
                },
            )
            .unwrap();
        assert!(checked.get());
        seen.successful();
        assert_200_hz(&track);
        f.clean(&seen);
    }

    fn conflicting_writer(hook_error: bool) {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let f = Fixture::new();
        let seen = Seen::default();
        let writer = RefCell::new(None::<File>);
        let released = Cell::new(false);
        let result = f.call(
            deadline(),
            &CancellationToken::new(),
            &seen,
            &|point, path| {
                match point {
                    WavTestPoint::WavWriterClosed => {
                        *writer.borrow_mut() =
                            Some(OpenOptions::new().write(true).open(path).unwrap());
                    }
                    WavTestPoint::WavPinAttempted { acquired } => {
                        assert!(
                            !acquired,
                            "an existing writer must prevent read-only pin acquisition"
                        );
                        drop(
                            writer
                                .borrow_mut()
                                .take()
                                .expect("writer held until PinAttempted"),
                        );
                        released.set(true);
                        if hook_error {
                            return Err(AnalysisError {
                                kind: AnalysisErrorKind::InvalidOutput,
                            });
                        }
                    }
                    _ => {}
                }
                Ok(())
            },
        );
        assert!(
            released.get(),
            "failed pin must still call the release hook"
        );
        assert!(writer.borrow().is_none());
        assert_eq!(seen.pin_attempted.get(), 1);
        assert_eq!(seen.pin_acquired.get(), 0);
        assert_eq!(seen.chunks.get(), 0);
        assert_eq!(seen.verified.get(), 0);
        f.rejected(result, &seen, AnalysisErrorKind::BackendFailed);
    }
    #[test]
    fn t3b_existing_writer_blocks_pin_and_is_released_before_cleanup_then_retry() {
        conflicting_writer(false);
    }
    #[test]
    fn t3b_original_pin_error_survives_release_hook_error_then_retry() {
        conflicting_writer(true);
    }

    fn interrupted(at_chunk: bool, timeout: bool, cancel_too: bool) {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let f = Fixture::new();
        let seen = Seen::default();
        let triggered = Cell::new(false);
        let cancel = CancellationToken::new();
        let until = if timeout {
            Instant::now() + Duration::from_secs(2)
        } else {
            deadline()
        };
        let started = Instant::now();
        let result = f.call(until, &cancel, &seen, &|point, _| {
            let target = if at_chunk {
                matches!(point, WavTestPoint::WavHashChunk)
            } else {
                matches!(point, WavTestPoint::WavWriterClosed)
            };
            if target && !triggered.replace(true) {
                assert!(
                    Instant::now() < until,
                    "fixture must reach hook before expiry"
                );
                if timeout {
                    // Wait at the known synchronous point until the *same*
                    // deadline has elapsed; no attempt to race a file operation.
                    while Instant::now() < until {
                        std::thread::park_timeout(until.saturating_duration_since(Instant::now()));
                    }
                    assert!(Instant::now() >= until);
                }
                if !timeout || cancel_too {
                    cancel.cancel();
                }
            }
            Ok(())
        });
        assert!(triggered.get(), "requested interrupt hook must be reached");
        if at_chunk {
            assert_eq!(
                seen.chunks.get(),
                1,
                "checkpoint must follow the interrupted chunk"
            );
        }
        assert_eq!(seen.verified.get(), 0);
        f.rejected(
            result,
            &seen,
            if !timeout || cancel_too {
                AnalysisErrorKind::Cancelled
            } else {
                AnalysisErrorKind::TimedOut
            },
        );
        eprintln!("M2 interrupt at_chunk={at_chunk} timeout={timeout} cancel_too={cancel_too} call_and_retry_ms={}", started.elapsed().as_millis());
    }
    #[test]
    fn t4_cancel_at_writer_closed_cleans_and_retries() {
        interrupted(false, false, false);
    }
    #[test]
    fn t4_cancel_at_hash_chunk_cleans_and_retries() {
        interrupted(true, false, false);
    }
    #[test]
    fn t4_timeout_at_writer_closed_cleans_and_retries() {
        interrupted(false, true, false);
    }
    #[test]
    fn t4_timeout_at_hash_chunk_cleans_and_retries() {
        interrupted(true, true, false);
    }
    #[test]
    fn t4_cancel_precedes_timeout_at_hash_chunk() {
        interrupted(true, true, true);
    }

    // 8kHz mono float64 RIFF, exactly two samples (+0.25, -0.5).
    // SHA calculated independently with .NET SHA256.HashData, not product code.
    const GOLDEN: &[u8] = b"RIFF\x42\0\0\0WAVEfmt \x12\0\0\0\x03\0\x01\0\x40\x1f\0\0\0\xfa\0\0\x08\0\x40\0\0\0fact\x04\0\0\0\x02\0\0\0data\x10\0\0\0\0\0\0\0\0\0\xd0\x3f\0\0\0\0\0\0\xe0\xbf";
    const GOLDEN_SHA: [u8; 32] = [
        0x5b, 0x2b, 0xf4, 0xbe, 0x3b, 0x68, 0xfd, 0xeb, 0x20, 0x64, 0xb2, 0x71, 0xd1, 0xfa, 0xdc,
        0x33, 0xeb, 0xda, 0xa6, 0xae, 0x4e, 0xe8, 0xb8, 0x7e, 0x40, 0xbe, 0x2e, 0xed, 0xb3, 0xd9,
        0xa4, 0xe0,
    ];
    #[derive(Default)]
    struct SinkState {
        bytes: Vec<u8>,
        interrupted: usize,
        flushes: usize,
        dropped: bool,
    }
    struct Sink {
        state: Rc<RefCell<SinkState>>,
        limit: usize,
        interrupt_once: bool,
        zero: bool,
        flush_error: bool,
    }
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.interrupt_once {
                self.interrupt_once = false;
                self.state.borrow_mut().interrupted += 1;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let n = if self.zero {
                0
            } else {
                bytes.len().min(self.limit)
            };
            self.state.borrow_mut().bytes.extend_from_slice(&bytes[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.state.borrow_mut().flushes += 1;
            if self.flush_error {
                Err(io::Error::other("M2_FLUSH_SENTINEL"))
            } else {
                Ok(())
            }
        }
    }
    impl Drop for Sink {
        fn drop(&mut self) {
            self.state.borrow_mut().dropped = true;
        }
    }
    fn sink(limit: usize) -> (Sink, Rc<RefCell<SinkState>>) {
        let state = Rc::new(RefCell::new(SinkState::default()));
        (
            Sink {
                state: state.clone(),
                limit,
                interrupt_once: false,
                zero: false,
                flush_error: false,
            },
            state,
        )
    }
    #[test]
    fn t5_short_writes_hash_only_accepted_prefix_and_finish_drops_inner() {
        for limit in [1, 3, 17, 74, 128] {
            let (inner, state) = sink(limit);
            let mut writer = DigestWriter::new(inner);
            assert_eq!(GOLDEN.len(), 74);
            writer.write_all(GOLDEN).unwrap();
            writer.flush().unwrap();
            assert_eq!(state.borrow().bytes, GOLDEN);
            assert!(!state.borrow().dropped);
            assert_eq!(writer.finish(), GOLDEN_SHA);
            assert!(
                state.borrow().dropped,
                "finish must close writer before returning digest"
            );
            assert_eq!(state.borrow().flushes, 1, "finish must not add a flush");
        }
    }
    #[test]
    fn t5_interrupted_write_is_retried_without_hashing_unaccepted_bytes() {
        let (mut inner, state) = sink(5);
        inner.interrupt_once = true;
        let mut writer = DigestWriter::new(inner);
        writer.write_all(GOLDEN).unwrap();
        writer.flush().unwrap();
        assert_eq!(state.borrow().interrupted, 1);
        assert_eq!(state.borrow().bytes, GOLDEN);
        assert_eq!(writer.finish(), GOLDEN_SHA);
        assert!(state.borrow().dropped);
    }
    #[test]
    fn t5_zero_write_is_write_zero_without_adopting_a_digest() {
        let (mut inner, state) = sink(3);
        inner.zero = true;
        let mut writer = DigestWriter::new(inner);
        assert_eq!(
            writer.write_all(GOLDEN).unwrap_err().kind(),
            io::ErrorKind::WriteZero
        );
        assert!(state.borrow().bytes.is_empty());
        drop(writer); // Contract forbids adopting finish() after an I/O failure.
        assert!(state.borrow().dropped);
    }
    #[test]
    fn t5_flush_error_is_propagated_without_adopting_a_digest() {
        let (mut inner, state) = sink(7);
        inner.flush_error = true;
        let mut writer = DigestWriter::new(inner);
        writer.write_all(GOLDEN).unwrap();
        assert_eq!(writer.flush().unwrap_err().kind(), io::ErrorKind::Other);
        assert_eq!(state.borrow().flushes, 1);
        assert_eq!(state.borrow().bytes, GOLDEN);
        drop(writer);
        assert!(state.borrow().dropped);
    }
}
