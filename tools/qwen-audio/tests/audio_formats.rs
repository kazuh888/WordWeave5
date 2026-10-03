#[path = "support/provider.rs"]
mod provider_fixture;
#[cfg(feature = "gui")]
#[path = "fixtures/reuse/mod.rs"]
mod reuse;
mod support;
use base64::Engine;
use qwen_audio::*;
use tokio_util::sync::CancellationToken;

const FIXTURES: &[(&str, &[u8], &str)] = &[
    ("wav", include_bytes!("fixtures/formats/tone.wav"), "wav"),
    ("mp3", include_bytes!("fixtures/formats/tone.mp3"), "mp3"),
    ("aac", include_bytes!("fixtures/formats/tone.aac"), "aac"),
    ("amr", include_bytes!("fixtures/formats/tone.amr"), "amr"),
    ("3gp", include_bytes!("fixtures/formats/tone.3gp"), "3gp"),
    ("3gpp", include_bytes!("fixtures/formats/tone.3gpp"), "3gp"),
];

#[test]
fn six_formats_preserve_encoded_bytes_and_provide_playable_pcm() {
    for (name, original, wire) in FIXTURES {
        let input = AudioInput::decode(original.to_vec(), &CancellationToken::new())
            .unwrap_or_else(|e| {
                panic!("{name}: {e}; FFmpeg/ffprobe must be installed for this test")
            });
        assert_eq!(
            input.bytes(),
            *original,
            "{name}: original must not be transcoded"
        );
        assert_eq!(input.wire_format(), *wire);
        assert!(input.info().duration_seconds >= 0.19 && input.info().duration_seconds < 0.5);
        assert_eq!(input.info().byte_len, original.len());
        let preview = AudioInput::parse(input.playback_wav().to_vec()).expect("PCM16 preview");
        assert_eq!(preview.info().sample_rate, input.info().sample_rate);
        assert_eq!(preview.info().channels, input.info().channels);
        assert_eq!(preview.info().frames, input.info().frames);
    }
}

#[tokio::test]
async fn each_format_sends_its_original_bytes_and_matching_wire_format_once() {
    use provider_fixture::*;
    for (name, original, format) in FIXTURES {
        let input = AudioInput::decode(original.to_vec(), &CancellationToken::new()).unwrap();
        let snapshot = EvaluationSnapshot::new(
            input,
            ReferenceText::new(REFERENCE.into()).unwrap(),
            connection(),
        )
        .unwrap();
        let wire = ScriptTransport::success();
        evaluate(snapshot, wire.clone(), CancellationToken::new())
            .await
            .unwrap();
        let requests = wire.requests.lock().unwrap();
        assert_eq!(requests.len(), 1, "{name}");
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        let audio = &body["messages"][1]["content"][1]["input_audio"];
        assert_eq!(audio["format"], *format);
        let data = audio["data"].as_str().unwrap().split_once(',').unwrap().1;
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(data)
                .unwrap(),
            *original
        );
    }
}

#[test]
fn cancelled_decode_and_oversized_input_are_not_sendable() {
    let token = CancellationToken::new();
    token.cancel();
    assert_eq!(
        AudioInput::decode(FIXTURES[1].1.to_vec(), &token)
            .unwrap_err()
            .code(),
        ErrorCode::Cancelled
    );
    assert_eq!(
        AudioInput::decode(vec![0; MAX_AUDIO_BYTES + 1], &CancellationToken::new())
            .unwrap_err()
            .code(),
        ErrorCode::AudioTooLarge
    );
    for bytes in [
        b"not audio".to_vec(),
        b"#!AMR\n".to_vec(),
        b"ID3\x04\0\0\0\0\0\0garbage".to_vec(),
    ] {
        assert!(AudioInput::decode(bytes, &CancellationToken::new()).is_err());
    }
}

#[test]
fn file_selection_uses_content_and_snapshot_survives_source_removal() {
    let temp = support::TempData::new();
    let original = FIXTURES[1].1;
    let path = temp.file("合成 音声.wav", original);
    let input = read_audio(&path, &CancellationToken::new()).unwrap();
    assert_eq!(input.wire_format(), "mp3");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    std::fs::remove_file(path).unwrap();
    assert_eq!(input.bytes(), original);
    assert!(!input.playback_wav().is_empty());
}

#[test]
fn cancelled_load_job_never_publishes_a_ready_input() {
    let temp = support::TempData::new();
    let path = temp.file("synthetic.mp3", FIXTURES[1].1);
    let mut job = AudioLoadJob::start(path).unwrap();
    job.cancel();
    for _ in 0..100 {
        if let Some(result) = job.try_take() {
            assert_eq!(result.unwrap_err().code(), ErrorCode::Cancelled);
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("cancelled job must settle");
}

#[test]
fn compressed_inputs_are_checked_to_the_end_without_changing_rate() {
    for (_, original, _) in &FIXTURES[1..] {
        let mut truncated = original.to_vec();
        truncated.pop();
        assert!(AudioInput::decode(truncated, &CancellationToken::new()).is_err());
    }
    for (first, second) in [
        (
            FIXTURES[1].1,
            include_bytes!("fixtures/formats/rate22050.mp3").as_slice(),
        ),
        (
            FIXTURES[2].1,
            include_bytes!("fixtures/formats/rate22050.aac").as_slice(),
        ),
    ] {
        let mut mixed = first.to_vec();
        mixed.extend_from_slice(second);
        assert!(AudioInput::decode(mixed, &CancellationToken::new()).is_err());
    }
    assert!(AudioInput::decode(
        include_bytes!("fixtures/formats/video.3gp").to_vec(),
        &CancellationToken::new()
    )
    .is_err());
}

#[test]
fn three_gp_external_reference_and_multiple_descriptions_are_rejected() {
    let original = include_bytes!("fixtures/formats/tone.3gp");
    let mut external = original.to_vec();
    let url = external.windows(4).position(|b| b == b"url ").unwrap();
    // Clear self-contained flag: even an otherwise decodable local mdat must not hide an external dref.
    external[url + 4..url + 8].copy_from_slice(&[0; 4]);
    assert!(AudioInput::decode(external, &CancellationToken::new()).is_err());
    let mut multi = original.to_vec();
    let stsd = multi.windows(4).position(|b| b == b"stsd").unwrap();
    multi[stsd + 8..stsd + 12].copy_from_slice(&2u32.to_be_bytes());
    assert!(AudioInput::decode(multi, &CancellationToken::new()).is_err());
    assert_eq!(
        AudioInput::decode(original.to_vec(), &CancellationToken::new())
            .unwrap()
            .bytes(),
        original
    );
}

#[test]
fn single_description_three_gp_with_midstream_channel_change_is_rejected() {
    // Copy-muxed mono then stereo AAC; one stsd. FFmpeg ashowinfo independently reports 1 -> 2 channels.
    let bytes = include_bytes!("fixtures/formats/changed-channels.3gp");
    let stsd = bytes.windows(4).position(|b| b == b"stsd").unwrap();
    assert_eq!(&bytes[stsd + 8..stsd + 12], &1u32.to_be_bytes());
    assert!(AudioInput::decode(bytes.to_vec(), &CancellationToken::new()).is_err());
}

#[test]
fn decoded_preview_limit_is_separate_from_original_file_limit() {
    let original = include_bytes!("fixtures/formats/large-preview.aac");
    assert!(original.len() < MAX_AUDIO_BYTES);
    let input = AudioInput::decode(original.to_vec(), &CancellationToken::new()).unwrap();
    assert!(input.playback_wav().len() > MAX_AUDIO_BYTES);
    assert_eq!(input.bytes(), original);
    assert_eq!(input.info().sample_rate, 48_000);
    assert_eq!(input.info().channels, 2);
    assert!(input.info().duration_seconds > 39.9 && input.info().duration_seconds < 41.0);
    let error = AudioInput::decode(
        include_bytes!("fixtures/formats/too-long.aac").to_vec(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(error.code(), ErrorCode::AudioTooLong);
}

#[test]
fn amr_wideband_sid_frames_are_supported_without_real_voice() {
    // RFC 4867 storage header; ten 20 ms AMR-WB SID (FT=9, Q=1) frames.
    let mut bytes = b"#!AMR-WB\n".to_vec();
    for _ in 0..10 {
        bytes.extend_from_slice(&[0x4c, 0, 0, 0, 0, 0]);
    }
    let input = AudioInput::decode(bytes.clone(), &CancellationToken::new()).unwrap();
    assert_eq!(input.wire_format(), "amr");
    assert_eq!(input.bytes(), bytes);
    assert_eq!(input.info().sample_rate, 16_000);
    assert_eq!(input.info().frames, 3200);
}

#[test]
fn compressed_sixty_seconds_is_accepted_and_one_codec_frame_more_is_rejected() {
    let mut exact = b"#!AMR-WB\n".to_vec();
    for _ in 0..3000 {
        exact.extend_from_slice(&[0x4c, 0, 0, 0, 0, 0]);
    }
    let input = AudioInput::decode(exact.clone(), &CancellationToken::new()).unwrap();
    assert_eq!(input.info().frames, 960_000);
    assert_eq!(input.info().duration_seconds, 60.0);
    exact.extend_from_slice(&[0x4c, 0, 0, 0, 0, 0]);
    assert_eq!(
        AudioInput::decode(exact, &CancellationToken::new())
            .unwrap_err()
            .code(),
        ErrorCode::AudioTooLong
    );
}

#[test]
fn amr_wideband_in_three_gp_preserves_container_bytes() {
    let mut source = b"#!AMR-WB\n".to_vec();
    for _ in 0..10 {
        source.extend_from_slice(&[0x4c, 0, 0, 0, 0, 0]);
    }
    let temp = support::TempData::new();
    let amr = temp.file("synthetic-wb.amr", &source);
    let container = amr.with_extension("3gp");
    let program = std::env::split_paths(&std::env::var_os("PATH").expect("FFmpeg PATH"))
        .filter(|p| p.is_absolute())
        .map(|p| {
            p.join(if cfg!(windows) {
                "ffmpeg.exe"
            } else {
                "ffmpeg"
            })
        })
        .find(|p| p.is_file())
        .expect("FFmpeg is required to construct synthetic WB-in-3GP");
    let mut command = std::process::Command::new(program);
    command
        .env_remove("FFREPORT")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "amr",
            "-i",
        ])
        .arg(&amr)
        .args(["-c:a", "copy", "-f", "3gp"])
        .arg(&container)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    assert!(
        command.status().unwrap().success(),
        "synthetic WB container mux must succeed"
    );
    let original = std::fs::read(&container).unwrap();
    let input = read_audio(&container, &CancellationToken::new()).unwrap();
    assert_eq!(input.wire_format(), "3gp");
    assert_eq!(input.bytes(), original);
    assert_eq!(input.info().sample_rate, 16_000);
    assert_eq!(input.info().frames, 3200);
}

#[cfg(feature = "gui")]
#[tokio::test]
async fn standalone_consumer_uses_all_six_formats_and_failed_reselection_clears_input() {
    use provider_fixture::*;
    for (name, original, format) in FIXTURES {
        let wire = ScriptTransport::success();
        let mut gui =
            gui::GuiController::new(REFERENCE.into(), reuse::Store::configured(), wire.clone())
                .unwrap();
        let input = AudioInput::decode(original.to_vec(), &CancellationToken::new()).unwrap();
        gui.select_audio(input, format!("synthetic.{name}"))
            .unwrap();
        assert_eq!(gui.model().audio_format, Some(*format));
        gui.clear_audio().unwrap();
        assert!(gui.model().audio_info.is_none());
        assert!(gui.send(&tokio::runtime::Handle::current()).is_err());
        assert!(wire.requests.lock().unwrap().is_empty());
        let input = AudioInput::decode(original.to_vec(), &CancellationToken::new()).unwrap();
        gui.select_audio(input, format!("synthetic.{name}"))
            .unwrap();
        gui.send(&tokio::runtime::Handle::current()).unwrap();
        gui.wait_for_terminal().await.unwrap();
        let requests = wire.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        let input = &body["messages"][1]["content"][1]["input_audio"];
        assert_eq!(input["format"], *format);
        let data = input["data"].as_str().unwrap().split_once(',').unwrap().1;
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(data)
                .unwrap(),
            *original
        );
    }
}
