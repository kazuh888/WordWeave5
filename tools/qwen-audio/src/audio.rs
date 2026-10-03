use crate::{error::err, ErrorCode, SafeError, MAX_AUDIO_BYTES, MAX_REFERENCE_SCALARS};
use std::{io::Read, path::Path, sync::Arc};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AudioFormat {
    Wav,
    Mp3,
    Aac,
    Amr,
    ThreeGp,
}
impl AudioFormat {
    pub(crate) fn wire(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::Amr => "amr",
            Self::ThreeGp => "3gp",
        }
    }
}
#[derive(Clone)]
pub struct ReferenceText(String);
impl ReferenceText {
    pub fn new(text: String) -> Result<Self, SafeError> {
        if text.trim().is_empty() || text.chars().count() > MAX_REFERENCE_SCALARS {
            return Err(err(ErrorCode::InvalidReference));
        }
        Ok(Self(text))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct AudioInfo {
    pub byte_len: usize,
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: u64,
    pub duration_seconds: f64,
}
#[derive(Clone)]
pub struct AudioInput {
    bytes: Arc<[u8]>,
    info: AudioInfo,
    format: AudioFormat,
    playback_wav: Arc<[u8]>,
}
impl std::fmt::Debug for AudioInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioInput")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}
impl AudioInput {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, SafeError> {
        if bytes.len() > MAX_AUDIO_BYTES {
            return Err(err(ErrorCode::AudioTooLarge));
        }
        if bytes.is_empty() {
            return Err(err(ErrorCode::AudioEmpty));
        }
        let corrupt = || err(ErrorCode::AudioCorrupt);
        if bytes.len() < 12
            || &bytes[..4] != b"RIFF"
            || &bytes[8..12] != b"WAVE"
            || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize != bytes.len() - 8
        {
            return Err(corrupt());
        }
        let mut pos = 12usize;
        let mut format = None;
        let mut data = None;
        while pos < bytes.len() {
            if bytes.len() - pos < 8 {
                return Err(corrupt());
            }
            let size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            let start = pos + 8;
            let end = start.checked_add(size).ok_or_else(corrupt)?;
            let next = end.checked_add(size % 2).ok_or_else(corrupt)?;
            if next > bytes.len() {
                return Err(corrupt());
            }
            match &bytes[pos..pos + 4] {
                b"fmt " => {
                    if format.is_some() || size < 16 {
                        return Err(corrupt());
                    }
                    format = Some(&bytes[start..end]);
                }
                b"data" => {
                    if data.is_some() {
                        return Err(corrupt());
                    }
                    data = Some(size);
                }
                _ => {}
            }
            pos = next;
        }
        let fmt = format.ok_or_else(corrupt)?;
        let size = data.ok_or_else(corrupt)?;
        let u16at = |i| u16::from_le_bytes(fmt[i..i + 2].try_into().unwrap());
        let u32at = |i| u32::from_le_bytes(fmt[i..i + 4].try_into().unwrap());
        let channels = u16at(2);
        let rate = u32at(4);
        if u16at(0) != 1
            || u16at(14) != 16
            || !(1..=2).contains(&channels)
            || !(8000..=48000).contains(&rate)
        {
            return Err(err(ErrorCode::AudioUnsupported));
        }
        let align = channels * 2;
        if u16at(12) != align
            || u32at(8) != rate * u32::from(align)
            || size % usize::from(align) != 0
        {
            return Err(corrupt());
        }
        let frames = (size / usize::from(align)) as u64;
        if frames == 0 {
            return Err(err(ErrorCode::AudioEmpty));
        }
        if frames > u64::from(rate) * 60 {
            return Err(err(ErrorCode::AudioTooLong));
        }
        let info = AudioInfo {
            byte_len: bytes.len(),
            sample_rate: rate,
            channels,
            frames,
            duration_seconds: frames as f64 / rate as f64,
        };
        let bytes: Arc<[u8]> = bytes.into();
        Ok(Self {
            playback_wav: bytes.clone(),
            bytes,
            info,
            format: AudioFormat::Wav,
        })
    }
    /// Validate original bytes and decode a separate PCM preview. Run off the UI thread.
    pub fn decode(bytes: Vec<u8>, cancel: &CancellationToken) -> Result<Self, SafeError> {
        check_cancel(cancel)?;
        if bytes.len() > MAX_AUDIO_BYTES {
            return Err(err(ErrorCode::AudioTooLarge));
        }
        if bytes.is_empty() {
            return Err(err(ErrorCode::AudioEmpty));
        }
        if bytes.starts_with(b"RIFF") {
            return Self::parse(bytes);
        }
        let format = crate::audio_parser::validate(&bytes)?;
        let (pcm, rate, channels) = crate::audio_backend::decode(&bytes, format, cancel)?;
        check_cancel(cancel)?;
        let frames = (pcm.len() / (usize::from(channels) * 2)) as u64;
        let info = AudioInfo {
            byte_len: bytes.len(),
            sample_rate: rate,
            channels,
            frames,
            duration_seconds: frames as f64 / rate as f64,
        };
        let playback_wav = pcm_wav(&pcm, rate, channels).into();
        Ok(Self {
            bytes: bytes.into(),
            info,
            format,
            playback_wav,
        })
    }
    pub fn playback_wav(&self) -> &[u8] {
        &self.playback_wav
    }
    pub fn wire_format(&self) -> &'static str {
        self.format.wire()
    }
    pub fn info(&self) -> &AudioInfo {
        &self.info
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
pub(crate) fn check_cancel(cancel: &CancellationToken) -> Result<(), SafeError> {
    if cancel.is_cancelled() {
        Err(err(ErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn pcm_wav(pcm: &[u8], rate: u32, channels: u16) -> Vec<u8> {
    let mut wav = Vec::with_capacity(pcm.len() + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(pcm.len() as u32 + 36).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0");
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
    wav.extend_from_slice(&(channels * 2).to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}
pub fn read_audio(path: &Path, cancel: &CancellationToken) -> Result<AudioInput, SafeError> {
    check_cancel(cancel)?;
    let mut file = std::fs::File::open(path).map_err(|_| err(ErrorCode::AudioIo))?;
    let metadata = file.metadata().map_err(|_| err(ErrorCode::AudioIo))?;
    if !metadata.is_file() {
        return Err(err(ErrorCode::AudioIo));
    }
    if metadata.len() > MAX_AUDIO_BYTES as u64 {
        return Err(err(ErrorCode::AudioTooLarge));
    }
    let mut bytes = Vec::new();
    let mut chunk = [0; 16 * 1024];
    loop {
        check_cancel(cancel)?;
        let available = chunk.len().min(MAX_AUDIO_BYTES + 1 - bytes.len());
        let n = file
            .read(&mut chunk[..available])
            .map_err(|_| err(ErrorCode::AudioIo))?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..n]);
        if bytes.len() > MAX_AUDIO_BYTES {
            return Err(err(ErrorCode::AudioTooLarge));
        }
    }
    AudioInput::decode(bytes, cancel)
}
pub fn read_wav(path: &Path) -> Result<AudioInput, SafeError> {
    let f = std::fs::File::open(path).map_err(|_| err(ErrorCode::AudioIo))?;
    if f.metadata().map_err(|_| err(ErrorCode::AudioIo))?.len() > MAX_AUDIO_BYTES as u64 {
        return Err(err(ErrorCode::AudioTooLarge));
    }
    let mut bytes = Vec::new();
    f.take(MAX_AUDIO_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| err(ErrorCode::AudioIo))?;
    AudioInput::parse(bytes)
}
