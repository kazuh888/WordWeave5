use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    io::Cursor,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

mod speech;
pub use speech::{PlaybackSnapshot, Speaker};

pub struct Recorder {
    stream: cpal::Stream,
    samples: Arc<Mutex<CaptureBuffer>>,
    error: Arc<Mutex<Option<String>>>,
    sample_rate: u32,
}

/// The captured take and a device warning, returned together after callbacks stop.
/// A warning never discards a valid, nonempty partial recording.
pub struct CapturedRecording {
    pub wav: Vec<u8>,
    pub warning: Option<String>,
}

#[derive(Default)]
struct CaptureBuffer {
    samples: Vec<i16>,
    paused: bool,
}

impl CaptureBuffer {
    fn append<T: Copy>(
        &mut self,
        input: &[T],
        channels: usize,
        limit: usize,
        convert: impl Fn(T) -> f32,
    ) {
        if self.paused || channels == 0 {
            return;
        }
        for frame in input.chunks_exact(channels) {
            if self.samples.len() >= limit {
                break;
            }
            let v = frame.iter().map(|s| convert(*s)).sum::<f32>() / frame.len() as f32;
            self.samples
                .push((v.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16);
        }
    }

    fn elapsed(&self, sample_rate: u32) -> Duration {
        if sample_rate == 0 {
            return Duration::ZERO;
        }
        Duration::from_secs_f64(self.samples.len() as f64 / sample_rate as f64)
    }
}
impl Recorder {
    pub fn start() -> Result<Self, String> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or("マイクがありません。Windowsの既定の入力デバイスを確認してください。")?;
        let supported = device.default_input_config().map_err(|e| e.to_string())?;
        let config: cpal::StreamConfig = supported.clone().into();
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;
        if channels == 0 || sample_rate == 0 || sample_rate > 192000 {
            return Err("このマイク形式には対応していません。".into());
        }
        let data = Arc::new(Mutex::new(CaptureBuffer::default()));
        let error = Arc::new(Mutex::new(None));
        let cap = sample_rate as usize * 30;
        macro_rules! stream {
            ($sample:ty,$convert:expr) => {{
                let d = data.clone();
                let err = error.clone();
                device.build_input_stream(
                    &config,
                    move |input: &[$sample], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buffer) = d.lock() {
                            buffer.append(input, channels, cap, $convert);
                        }
                    },
                    move |e| {
                        if let Ok(mut x) = err.lock() {
                            *x = Some(e.to_string());
                        }
                    },
                    None,
                )
            }};
        }
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => stream!(f32, |x: f32| x),
            cpal::SampleFormat::I16 => stream!(i16, |x: i16| x as f32 / 32768.0),
            cpal::SampleFormat::U16 => stream!(u16, |x: u16| (x as f32 - 32768.0) / 32768.0),
            _ => return Err("このマイクのサンプル形式には対応していません。".into()),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| {
            format!("録音を開始できません。マイクのプライバシー設定を確認してください: {e}")
        })?;
        Ok(Self {
            stream,
            samples: data,
            error,
            sample_rate,
        })
    }
    /// Duration actually captured; paused wall-clock time adds no silence.
    pub fn elapsed(&self) -> Duration {
        self.samples
            .lock()
            .map(|s| s.elapsed(self.sample_rate))
            .unwrap_or_default()
    }
    pub fn is_paused(&self) -> bool {
        self.samples.lock().map(|s| s.paused).unwrap_or(true)
    }
    pub fn pause(&mut self) -> Result<(), String> {
        if self.is_paused() {
            return Ok(());
        }
        // Gate callbacks before requesting the device pause, including callbacks
        // already queued by WASAPI. Both the gate and samples share this lock.
        self.samples
            .lock()
            .map_err(|_| "録音状態を読み取れません。")?
            .paused = true;
        if let Err(error) = self.stream.pause() {
            self.samples
                .lock()
                .map_err(|_| "録音状態を読み取れません。")?
                .paused = false;
            return Err(format!("録音を一時停止できません: {error}"));
        }
        Ok(())
    }
    pub fn resume(&mut self) -> Result<(), String> {
        if !self.is_paused() {
            return Ok(());
        }
        self.stream
            .play()
            .map_err(|e| format!("録音を再開できません: {e}"))?;
        self.samples
            .lock()
            .map_err(|_| "録音状態を読み取れません。")?
            .paused = false;
        Ok(())
    }
    /// Releases the input device and discards only this unsaved in-memory take.
    pub fn cancel(self) {
        drop(self);
    }

    pub fn error(&self) -> Option<String> {
        match self.error.lock() {
            Ok(error) => error.clone(),
            Err(_) => Some("録音状態を読み取れません。".into()),
        }
    }

    pub fn level(&self) -> f32 {
        self.samples
            .lock()
            .ok()
            .map(|s| {
                if s.paused {
                    return 0.0;
                }
                s.samples
                    .iter()
                    .rev()
                    .take(1000)
                    .map(|x| (*x as f32 / 32768.0).abs())
                    .fold(0.0, f32::max)
            })
            .unwrap_or(0.0)
    }
    pub fn finish(self) -> Result<Vec<u8>, String> {
        self.finish_with_warning().map(|captured| captured.wav)
    }

    /// Stop callbacks first so a last-moment device failure cannot lose its warning.
    pub fn finish_with_warning(self) -> Result<CapturedRecording, String> {
        drop(self.stream);
        let warning = self
            .error
            .lock()
            .map_err(|_| "録音状態を読み取れません。")?
            .clone();
        let samples = self
            .samples
            .lock()
            .map_err(|_| "録音データを読み取れません。")?;
        encode_recording(&samples.samples, self.sample_rate, warning)
    }
}

fn encode_recording(
    samples: &[i16],
    sample_rate: u32,
    warning: Option<String>,
) -> Result<CapturedRecording, String> {
    if sample_rate == 0 {
        return Err("録音のサンプルレートが不正です。".into());
    }
    if samples.is_empty() {
        return Err(warning.unwrap_or_else(|| "録音が短すぎます。".into()));
    }
    // Normal takes retain the existing minimum duration. Recover every available
    // sample after a device error, even when the failure happened immediately.
    if warning.is_none() && samples.len() < sample_rate as usize / 4 {
        return Err("録音が短すぎます。".into());
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;
        for &sample in samples {
            writer.write_sample(sample).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
    }
    Ok(CapturedRecording {
        wav: cursor.into_inner(),
        warning,
    })
}

fn validate_playback_rate(rate: f64) -> Result<f64, String> {
    if !rate.is_finite() || !(0.5..=4.0).contains(&rate) {
        return Err("読み上げ速度は0.5〜4.0倍で指定してください。".into());
    }
    Ok(rate)
}

fn bounded_position(position: f64, duration: f64) -> Result<f64, String> {
    if !position.is_finite() || !duration.is_finite() || duration <= 0.0 {
        return Err("読み上げの再生位置をまだ変更できません。".into());
    }
    Ok(position.clamp(0.0, duration))
}

#[link(name = "winmm")]
extern "system" {
    fn PlaySoundW(sound: *const u16, module: isize, flags: u32) -> i32;
}

pub fn playback(wav: Vec<u8>, dir: PathBuf) -> Result<(), String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = dir.join(format!("playback-{stamp}.wav"));
    std::fs::write(&path, wav).map_err(|e| e.to_string())?;
    let name: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SND_FILENAME | SND_NODEFAULT; synchronous on a dedicated worker thread.
    // `name` remains alive throughout the native call.
    let ok = unsafe { PlaySoundW(name.as_ptr(), 0, 0x00020002) };
    let _ = std::fs::remove_file(path);
    if ok == 0 {
        Err("録音を再生できませんでした。".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod controls_tests {
    use super::*;

    #[test]
    fn device_failure_preserves_all_captured_samples_and_returns_a_warning() {
        let captured =
            encode_recording(&[1, -2, 3], 48000, Some("microphone disconnected".into())).unwrap();
        assert_eq!(captured.warning.as_deref(), Some("microphone disconnected"));
        let mut reader = hound::WavReader::new(Cursor::new(captured.wav)).unwrap();
        assert_eq!(
            reader
                .samples::<i16>()
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            vec![1, -2, 3]
        );
    }

    #[test]
    fn no_audio_is_not_reported_as_a_successful_recovered_recording() {
        assert!(encode_recording(&[], 48000, Some("microphone disconnected".into())).is_err());
        assert!(encode_recording(&[1, 2, 3], 48000, None).is_err());
    }

    #[test]
    fn playback_speed_rejects_non_finite_and_out_of_range_values() {
        for rate in [f64::NAN, f64::INFINITY, -1.0, 0.49, 4.01] {
            assert!(validate_playback_rate(rate).is_err());
        }
        for rate in [0.5, 0.8, 1.0, 4.0] {
            assert_eq!(validate_playback_rate(rate).unwrap(), rate);
        }
    }

    #[test]
    fn seeking_clamps_to_the_audio_bounds() {
        assert_eq!(bounded_position(-5.0, 12.0).unwrap(), 0.0);
        assert_eq!(bounded_position(17.0, 12.0).unwrap(), 12.0);
        assert_eq!(bounded_position(6.25, 12.0).unwrap(), 6.25);
        assert!(bounded_position(f64::NAN, 12.0).is_err());
        assert!(bounded_position(1.0, 0.0).is_err());
    }

    #[test]
    fn paused_recording_has_no_samples_or_elapsed_gap() {
        let mut capture = CaptureBuffer::default();
        capture.append(&[100i16, 200], 1, 8, |x| x as f32 / 32767.0);
        let before_pause = capture.elapsed(4);
        capture.paused = true;
        capture.append(&[300i16, 400, 500, 600], 1, 8, |x| x as f32 / 32767.0);
        assert_eq!(capture.elapsed(4), before_pause);
        capture.paused = false;
        capture.append(&[700i16, 800], 1, 8, |x| x as f32 / 32767.0);
        assert_eq!(capture.samples, vec![100, 200, 700, 800]);
        assert_eq!(capture.elapsed(4), std::time::Duration::from_secs(1));
    }

    #[test]
    fn recording_cap_counts_frames_and_survives_pause_resume() {
        let mut capture = CaptureBuffer::default();
        capture.append(&[0.25f32, 0.75, 0.5, 0.5], 2, 3, |x| x);
        assert_eq!(capture.samples.len(), 2);
        capture.paused = true;
        capture.append(&[1.0f32; 20], 2, 3, |x| x);
        capture.paused = false;
        capture.append(&[1.0f32; 20], 2, 3, |x| x);
        assert_eq!(capture.samples.len(), 3);
        assert_eq!(capture.elapsed(1), std::time::Duration::from_secs(3));
    }
}
