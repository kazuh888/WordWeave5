//! Fixed local numeric protocol, independent of Windows process control.

use std::{io::Write, time::Instant};
use tokio_util::sync::CancellationToken;

use crate::{
    process::{ProcessFailure, ProcessFailureReason, ProcessOutput},
    AnalysisError, AnalysisErrorKind, AnalysisProfile, MissingReason, MonoPcm, PitchProfile,
    PitchTrack, SampleValue, TimedValue,
};

pub(crate) fn parse_pitch(
    bytes: &[u8],
    duration: f64,
    profile: &PitchProfile,
) -> Result<PitchTrack, AnalysisError> {
    let invalid = || AnalysisError {
        kind: AnalysisErrorKind::InvalidOutput,
    };
    let resource = || AnalysisError {
        kind: AnalysisErrorKind::ResourceLimit,
    };
    if bytes.len() > 1024 * 1024 {
        return Err(resource());
    }
    if profile != &AnalysisProfile::v1().pitch {
        return Err(AnalysisError {
            kind: AnalysisErrorKind::UnsupportedProfile,
        });
    }
    if !duration.is_finite() || !(0.0..=60.0).contains(&duration) {
        return Err(AnalysisError {
            kind: AnalysisErrorKind::InvalidInput,
        });
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
    if bytes
        .iter()
        .enumerate()
        .any(|(i, &b)| b == b'\r' && bytes.get(i + 1) != Some(&b'\n'))
    {
        return Err(invalid());
    }
    let mut lines = text
        .strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line));
    let mut header = lines.next().ok_or_else(invalid)?.split('\t');
    if header.next() != Some("WWPITCH") || header.next() != Some("1") {
        return Err(invalid());
    }
    let count = header.next().ok_or_else(invalid)?;
    if header.next().is_some() || count.is_empty() || !count.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let count = count.parse::<usize>().map_err(|_| resource())?;
    if count > 6001 {
        return Err(resource());
    }
    if count == 0 {
        return Err(invalid());
    }
    let mut frames = Vec::<TimedValue>::new();
    frames.try_reserve_exact(count).map_err(|_| resource())?;
    for line in lines {
        if frames.len() == count {
            return Err(invalid());
        }
        let (time, hz) = line.split_once('\t').ok_or_else(invalid)?;
        let time_seconds = decimal(time).ok_or_else(invalid)?;
        if time_seconds > duration
            || frames
                .last()
                .is_some_and(|previous| time_seconds <= previous.time_seconds)
        {
            return Err(invalid());
        }
        let value = if hz == "unvoiced" {
            SampleValue::Missing(MissingReason::Unvoiced)
        } else {
            let hz = decimal(hz).ok_or_else(invalid)?;
            if !(profile.floor_hz..=profile.top_hz).contains(&hz) {
                return Err(invalid());
            }
            SampleValue::Finite(hz)
        };
        frames.push(TimedValue {
            time_seconds,
            value,
        });
    }
    if frames.len() != count {
        return Err(invalid());
    }
    Ok(PitchTrack { frames })
}

fn decimal(value: &str) -> Option<f64> {
    let (integer, fraction) = match value.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (value, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(integer) || fraction.is_some_and(|fraction| !digits(fraction)) {
        return None;
    }
    let parsed: f64 = value.parse().ok()?;
    parsed.is_finite().then_some(parsed)
}

pub(crate) fn write_mono_wav(
    out: &mut impl Write,
    pcm: MonoPcm<'_>,
    deadline: Instant,
    cancel: &CancellationToken,
) -> Result<(), AnalysisError> {
    checkpoint(deadline, cancel)?;
    if pcm.samples.is_empty()
        || !(8_000..=48_000).contains(&pcm.sample_rate)
        || pcm.samples.len() as u64 > u64::from(pcm.sample_rate) * 60
    {
        return Err(AnalysisError {
            kind: AnalysisErrorKind::InvalidInput,
        });
    }
    // Validate the entire borrowed input before publishing any header bytes.
    for chunk in pcm.samples.chunks(4096) {
        checkpoint(deadline, cancel)?;
        if chunk
            .iter()
            .any(|sample| !sample.is_finite() || !(-1.0..=1.0).contains(sample))
        {
            return Err(AnalysisError {
                kind: AnalysisErrorKind::InvalidInput,
            });
        }
    }
    let resource = || AnalysisError {
        kind: AnalysisErrorKind::ResourceLimit,
    };
    let frames = u32::try_from(pcm.samples.len()).map_err(|_| resource())?;
    let data_bytes = frames.checked_mul(8).ok_or_else(resource)?;
    let riff_bytes = data_bytes.checked_add(50).ok_or_else(resource)?;
    let byte_rate = pcm.sample_rate.checked_mul(8).ok_or_else(resource)?;
    let mut header = [0_u8; 58];
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&riff_bytes.to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&18_u32.to_le_bytes());
    header[20..22].copy_from_slice(&3_u16.to_le_bytes());
    header[22..24].copy_from_slice(&1_u16.to_le_bytes());
    header[24..28].copy_from_slice(&pcm.sample_rate.to_le_bytes());
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&8_u16.to_le_bytes());
    header[34..36].copy_from_slice(&64_u16.to_le_bytes());
    // cbSize at [36..38] is zero for the IEEE_FLOAT fmt extension.
    header[38..42].copy_from_slice(b"fact");
    header[42..46].copy_from_slice(&4_u32.to_le_bytes());
    header[46..50].copy_from_slice(&frames.to_le_bytes());
    header[50..54].copy_from_slice(b"data");
    header[54..58].copy_from_slice(&data_bytes.to_le_bytes());
    write_checked(out, &header, deadline, cancel)?;
    let mut buffer = [0_u8; 32 * 1024];
    for samples in pcm.samples.chunks(buffer.len() / 8) {
        checkpoint(deadline, cancel)?;
        for (sample, bytes) in samples.iter().zip(buffer.chunks_exact_mut(8)) {
            bytes.copy_from_slice(&sample.to_le_bytes());
        }
        write_checked(out, &buffer[..samples.len() * 8], deadline, cancel)?;
    }
    let flushed = out.flush();
    checkpoint(deadline, cancel)?;
    flushed.map_err(|_| AnalysisError {
        kind: AnalysisErrorKind::BackendFailed,
    })
}

fn checkpoint(deadline: Instant, cancel: &CancellationToken) -> Result<(), AnalysisError> {
    if cancel.is_cancelled() {
        Err(AnalysisError {
            kind: AnalysisErrorKind::Cancelled,
        })
    } else if Instant::now() >= deadline {
        Err(AnalysisError {
            kind: AnalysisErrorKind::TimedOut,
        })
    } else {
        Ok(())
    }
}

fn write_checked(
    out: &mut impl Write,
    mut bytes: &[u8],
    deadline: Instant,
    cancel: &CancellationToken,
) -> Result<(), AnalysisError> {
    while !bytes.is_empty() {
        checkpoint(deadline, cancel)?;
        let written = out.write(bytes);
        checkpoint(deadline, cancel)?;
        match written {
            Ok(count) if count > 0 && count <= bytes.len() => bytes = &bytes[count..],
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            _ => {
                return Err(AnalysisError {
                    kind: AnalysisErrorKind::BackendFailed,
                })
            }
        }
    }
    Ok(())
}

pub(crate) fn classify_output(output: &ProcessOutput) -> Result<(), ProcessFailure> {
    const OOM: &[u8] = b"Error: Out of memory:";
    if output.memory_limit_observed || output.stderr.windows(OOM.len()).any(|window| window == OOM)
    {
        return Err(ProcessFailure {
            kind: AnalysisErrorKind::ResourceLimit,
            reason: ProcessFailureReason::MemoryLimit,
        });
    }
    // A low peak or absent notification cannot prove completion within the cap.
    if output.exit_code != 0 {
        return Err(ProcessFailure {
            kind: AnalysisErrorKind::ResourceLimit,
            reason: ProcessFailureReason::UnclassifiedAbnormalExit,
        });
    }
    if !output.stderr.is_empty() {
        return Err(ProcessFailure {
            kind: AnalysisErrorKind::BackendFailed,
            reason: ProcessFailureReason::BackendDiagnostic,
        });
    }
    Ok(())
}
