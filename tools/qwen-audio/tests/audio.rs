mod support;

use qwen_audio::{read_wav, AudioInput, ErrorCode, ReferenceText, SendDisposition};
use support::{pcm_format, riff, set_u16, set_u32, wav, TempData};

const SIX_MIB: usize = 6 * 1024 * 1024;

fn rejected(bytes: Vec<u8>, code: ErrorCode) {
    let error = AudioInput::parse(bytes).err().expect("must reject input");
    assert_eq!(error.code(), code);
    assert_eq!(error.disposition(), SendDisposition::NotSent);
}

#[test]
fn pcm16_mono_and_stereo_report_frames_not_samples_and_preserve_bytes() {
    for channels in [1, 2] {
        let bytes = wav(channels, 16_000, 24_000);
        let input = AudioInput::parse(bytes.clone()).expect("valid synthetic WAV");
        assert_eq!(input.bytes(), bytes);
        assert_eq!(input.info().channels, channels);
        assert_eq!(input.info().sample_rate, 16_000);
        assert_eq!(input.info().frames, 24_000);
        assert_eq!(input.info().duration_seconds, 1.5);
        assert_eq!(input.info().byte_len, bytes.len());
    }
}

#[test]
fn silence_and_one_frame_are_valid_local_inputs() {
    let input = AudioInput::parse(wav(1, 8_000, 1)).expect("silence is not invalid");
    assert_eq!(input.info().frames, 1);
}

#[test]
fn sample_rate_endpoints_are_inclusive() {
    for sample_rate in [8_000, 48_000] {
        assert!(AudioInput::parse(wav(1, sample_rate, 1)).is_ok());
    }
}

#[test]
fn sample_rates_immediately_outside_range_are_rejected() {
    for sample_rate in [7_999, 48_001] {
        rejected(wav(1, sample_rate, 1), ErrorCode::AudioUnsupported);
    }
}

#[test]
fn sixty_seconds_is_valid_but_one_more_frame_is_not() {
    for channels in [1, 2] {
        assert!(AudioInput::parse(wav(channels, 8_000, 8_000 * 60)).is_ok());
        rejected(
            wav(channels, 8_000, 8_000 * 60 + 1),
            ErrorCode::AudioTooLong,
        );
    }
}

#[test]
fn sixty_one_seconds_is_rejected_before_any_send() {
    rejected(wav(1, 8_000, 8_000 * 61), ErrorCode::AudioTooLong);
}

#[test]
fn forty_eight_khz_sixty_seconds_mono_fits_but_stereo_exceeds_size_limit() {
    assert!(AudioInput::parse(wav(1, 48_000, 48_000 * 60)).is_ok());
    rejected(wav(2, 48_000, 48_000 * 60), ErrorCode::AudioTooLarge);
}

#[test]
fn six_mib_including_unknown_chunks_is_inclusive() {
    let format = pcm_format(1, 8_000);
    let samples = [0_u8; 2];
    // RIFF 12 + fmt 24 + JUNK header 8 + data 10 = 54 bytes.
    let junk = vec![0; SIX_MIB - 54];
    let bytes = riff(&[(b"fmt ", &format), (b"JUNK", &junk), (b"data", &samples)]);
    assert_eq!(bytes.len(), SIX_MIB, "fixture size must hit exact boundary");
    assert!(AudioInput::parse(bytes).is_ok());
}

#[test]
fn one_byte_above_file_limit_is_rejected_even_with_a_valid_declared_length() {
    let mut bytes = wav(1, 8_000, 1);
    bytes.resize(SIX_MIB + 1, 0);
    let declared = (bytes.len() - 8) as u32;
    set_u32(&mut bytes, 4, declared);
    rejected(bytes, ErrorCode::AudioTooLarge);
}

#[test]
fn zero_frames_is_not_a_recording() {
    rejected(wav(1, 8_000, 0), ErrorCode::AudioEmpty);
}

#[test]
fn empty_file_and_non_wave_bytes_are_rejected() {
    assert!(AudioInput::parse(Vec::new()).is_err());
    rejected(b"not an audio file".to_vec(), ErrorCode::AudioCorrupt);
    let mut bytes = wav(1, 8_000, 1);
    bytes[8..12].copy_from_slice(b"NOPE");
    rejected(bytes, ErrorCode::AudioCorrupt);
}

#[test]
fn riff_declared_length_must_equal_actual_file_length() {
    for difference in [-1_i64, 1] {
        let mut bytes = wav(1, 8_000, 1);
        let declared = (bytes.len() as i64 - 8 + difference) as u32;
        set_u32(&mut bytes, 4, declared);
        rejected(bytes, ErrorCode::AudioCorrupt);
    }
}

#[test]
fn chunk_overflow_and_partial_header_are_rejected_without_panicking() {
    let mut bytes = wav(1, 8_000, 1);
    set_u32(&mut bytes, 40, u32::MAX);
    rejected(bytes, ErrorCode::AudioCorrupt);
    let mut bytes = wav(1, 8_000, 1);
    bytes.extend_from_slice(b"JUNK\0\0");
    let declared = (bytes.len() - 8) as u32;
    set_u32(&mut bytes, 4, declared);
    rejected(bytes, ErrorCode::AudioCorrupt);
}

#[test]
fn exactly_one_fmt_and_data_chunk_are_required() {
    let format = pcm_format(1, 8_000);
    let samples = [0_u8; 2];
    for bytes in [
        riff(&[(b"data", &samples)]),
        riff(&[(b"fmt ", &format)]),
        riff(&[(b"fmt ", &format), (b"fmt ", &format), (b"data", &samples)]),
        riff(&[(b"fmt ", &format), (b"data", &samples), (b"data", &samples)]),
    ] {
        rejected(bytes, ErrorCode::AudioCorrupt);
    }
}

#[test]
fn unknown_odd_chunk_requires_padding_and_may_precede_format() {
    let format = pcm_format(1, 8_000);
    let bytes = riff(&[(b"JUNK", &[7]), (b"fmt ", &format), (b"data", &[0, 0])]);
    assert!(AudioInput::parse(bytes).is_ok());
    let mut missing_pad = riff(&[(b"fmt ", &format), (b"data", &[0, 0]), (b"JUNK", &[7])]);
    missing_pad.pop();
    let declared = (missing_pad.len() - 8) as u32;
    set_u32(&mut missing_pad, 4, declared);
    rejected(missing_pad, ErrorCode::AudioCorrupt);
}

#[test]
fn incomplete_fmt_data_and_frame_alignment_are_rejected() {
    let format = pcm_format(2, 8_000);
    rejected(
        riff(&[(b"fmt ", &format[..15]), (b"data", &[0; 4])]),
        ErrorCode::AudioCorrupt,
    );
    rejected(
        riff(&[(b"fmt ", &format), (b"data", &[0; 3])]),
        ErrorCode::AudioCorrupt,
    );
    let mut bytes = wav(2, 8_000, 1);
    bytes.pop();
    let declared = (bytes.len() - 8) as u32;
    set_u32(&mut bytes, 4, declared);
    rejected(bytes, ErrorCode::AudioCorrupt);
}

#[test]
fn byte_rate_and_block_align_must_agree_with_pcm_frames() {
    let mut wrong_rate = wav(2, 8_000, 1);
    set_u32(&mut wrong_rate, 28, 16_000);
    rejected(wrong_rate, ErrorCode::AudioCorrupt);
    let mut wrong_align = wav(2, 8_000, 1);
    set_u16(&mut wrong_align, 32, 2);
    rejected(wrong_align, ErrorCode::AudioCorrupt);
}

#[test]
fn float_compressed_extensible_bit_depth_and_extra_channels_are_unsupported() {
    for tag in [3_u16, 6, 0xfffe] {
        let mut bytes = wav(1, 8_000, 1);
        set_u16(&mut bytes, 20, tag);
        rejected(bytes, ErrorCode::AudioUnsupported);
    }
    for bits in [8_u16, 24, 32] {
        let mut bytes = wav(1, 8_000, 1);
        set_u16(&mut bytes, 34, bits);
        rejected(bytes, ErrorCode::AudioUnsupported);
    }
    rejected(wav(3, 8_000, 1), ErrorCode::AudioUnsupported);
    let mut rf64 = wav(1, 8_000, 1);
    rf64[..4].copy_from_slice(b"RF64");
    assert!(AudioInput::parse(rf64).is_err());
}

#[test]
fn reference_limit_counts_unicode_scalars_and_preserves_whitespace_and_newlines() {
    let text = format!(" \n{}\t", "🌱".repeat(9_997));
    assert_eq!(text.chars().count(), 10_000);
    let reference = ReferenceText::new(text.clone()).expect("10,000 Unicode scalars");
    assert_eq!(reference.as_str(), text);
    let error = ReferenceText::new("🌱".repeat(10_001))
        .err()
        .expect("limit exceeded");
    assert_eq!(error.code(), ErrorCode::InvalidReference);
    assert_eq!(error.disposition(), SendDisposition::NotSent);
}

#[test]
fn empty_and_unicode_whitespace_only_references_are_invalid() {
    for text in ["", " \r\n\t", "\u{2003}\u{3000}\u{a0}"] {
        let error = ReferenceText::new(text.to_owned())
            .err()
            .expect("blank reference");
        assert_eq!(error.code(), ErrorCode::InvalidReference);
        assert_eq!(error.disposition(), SendDisposition::NotSent);
    }
}

#[test]
fn read_wav_inspects_bytes_regardless_of_extension_and_does_not_modify_source() {
    let temp = TempData::new();
    let bytes = wav(1, 8_000, 8);
    let path = temp.file("synthetic.bin", &bytes);
    assert_eq!(read_wav(&path).expect("actual RIFF content").bytes(), bytes);
    assert_eq!(std::fs::read(path).expect("source retained"), bytes);
    let fake = temp.file("fake.wav", b"not WAV");
    assert!(read_wav(&fake).is_err());
}

#[test]
fn read_snapshot_survives_source_replacement_and_deletion() {
    let temp = TempData::new();
    let original = wav(1, 8_000, 8);
    let path = temp.file("synthetic.wav", &original);
    let snapshot = read_wav(&path).expect("snapshot read");
    std::fs::write(&path, wav(2, 16_000, 16)).expect("replace owned source");
    assert_eq!(snapshot.bytes(), original);
    assert_eq!(snapshot.info().sample_rate, 8_000);
    std::fs::remove_file(&path).expect("delete owned source");
    assert_eq!(snapshot.bytes(), original);
    assert_eq!(snapshot.clone().bytes(), original);
}

#[test]
fn unreadable_source_error_does_not_expose_local_path() {
    let temp = TempData::new();
    let path = temp.path().join("private-path-sentinel.wav");
    let error = read_wav(&path).err().expect("missing owned source");
    assert_eq!(error.code(), ErrorCode::AudioIo);
    assert_eq!(error.disposition(), SendDisposition::NotSent);
    assert!(!format!("{error} {error:?}").contains("private-path-sentinel"));
}
