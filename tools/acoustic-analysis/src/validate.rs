use crate::measure::{error, window};
use crate::*;

pub(crate) fn close(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-10
}

fn require(condition: bool) -> Result<(), AnalysisError> {
    if condition {
        Ok(())
    } else {
        Err(error(AnalysisErrorKind::InvalidOutput))
    }
}

fn energy(value: &SampleValue) -> bool {
    matches!(value, SampleValue::DigitalSilence)
        || matches!(value, SampleValue::Finite(db) if db.is_finite() && *db <= 0.0)
}

pub(crate) fn pitch_frames(
    frames: &[TimedValue],
    duration: f64,
    profile: &PitchProfile,
) -> Result<(), AnalysisError> {
    let mut previous = None;
    for frame in frames {
        require(
            frame.time_seconds.is_finite()
                && frame.time_seconds >= 0.0
                && frame.time_seconds <= duration
                && previous.is_none_or(|p| frame.time_seconds > p),
        )?;
        require(
            matches!(frame.value, SampleValue::Missing(MissingReason::Unvoiced))
                || matches!(frame.value, SampleValue::Finite(hz) if hz.is_finite() && hz >= profile.floor_hz && hz <= profile.top_hz),
        )?;
        previous = Some(frame.time_seconds);
    }
    Ok(())
}

pub(crate) fn report(report: &AnalysisReport) -> Result<(), AnalysisError> {
    require(
        report.schema_version == 1
            && report.metrics_version == "acoustic-metrics-v1"
            && !report.library_version.is_empty(),
    )?;
    report.profile.validate()?;
    require(
        (8000..=48000).contains(&report.sample_rate)
            && (1..=2).contains(&report.channels)
            && report.input_frames > 0
            && report.input_frames <= u64::from(report.sample_rate) * 60,
    )?;
    let duration = report.input_frames as f64 / f64::from(report.sample_rate);
    let m = &report.measures;
    require(close(m.duration_seconds, duration) && energy(&m.rms_dbfs))?;
    let frames = report.input_frames as usize;
    let count = (0..=6001)
        .find(|&k| window(k, report.sample_rate, frames).0 >= frames)
        .ok_or_else(|| error(AnalysisErrorKind::InvalidOutput))?;
    require(m.rms_frames.len() == count && m.pitch.frames.len() <= 6001)?;
    for (k, f) in m.rms_frames.iter().enumerate() {
        let (start, end) = window(k, report.sample_rate, frames);
        require(
            close(
                f.time_seconds,
                (start + end) as f64 / (2.0 * f64::from(report.sample_rate)),
            ) && energy(&f.value),
        )?;
    }
    // Reconstruct from non-overlapping hop intervals, so plausible totals alone
    // cannot authorize intervals that contradict the stored energy frames.
    let mut expected = Vec::new();
    expected
        .try_reserve_exact(count)
        .map_err(|_| error(AnalysisErrorKind::ResourceLimit))?;
    measure::low_intervals(&m.rms_frames, duration, &mut expected);
    require(m.low_intervals.len() == expected.len())?;
    for (actual, expected) in m.low_intervals.iter().zip(&expected) {
        require(
            close(actual.start_seconds, expected.start_seconds)
                && close(actual.end_seconds, expected.end_seconds),
        )?;
    }
    let total: f64 = expected
        .iter()
        .map(|i| i.end_seconds - i.start_seconds)
        .sum();
    require(
        close(m.low_total_seconds, total)
            && close(m.low_fraction, total / duration)
            && (0.0..=1.0).contains(&m.low_fraction),
    )?;
    validate_pitch(report)?;
    for quality in &report.quality {
        require(
            quality
                .fraction
                .is_none_or(|f| f.is_finite() && (0.0..=1.0).contains(&f)),
        )?;
    }
    Ok(())
}

fn validate_pitch(report: &AnalysisReport) -> Result<(), AnalysisError> {
    let pitch = &report.measures.pitch;
    let duration = report.measures.duration_seconds;
    pitch_frames(&pitch.frames, duration, &report.profile.pitch)?;
    if pitch.frames.is_empty() {
        let SampleValue::Missing(reason) = pitch.median_hz else {
            return Err(error(AnalysisErrorKind::InvalidOutput));
        };
        require(matches!(
            reason,
            MissingReason::TooShort
                | MissingReason::BackendUnavailable
                | MissingReason::BackendFailed
                | MissingReason::InvalidOutput
        ))?;
        require(
            pitch.p10_hz == SampleValue::Missing(reason)
                && pitch.p90_hz == SampleValue::Missing(reason)
                && pitch.valid_frame_fraction == SampleValue::Missing(reason)
                && report.status == AnalysisStatus::Partial,
        )?;
        require(
            (duration < 3.0 / report.profile.pitch.floor_hz) == (reason == MissingReason::TooShort),
        )?;
    } else {
        require(duration >= 3.0 / report.profile.pitch.floor_hz)?;
        let mut voiced = Vec::new();
        voiced
            .try_reserve_exact(pitch.frames.len())
            .map_err(|_| error(AnalysisErrorKind::ResourceLimit))?;
        voiced.extend(pitch.frames.iter().filter_map(|f| match f.value {
            SampleValue::Finite(hz) => Some(hz),
            _ => None,
        }));
        let expected = measure::summary(&pitch.frames, &mut voiced);
        for (actual, expected) in [
            (&pitch.median_hz, &expected.median_hz),
            (&pitch.p10_hz, &expected.p10_hz),
            (&pitch.p90_hz, &expected.p90_hz),
            (&pitch.valid_frame_fraction, &expected.valid_frame_fraction),
        ] {
            require(match (actual, expected) {
                (SampleValue::Finite(a), SampleValue::Finite(b)) => close(*a, *b),
                _ => actual == expected,
            })?;
        }
        require(
            report.status
                == if voiced.is_empty() {
                    AnalysisStatus::Partial
                } else {
                    AnalysisStatus::Complete
                },
        )?;
    }
    Ok(())
}
