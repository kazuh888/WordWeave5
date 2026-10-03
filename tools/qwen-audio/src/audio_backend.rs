use crate::{
    audio::{check_cancel, AudioFormat},
    error::err,
    ErrorCode, SafeError,
};
use std::{
    ffi::OsString,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

static CLEANUP_WARNING: AtomicBool = AtomicBool::new(false);
pub fn take_audio_cleanup_warning() -> bool {
    CLEANUP_WARNING.swap(false, Ordering::AcqRel)
}

struct Backend {
    probe: PathBuf,
    decode: PathBuf,
}
impl Backend {
    fn discover() -> Result<Self, SafeError> {
        let mut directories = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                directories.push(parent.join("media-tools"));
            }
        }
        if let Some(path) = std::env::var_os("PATH") {
            directories.extend(std::env::split_paths(&path).filter(|p| p.is_absolute()));
        }
        Self::in_directories(directories)
    }
    fn in_directories(directories: impl IntoIterator<Item = PathBuf>) -> Result<Self, SafeError> {
        for directory in directories {
            if !directory.is_absolute() {
                continue;
            }
            let suffix = std::env::consts::EXE_SUFFIX;
            let probe = directory.join(format!("ffprobe{suffix}"));
            let decode = directory.join(format!("ffmpeg{suffix}"));
            if probe.is_file() && decode.is_file() {
                return Ok(Self { probe, decode });
            }
        }
        Err(err(ErrorCode::AudioBackendUnavailable))
    }
}

struct Input<'a> {
    bytes: &'a [u8],
    path: Option<PathBuf>,
    format: AudioFormat,
}
impl Input<'_> {
    fn arguments(&self) -> Vec<OsString> {
        let demux = match self.format {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Aac => "aac",
            AudioFormat::Amr => "amr",
            AudioFormat::ThreeGp => "mov",
            AudioFormat::Wav => unreachable!(),
        };
        let mut args: Vec<OsString> = [
            "-protocol_whitelist",
            if self.path.is_some() {
                "file,pipe"
            } else {
                "pipe"
            },
            "-f",
            demux,
        ]
        .iter()
        .map(OsString::from)
        .collect();
        if self.path.is_some() {
            args.extend(
                ["-enable_drefs", "0", "-use_absolute_path", "0"]
                    .iter()
                    .map(OsString::from),
            );
        }
        args.push("-i".into());
        args.push(
            self.path
                .as_ref()
                .map(|p| p.as_os_str().to_owned())
                .unwrap_or_else(|| "pipe:0".into()),
        );
        args
    }
}

pub(crate) fn decode(
    bytes: &[u8],
    format: AudioFormat,
    cancel: &CancellationToken,
) -> Result<(Vec<u8>, u32, u16), SafeError> {
    check_cancel(cancel)?;
    let backend = Backend::discover()?;
    if format == AudioFormat::ThreeGp {
        let temporary = tempfile::Builder::new()
            .prefix("wordweave-audio-")
            .tempdir()
            .map_err(|_| err(ErrorCode::AudioIo))?;
        let path = temporary.path().join("input.3gp");
        let result = (|| {
            check_cancel(cancel)?;
            // No user path reaches the subprocess. Both invocations use this immutable copy.
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|_| err(ErrorCode::AudioIo))?;
            file.write_all(bytes).map_err(|_| err(ErrorCode::AudioIo))?;
            drop(file);
            decode_input(
                &backend,
                &Input {
                    bytes,
                    path: Some(path),
                    format,
                },
                cancel,
            )
        })();
        finish_cleanup(result, || temporary.close(), &CLEANUP_WARNING)
    } else {
        decode_input(
            &backend,
            &Input {
                bytes,
                path: None,
                format,
            },
            cancel,
        )
    }
}

fn finish_cleanup<T>(
    result: Result<T, SafeError>,
    close: impl FnOnce() -> std::io::Result<()>,
    warning: &AtomicBool,
) -> Result<T, SafeError> {
    if close().is_err() {
        warning.store(true, Ordering::Release);
        Err(err(ErrorCode::AudioCleanup))
    } else {
        result
    }
}

fn decode_input(
    backend: &Backend,
    input: &Input<'_>,
    cancel: &CancellationToken,
) -> Result<(Vec<u8>, u32, u16), SafeError> {
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut args: Vec<OsString> = [
        "-v",
        "error",
        "-show_entries",
        "stream=codec_name,codec_type,sample_rate,channels",
        "-of",
        "json",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    args.extend(input.arguments());
    let json = run(
        &backend.probe,
        &args,
        input,
        64 * 1024,
        None,
        deadline,
        cancel,
    )?;
    let (rate, channels, wideband) = probe_info(&json, input.format)?;
    // AAC can signal decoded attributes in-band (e.g. SBR), beyond its ADTS header.
    let audit =
        matches!(input.format, AudioFormat::ThreeGp | AudioFormat::Aac).then_some((rate, channels));
    let mut args: Vec<OsString> = [
        "-hide_banner",
        "-nostdin",
        "-nostats",
        "-loglevel",
        if audit.is_some() { "info" } else { "error" },
        "-xerror",
        "-err_detect",
        "explode",
        "-reinit_filter",
        "0",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    // FFmpeg's native AMR-WB decoder does not implement SID/DTX frames.
    if wideband {
        args.extend(["-c:a", "libopencore_amrwb"].iter().map(OsString::from));
    }
    args.extend(input.arguments());
    args.extend(
        ["-map", "0:a:0", "-vn", "-sn", "-dn"]
            .iter()
            .map(OsString::from),
    );
    if audit.is_some() {
        args.extend(["-af", "ashowinfo"].iter().map(OsString::from));
    }
    args.extend(
        ["-c:a", "pcm_s16le", "-f", "s16le", "pipe:1"]
            .iter()
            .map(OsString::from),
    );
    let align = usize::from(channels) * 2;
    let maximum = rate as usize * align * 60;
    let pcm = run(
        &backend.decode,
        &args,
        input,
        maximum + align,
        audit,
        deadline,
        cancel,
    )?;
    if pcm.len() > maximum {
        return Err(err(ErrorCode::AudioTooLong));
    }
    if pcm.is_empty() {
        return Err(err(ErrorCode::AudioEmpty));
    }
    if pcm.len() % align != 0 {
        return Err(err(ErrorCode::AudioCorrupt));
    }
    Ok((pcm, rate, channels))
}

fn probe_info(bytes: &[u8], format: AudioFormat) -> Result<(u32, u16, bool), SafeError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| err(ErrorCode::AudioDecodeFailed))?;
    let streams = value["streams"]
        .as_array()
        .ok_or_else(|| err(ErrorCode::AudioDecodeFailed))?;
    if streams.len() != 1 {
        return Err(err(ErrorCode::AudioUnsupported));
    }
    let stream = &streams[0];
    let codec = stream["codec_name"].as_str().unwrap_or("");
    let allowed = match format {
        AudioFormat::Mp3 => codec == "mp3",
        AudioFormat::Aac => codec == "aac",
        AudioFormat::Amr => ["amr_nb", "amr_wb"].contains(&codec),
        AudioFormat::ThreeGp => ["aac", "amr_nb", "amr_wb"].contains(&codec),
        AudioFormat::Wav => false,
    };
    if stream["codec_type"] != "audio" || !allowed {
        return Err(err(ErrorCode::AudioUnsupported));
    }
    let rate = stream["sample_rate"]
        .as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| err(ErrorCode::AudioCorrupt))?;
    let channels = stream["channels"]
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .ok_or_else(|| err(ErrorCode::AudioCorrupt))?;
    if !(8000..=48000).contains(&rate) || !(1..=2).contains(&channels) {
        return Err(err(ErrorCode::AudioUnsupported));
    }
    Ok((rate, channels, codec == "amr_wb"))
}

fn run(
    program: &Path,
    args: &[OsString],
    input: &Input<'_>,
    limit: usize,
    audit: Option<(u32, u16)>,
    deadline: Instant,
    cancel: &CancellationToken,
) -> Result<Vec<u8>, SafeError> {
    check_cancel(cancel)?;
    if Instant::now() >= deadline {
        return Err(err(ErrorCode::AudioDecodeTimeout));
    }
    let mut command = Command::new(program);
    // A host-level FFREPORT must not create a diagnostic copy of private input metadata.
    command
        .env_remove("FFREPORT")
        .env_remove("AV_LOG_FORCE_COLOR")
        .env("AV_LOG_FORCE_NOCOLOR", "1");
    command
        .args(args)
        .stdin(if input.path.is_none() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(if audit.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    supervise(command, input, limit, audit, deadline, cancel, || {})
}

fn supervise(
    mut command: Command,
    input: &Input<'_>,
    limit: usize,
    audit: Option<(u32, u16)>,
    deadline: Instant,
    cancel: &CancellationToken,
    spawned: impl FnOnce(),
) -> Result<Vec<u8>, SafeError> {
    let mut child = command
        .spawn()
        .map_err(|_| err(ErrorCode::AudioBackendUnavailable))?;
    let stdin = child.stdin.take();
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take();
    let fault = AtomicBool::new(false);
    // Scoped pipe workers borrow the snapshot. The supervisor always reaps before joining them.
    thread::scope(|scope| {
        let writer = thread::Builder::new()
            .spawn_scoped(scope, || -> std::io::Result<()> {
                if let Some(mut stdin) = stdin {
                    stdin.write_all(input.bytes)?;
                }
                Ok(())
            })
            .map_err(|_| stop_child(&mut child))?;
        let reader = thread::Builder::new()
            .spawn_scoped(scope, || read_bounded(stdout, limit, &fault))
            .map_err(|_| stop_child(&mut child))?;
        let auditor = thread::Builder::new()
            .spawn_scoped(scope, || -> Result<(), SafeError> {
                if let (Some(stderr), Some(attributes)) = (stderr, audit) {
                    audit_frames(stderr, attributes, &fault)
                } else {
                    Ok(())
                }
            })
            .map_err(|_| stop_child(&mut child))?;
        spawned();
        let mut result = loop {
            if cancel.is_cancelled() {
                break Err(err(ErrorCode::Cancelled));
            }
            if Instant::now() >= deadline {
                break Err(err(ErrorCode::AudioDecodeTimeout));
            }
            if fault.load(Ordering::Acquire) {
                break Err(err(ErrorCode::AudioDecodeFailed));
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    break if status.success() {
                        Ok(())
                    } else {
                        Err(err(ErrorCode::AudioDecodeFailed))
                    }
                }
                Ok(None) => thread::sleep(Duration::from_millis(5)),
                Err(_) => break Err(err(ErrorCode::AudioDecodeFailed)),
            }
        };
        if result.is_err() {
            let _ = child.kill();
        }
        if child.wait().is_err() {
            result = Err(err(ErrorCode::AudioDecodeFailed));
        }
        let wrote = writer.join();
        let read = reader.join();
        let audited = auditor.join();
        check_cancel(cancel)?;
        if result
            .as_ref()
            .is_err_and(|e| e.code() == ErrorCode::AudioDecodeTimeout)
        {
            return Err(err(ErrorCode::AudioDecodeTimeout));
        }
        // An output limit remains a duration error even if the supervisor killed the child.
        let bytes = read
            .map_err(|_| err(ErrorCode::AudioDecodeFailed))?
            .map_err(|error| {
                if limit == 64 * 1024 && error.code() == ErrorCode::AudioTooLong {
                    err(ErrorCode::AudioDecodeFailed)
                } else {
                    error
                }
            })?;
        audited.map_err(|_| err(ErrorCode::AudioDecodeFailed))??;
        result?;
        if let Err(error) = wrote.map_err(|_| err(ErrorCode::AudioDecodeFailed))? {
            // Stream probing may finish after a prefix. The full framing scan and decode
            // remain mandatory; a successful probe need not consume the entire pipe.
            if !(limit == 64 * 1024 && error.kind() == std::io::ErrorKind::BrokenPipe) {
                return Err(err(ErrorCode::AudioDecodeFailed));
            }
        }
        Ok(bytes)
    })
}

fn stop_child(child: &mut std::process::Child) -> SafeError {
    let _ = child.kill();
    let _ = child.wait();
    err(ErrorCode::AudioDecodeFailed)
}

fn read_bounded(
    mut stream: impl Read,
    maximum: usize,
    fault: &AtomicBool,
) -> Result<Vec<u8>, SafeError> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let n = stream.read(&mut chunk).map_err(|_| {
            fault.store(true, Ordering::Release);
            err(ErrorCode::AudioDecodeFailed)
        })?;
        if n == 0 {
            return Ok(bytes);
        }
        if bytes.len() + n > maximum {
            fault.store(true, Ordering::Release);
            return Err(err(ErrorCode::AudioTooLong));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
}

// Only 3GP needs decoded-frame attribute auditing. No raw diagnostic is retained or exposed.
fn audit_frames(
    mut stream: impl Read,
    expected: (u32, u16),
    fault: &AtomicBool,
) -> Result<(), SafeError> {
    let result = (|| {
        let mut line = Vec::new();
        let mut chunk = [0; 4096];
        let mut total = 0usize;
        let mut frames = 0usize;
        loop {
            let n = stream
                .read(&mut chunk)
                .map_err(|_| err(ErrorCode::AudioDecodeFailed))?;
            if n == 0 {
                break;
            }
            total += n;
            if total > 2 * 1024 * 1024 {
                return Err(err(ErrorCode::AudioDecodeFailed));
            }
            for byte in &chunk[..n] {
                if *byte == b'\n' || *byte == b'\r' {
                    frames += usize::from(audit_line(&line, expected)?);
                    line.clear();
                } else {
                    if line.len() >= 4096 {
                        return Err(err(ErrorCode::AudioDecodeFailed));
                    }
                    line.push(*byte);
                }
            }
        }
        frames += usize::from(audit_line(&line, expected)?);
        if frames == 0 {
            return Err(err(ErrorCode::AudioDecodeFailed));
        }
        Ok(())
    })();
    if result.is_err() {
        fault.store(true, Ordering::Release);
    }
    result
}

fn audit_line(line: &[u8], expected: (u32, u16)) -> Result<bool, SafeError> {
    if !line.starts_with(b"[Parsed_ashowinfo_") {
        return Ok(false);
    }
    let text = std::str::from_utf8(line).map_err(|_| err(ErrorCode::AudioDecodeFailed))?;
    let number = |key: &str| {
        text.split_ascii_whitespace()
            .find_map(|part| part.strip_prefix(key))
            .and_then(|s| s.parse::<u32>().ok())
    };
    if number("rate:") != Some(expected.0) || number("channels:") != Some(u32::from(expected.1)) {
        return Err(err(ErrorCode::AudioCorrupt));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_ignores_relative_and_missing_directories() {
        assert_eq!(
            Backend::in_directories([PathBuf::from("."), PathBuf::from("media-tools")])
                .err()
                .unwrap()
                .code(),
            ErrorCode::AudioBackendUnavailable
        );
    }
    #[test]
    fn bounded_output_and_audit_fail_closed() {
        let fault = AtomicBool::new(false);
        assert!(read_bounded(&b"12345"[..], 4, &fault).is_err());
        assert!(fault.load(Ordering::Acquire));
        assert!(audit_frames(
            &b"[Parsed_ashowinfo_0 @ x] rate:22050 channels:1\n"[..],
            (16000, 1),
            &fault
        )
        .is_err());
        assert!(audit_frames(
            &b"[Parsed_ashowinfo_0 @ x] rate:16000 channels:1\n"[..],
            (16000, 1),
            &fault
        )
        .is_ok());
    }
    #[test]
    fn cleanup_runs_on_success_failure_and_cancellation_and_warns_on_failure() {
        for code in [
            None,
            Some(ErrorCode::AudioDecodeFailed),
            Some(ErrorCode::Cancelled),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().to_owned();
            std::fs::write(path.join("input"), b"synthetic").unwrap();
            let warning = AtomicBool::new(false);
            let result = finish_cleanup(
                code.map_or(Ok(()), |c| Err(err(c))),
                || temp.close(),
                &warning,
            );
            assert!(!path.exists());
            assert_eq!(result.err().map(|e| e.code()), code);
            assert!(!warning.load(Ordering::Acquire));
        }
        let warning = AtomicBool::new(false);
        let error = finish_cleanup(Ok(()), || Err(std::io::Error::other("injected")), &warning)
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::AudioCleanup);
        assert!(warning.load(Ordering::Acquire));
    }
    #[test]
    fn process_fixture() {
        match std::env::var("WORDWEAVE_SYNTHETIC_PROCESS").as_deref() {
            Ok("fail") => std::process::exit(17),
            Ok("large") => {
                let _ = std::io::stdout().write_all(&[0; 8192]);
            }
            Ok("stderr") => {
                let _ = std::io::stderr().write_all(&[b'x'; 8192]);
            }
            Ok("hang") => loop {
                thread::park();
            },
            _ => {}
        }
    }
    fn process_command(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "audio_backend::tests::process_fixture",
                "--nocapture",
            ])
            .env("WORDWEAVE_SYNTHETIC_PROCESS", mode)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
    #[test]
    fn injected_children_fail_timeout_cancel_and_reap_with_bounded_pipes() {
        let input = Input {
            bytes: &[],
            path: Some(PathBuf::new()),
            format: AudioFormat::ThreeGp,
        };
        let cancel = CancellationToken::new();
        let normal = || Instant::now() + Duration::from_secs(5);
        let result = supervise(
            process_command("fail"),
            &input,
            65536,
            None,
            normal(),
            &cancel,
            || {},
        );
        assert_eq!(result.unwrap_err().code(), ErrorCode::AudioDecodeFailed);
        let result = supervise(
            process_command("large"),
            &input,
            100,
            None,
            normal(),
            &cancel,
            || {},
        );
        assert_eq!(result.unwrap_err().code(), ErrorCode::AudioTooLong);
        let result = supervise(
            process_command("stderr"),
            &input,
            65536,
            Some((16000, 1)),
            normal(),
            &cancel,
            || {},
        );
        assert_eq!(result.unwrap_err().code(), ErrorCode::AudioDecodeFailed);
        let result = supervise(
            process_command("hang"),
            &input,
            65536,
            None,
            Instant::now() + Duration::from_millis(100),
            &cancel,
            || {},
        );
        assert_eq!(result.unwrap_err().code(), ErrorCode::AudioDecodeTimeout);
        let result = supervise(
            process_command("hang"),
            &input,
            65536,
            None,
            normal(),
            &cancel,
            || cancel.cancel(),
        );
        assert_eq!(result.unwrap_err().code(), ErrorCode::Cancelled);
    }
}
