use std::{mem::size_of, time::Instant};

use tokio_util::sync::CancellationToken;

use crate::*;

pub(crate) fn error(kind: AnalysisErrorKind) -> AnalysisError {
    AnalysisError { kind }
}

fn checkpoint(deadline: Instant, cancel: &CancellationToken) -> Result<(), AnalysisError> {
    if cancel.is_cancelled() {
        Err(error(AnalysisErrorKind::Cancelled))
    } else if Instant::now() >= deadline {
        Err(error(AnalysisErrorKind::TimedOut))
    } else {
        Ok(())
    }
}

struct Budget {
    remaining: usize,
}

impl Budget {
    fn account<T>(&mut self, count: usize) -> Result<(), AnalysisError> {
        self.remaining = count
            .checked_mul(size_of::<T>())
            .and_then(|bytes| self.remaining.checked_sub(bytes))
            .ok_or_else(|| error(AnalysisErrorKind::ResourceLimit))?;
        Ok(())
    }

    fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, AnalysisError> {
        // Check before allocation; then account actual capacity (the allocator
        // may provide more than the requested number of elements).
        let bytes = count
            .checked_mul(size_of::<T>())
            .ok_or_else(|| error(AnalysisErrorKind::ResourceLimit))?;
        if bytes > self.remaining {
            return Err(error(AnalysisErrorKind::ResourceLimit));
        }
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| error(AnalysisErrorKind::ResourceLimit))?;
        self.account::<T>(values.capacity())?;
        Ok(values)
    }
}

pub(crate) fn rms(samples: &[f64]) -> SampleValue {
    let sum: f64 = samples.iter().map(|x| x * x).sum();
    if sum == 0.0 {
        SampleValue::DigitalSilence
    } else {
        SampleValue::Finite(10.0 * (sum / samples.len() as f64).log10())
    }
}

pub(crate) fn window(k: usize, rate: u32, frames: usize) -> (usize, usize) {
    let start = (k as f64 * f64::from(rate) * 0.010).round() as usize;
    let width = (f64::from(rate) * 0.025).round() as usize;
    (start, (start + width).min(frames))
}

pub(crate) fn low_intervals(
    rms_frames: &[TimedValue],
    duration: f64,
    intervals: &mut Vec<TimeInterval>,
) {
    let mut run_start = None;
    for k in 0..=rms_frames.len() {
        let low = rms_frames.get(k).is_some_and(|f| {
            matches!(f.value, SampleValue::DigitalSilence)
                || matches!(f.value, SampleValue::Finite(db) if db <= -40.0)
        });
        if low {
            run_start.get_or_insert(k);
        } else if let Some(start) = run_start.take() {
            let start_seconds = start as f64 * 0.010;
            let end_seconds = (k as f64 * 0.010).min(duration);
            if end_seconds - start_seconds >= 0.200 - 1e-12 {
                intervals.push(TimeInterval {
                    start_seconds,
                    end_seconds,
                });
            }
        }
    }
}

pub(crate) fn analyze(
    request: AnalysisRequest<'_>,
    pitch: &dyn PitchBackend,
    cancel: &CancellationToken,
) -> Result<AnalysisReport, AnalysisError> {
    let deadline = Instant::now()
        .checked_add(request.limits.wall_time.min(AnalysisLimits::v1().wall_time))
        .ok_or_else(|| error(AnalysisErrorKind::ResourceLimit))?;
    checkpoint(deadline, cancel)?;
    request.profile.validate()?;
    let pcm = request.pcm;
    let duration = pcm.frames() as f64 / f64::from(pcm.sample_rate());
    let frame_limit = request.limits.max_frames.min(6001);
    let count = (0..=6001)
        .find(|&k| window(k, pcm.sample_rate(), pcm.frames()).0 >= pcm.frames())
        .ok_or_else(|| error(AnalysisErrorKind::ResourceLimit))?;
    if count > frame_limit {
        return Err(error(AnalysisErrorKind::ResourceLimit));
    }
    let mut budget = Budget {
        remaining: request.limits.max_buffer_bytes.min(64 * 1024 * 1024),
    };
    let mut mono = budget.vector::<f64>(pcm.frames())?;
    let mut clips = 0usize;
    for (k, source) in pcm
        .samples()
        .chunks_exact(usize::from(pcm.channels()))
        .enumerate()
    {
        if k % 4096 == 0 {
            checkpoint(deadline, cancel)?;
        }
        clips += source
            .iter()
            .filter(|&&x| x == i16::MIN || x == i16::MAX)
            .count();
        mono.push(
            source.iter().map(|&x| f64::from(x) / 32768.0).sum::<f64>() / f64::from(pcm.channels()),
        );
    }
    let rms_dbfs = rms(&mono);
    checkpoint(deadline, cancel)?;
    let mut rms_frames = budget.vector::<TimedValue>(count)?;
    for k in 0..count {
        checkpoint(deadline, cancel)?;
        let (start, end) = window(k, pcm.sample_rate(), pcm.frames());
        rms_frames.push(TimedValue {
            time_seconds: (start + end) as f64 / (2.0 * f64::from(pcm.sample_rate())),
            value: rms(&mono[start..end]),
        });
    }
    let mut intervals = budget.vector::<TimeInterval>(count)?;
    low_intervals(&rms_frames, duration, &mut intervals);
    let low_total_seconds: f64 = intervals
        .iter()
        .map(|i| i.end_seconds - i.start_seconds)
        .sum();
    let mut quality = budget.vector::<Quality>(5)?;
    if pcm.channels() == 2 {
        quality.push(Quality {
            kind: QualityKind::StereoMixed,
            fraction: None,
        });
    }
    if clips > 0 {
        quality.push(Quality {
            kind: QualityKind::ClipCandidate,
            fraction: Some(clips as f64 / pcm.samples().len() as f64),
        });
    }
    let (pitch_measure, pitch_engine) = extract_pitch(
        &mono,
        &request,
        pitch,
        deadline,
        cancel,
        &mut budget,
        &mut quality,
    )?;
    // Include report strings and validation scratch space in the common budget.
    budget.account::<u8>(
        request.profile.profile_id.len()
            + "acoustic-metrics-v1".len()
            + env!("CARGO_PKG_VERSION").len(),
    )?;
    budget.account::<TimeInterval>(count)?;
    budget.account::<f64>(pitch_measure.frames.len())?;
    let status = if matches!(pitch_measure.median_hz, SampleValue::Finite(_)) {
        AnalysisStatus::Complete
    } else {
        AnalysisStatus::Partial
    };
    let report = AnalysisReport {
        schema_version: 1,
        metrics_version: "acoustic-metrics-v1".into(),
        library_version: env!("CARGO_PKG_VERSION").into(),
        profile: request.profile.clone(),
        sample_rate: pcm.sample_rate(),
        channels: pcm.channels(),
        input_frames: pcm.frames() as u64,
        pitch_engine,
        status,
        quality,
        measures: Measures {
            duration_seconds: duration,
            rms_dbfs,
            rms_frames,
            low_intervals: intervals,
            low_total_seconds,
            low_fraction: low_total_seconds / duration,
            pitch: pitch_measure,
        },
    };
    report.validate()?;
    checkpoint(deadline, cancel)?;
    Ok(report)
}

fn extract_pitch(
    mono: &[f64],
    request: &AnalysisRequest<'_>,
    pitch: &dyn PitchBackend,
    deadline: Instant,
    cancel: &CancellationToken,
    budget: &mut Budget,
    quality: &mut Vec<Quality>,
) -> Result<(PitchMeasure, Option<EngineIdentity>), AnalysisError> {
    let rate = request.pcm.sample_rate();
    let duration = mono.len() as f64 / f64::from(rate);
    let profile = request.profile;
    if duration < 3.0 / profile.pitch.floor_hz {
        quality.push(Quality {
            kind: QualityKind::ShortInput,
            fraction: None,
        });
        return Ok((missing_pitch(MissingReason::TooShort), None));
    }
    checkpoint(deadline, cancel)?;
    let engine = pitch.identity();
    if let Some(engine) = &engine {
        for s in [
            Some(&engine.name),
            Some(&engine.version),
            engine.binary_sha256.as_ref(),
            engine.script_sha256.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            budget.account::<u8>(s.capacity())?;
        }
    }
    let extracted = pitch.extract(
        MonoPcm {
            samples: mono,
            sample_rate: rate,
        },
        &profile.pitch,
        deadline,
        cancel,
    );
    checkpoint(deadline, cancel)?;
    let track = match extracted {
        Ok(track) => track,
        Err(e) => {
            let reason = match e.kind {
                AnalysisErrorKind::BackendUnavailable => MissingReason::BackendUnavailable,
                AnalysisErrorKind::BackendFailed => MissingReason::BackendFailed,
                AnalysisErrorKind::InvalidOutput => MissingReason::InvalidOutput,
                _ => return Err(e),
            };
            quality.push(Quality {
                kind: QualityKind::BackendWarning,
                fraction: None,
            });
            return Ok((missing_pitch(reason), engine));
        }
    };
    if track.frames.len() > request.limits.max_frames.min(6001) {
        return Err(error(AnalysisErrorKind::ResourceLimit));
    }
    budget.account::<TimedValue>(track.frames.capacity())?;
    if track.frames.is_empty()
        || validate::pitch_frames(&track.frames, duration, &profile.pitch).is_err()
    {
        quality.push(Quality {
            kind: QualityKind::BackendWarning,
            fraction: None,
        });
        return Ok((missing_pitch(MissingReason::InvalidOutput), engine));
    }
    let mut voiced = budget.vector::<f64>(track.frames.len())?;
    for f in &track.frames {
        if let SampleValue::Finite(hz) = f.value {
            voiced.push(hz);
        }
    }
    let mut measure = summary(&track.frames, &mut voiced);
    if voiced.is_empty() {
        quality.push(Quality {
            kind: QualityKind::NoVoicedFrames,
            fraction: None,
        });
    }
    measure.frames = track.frames;
    Ok((measure, engine))
}

pub(crate) fn missing_pitch(reason: MissingReason) -> PitchMeasure {
    PitchMeasure {
        frames: Vec::new(),
        median_hz: SampleValue::Missing(reason),
        p10_hz: SampleValue::Missing(reason),
        p90_hz: SampleValue::Missing(reason),
        valid_frame_fraction: SampleValue::Missing(reason),
    }
}

pub(crate) fn summary(frames: &[TimedValue], voiced: &mut [f64]) -> PitchMeasure {
    voiced.sort_unstable_by(f64::total_cmp);
    let quantile = |p: f64| {
        if voiced.is_empty() {
            return SampleValue::Missing(MissingReason::Unvoiced);
        }
        let h = (voiced.len() - 1) as f64 * p;
        let left = h.floor() as usize;
        let right = h.ceil() as usize;
        SampleValue::Finite(voiced[left] + (voiced[right] - voiced[left]) * (h - left as f64))
    };
    PitchMeasure {
        frames: Vec::new(),
        median_hz: quantile(0.5),
        p10_hz: quantile(0.1),
        p90_hz: quantile(0.9),
        valid_frame_fraction: SampleValue::Finite(voiced.len() as f64 / frames.len() as f64),
    }
}
