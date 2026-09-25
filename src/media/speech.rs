//! Local Windows speech synthesis and independently controllable playback.
//! Synthesis is polled, not blocked on the UI thread. No audio is sent to a server.
use std::sync::{Arc, Mutex};
use windows::{
    core::{Interface, HSTRING},
    Foundation::{AsyncStatus, IAsyncOperation, TimeSpan, TypedEventHandler},
    Media::{
        Core::MediaSource,
        Playback::{
            MediaPlaybackState, MediaPlayer, MediaPlayerAudioCategory, MediaPlayerFailedEventArgs,
        },
        SpeechSynthesis::{SpeechSynthesisStream, SpeechSynthesizer},
    },
    Storage::Streams::{DataWriter, IRandomAccessStream, InMemoryRandomAccessStream},
};

use super::{bounded_position, validate_playback_rate};

#[derive(Clone, Copy, Debug)]
pub struct PlaybackSnapshot {
    pub loaded: bool,
    pub playing: bool,
    pub loading: bool,
    pub can_seek: bool,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub rate: f64,
    pub volume: f64,
    pub repeat: bool,
}
impl Default for PlaybackSnapshot {
    fn default() -> Self {
        Self {
            loaded: false,
            playing: false,
            loading: false,
            can_seek: false,
            position_seconds: 0.0,
            duration_seconds: 0.0,
            rate: 1.0,
            volume: 0.5,
            repeat: false,
        }
    }
}

fn native_error(error: windows::core::Error) -> String {
    // Native messages may contain OS/user data. Keep a diagnosable code only.
    format!(
        "Windows音声処理に失敗した (0x{:08X})。音声機能と出力デバイスを確認してください。",
        error.code().0 as u32
    )
}

fn validate_wav(wav: &[u8]) -> Result<(), String> {
    const INVALID: &str = "WAV音声データが不正です。";
    if wav.len() < 12
        || wav.len() > u32::MAX as usize
        || &wav[..4] != b"RIFF"
        || &wav[8..12] != b"WAVE"
    {
        return Err(INVALID.into());
    }
    let riff_size = u32::from_le_bytes(wav[4..8].try_into().unwrap()) as usize;
    if riff_size.checked_add(8) != Some(wav.len()) {
        return Err(INVALID.into());
    }

    let mut offset = 12;
    let mut has_format = false;
    let mut has_audio = false;
    while offset < wav.len() {
        if wav.len() - offset < 8 {
            return Err(INVALID.into());
        }
        let chunk_size =
            u32::from_le_bytes(wav[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let next = offset
            .checked_add(8)
            .and_then(|start| start.checked_add(chunk_size))
            .and_then(|end| end.checked_add(chunk_size % 2))
            .ok_or(INVALID)?;
        if next > wav.len() {
            return Err(INVALID.into());
        }
        match &wav[offset..offset + 4] {
            b"fmt " => has_format = chunk_size >= 16,
            b"data" => has_audio = chunk_size > 0,
            _ => {}
        }
        offset = next;
    }
    if !has_format || !has_audio {
        return Err(INVALID.into());
    }
    Ok(())
}

fn wav_stream(wav: &[u8]) -> Result<IRandomAccessStream, String> {
    let stream = InMemoryRandomAccessStream::new().map_err(native_error)?;
    let writer = DataWriter::CreateDataWriter(&stream).map_err(native_error)?;
    writer.WriteBytes(wav).map_err(native_error)?;
    if writer
        .StoreAsync()
        .map_err(native_error)?
        .get()
        .map_err(native_error)? as usize
        != wav.len()
    {
        return Err("WAV音声データをメモリに読み込めませんでした。".into());
    }
    writer.DetachStream().map_err(native_error)?;
    stream.Seek(0).map_err(native_error)?;
    stream.cast().map_err(native_error)
}

struct PendingSpeech {
    _synthesizer: SpeechSynthesizer,
    operation: IAsyncOperation<SpeechSynthesisStream>,
    rate: f64,
    play_when_ready: bool,
}

struct SpeechPlayback {
    player: MediaPlayer,
    // Retain source and random-access stream through pause, seek, and replay.
    _source: MediaSource,
    _stream: IRandomAccessStream,
    failure: Arc<Mutex<Option<String>>>,
}
impl SpeechPlayback {
    fn new(
        stream: IRandomAccessStream,
        content_type: &HSTRING,
        rate: f64,
        play: bool,
        volume: f64,
        repeat: bool,
    ) -> Result<Self, String> {
        let player = MediaPlayer::new().map_err(native_error)?;
        player.SetAutoPlay(false).map_err(native_error)?;
        player.SetVolume(volume).map_err(native_error)?;
        player.SetIsLoopingEnabled(repeat).map_err(native_error)?;
        player
            .SetAudioCategory(MediaPlayerAudioCategory::Speech)
            .map_err(native_error)?;
        // Do not intercept global media keys or displace another app's player.
        player
            .CommandManager()
            .map_err(native_error)?
            .SetIsEnabled(false)
            .map_err(native_error)?;
        let failure = Arc::new(Mutex::new(None));
        let failed = failure.clone();
        player
            .MediaFailed(&TypedEventHandler::new(
                move |_: &Option<MediaPlayer>, args: &Option<MediaPlayerFailedEventArgs>| {
                    let code = args.as_ref().and_then(|a| a.ExtendedErrorCode().ok());
                    if let Ok(mut error) = failed.lock() {
                        *error = Some(match code {
                            Some(code) => format!(
                                "音声の再生に失敗した (0x{:08X})。出力デバイスを確認してください。",
                                code.0 as u32
                            ),
                            None => "音声の再生に失敗した。出力デバイスを確認してください。".into(),
                        });
                    }
                    Ok(())
                },
            ))
            .map_err(native_error)?;
        let source = MediaSource::CreateFromStream(&stream, content_type).map_err(native_error)?;
        let playback = Self {
            player,
            _source: source,
            _stream: stream,
            failure,
        };
        playback
            .player
            .SetSource(&playback._source)
            .map_err(native_error)?;
        playback
            .player
            .PlaybackSession()
            .map_err(native_error)?
            .SetPlaybackRate(rate)
            .map_err(native_error)?;
        if play {
            playback.player.Play().map_err(native_error)?;
        }
        Ok(playback)
    }

    fn check_error(&self) -> Result<(), String> {
        if let Some(error) = self
            .failure
            .lock()
            .map_err(|_| "音声の再生状態を読み取れません。")?
            .as_ref()
        {
            return Err(error.clone());
        }
        Ok(())
    }
}
impl Drop for SpeechPlayback {
    fn drop(&mut self) {
        let _ = self.player.Close();
    }
}

pub struct Speaker {
    volume: f64,
    repeat: bool,
    engine: Option<SpeechPlayback>,
    pending: Option<PendingSpeech>,
    pub voices: Vec<(String, String)>,
}
impl Speaker {
    pub fn new() -> Self {
        let voices = SpeechSynthesizer::AllVoices()
            .map(|voices| {
                voices
                    .into_iter()
                    .filter_map(|v| {
                        Some((
                            v.Id().ok()?.to_string(),
                            format!("{} ({})", v.DisplayName().ok()?, v.Language().ok()?),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            volume: 0.5,
            repeat: false,
            engine: None,
            pending: None,
            voices,
        }
    }

    /// Compatibility for older call sites; new controls use an explicit rate.
    pub fn say(&mut self, text: &str, voice_id: &str, slow: bool) -> Result<(), String> {
        self.say_at_rate(text, voice_id, if slow { 0.8 } else { 1.0 })
    }

    pub fn say_at_rate(&mut self, text: &str, voice_id: &str, rate: f64) -> Result<(), String> {
        let rate = validate_playback_rate(rate)?;
        if text.trim().is_empty() {
            return Err("読み上げる文章がありません。".into());
        }
        let voices = SpeechSynthesizer::AllVoices().map_err(native_error)?;
        let voice = voices
            .into_iter()
            .find(|v| {
                if voice_id.is_empty() {
                    v.Language()
                        .map(|l| l.to_string().to_ascii_lowercase().starts_with("en"))
                        .unwrap_or(false)
                } else {
                    v.Id().map(|id| id.to_string() == voice_id).unwrap_or(false)
                }
            })
            .ok_or(
                "英語の音声が見つかりません。Windowsに英語の音声を追加し、設定を確認してください。",
            )?;
        let synthesizer = SpeechSynthesizer::new().map_err(native_error)?;
        synthesizer.SetVoice(&voice).map_err(native_error)?;
        // Synthesize at natural speed; the player owns all speed changes.
        synthesizer
            .Options()
            .map_err(native_error)?
            .SetSpeakingRate(1.0)
            .map_err(native_error)?;
        let operation = synthesizer
            .SynthesizeTextToStreamAsync(&HSTRING::from(text))
            .map_err(native_error)?;
        self.stop()?;
        self.engine = None;
        self.pending = Some(PendingSpeech {
            _synthesizer: synthesizer,
            operation,
            rate,
            play_when_ready: true,
        });
        Ok(())
    }

    /// Play a saved WAV through the same controllable player used for speech synthesis.
    pub fn play_wav(&mut self, wav: &[u8], rate: f64) -> Result<(), String> {
        let rate = validate_playback_rate(rate)?;
        validate_wav(wav)?;
        let stream = wav_stream(wav)?;
        let playback = SpeechPlayback::new(
            stream,
            &HSTRING::from("audio/wav"),
            rate,
            false,
            self.volume,
            self.repeat,
        )?;
        self.stop()?;
        self.pending = None;
        self.engine = Some(playback);
        self.engine
            .as_ref()
            .unwrap()
            .player
            .Play()
            .map_err(native_error)
    }

    fn poll(&mut self) -> Result<(), String> {
        let Some(pending) = self.pending.take() else {
            return Ok(());
        };
        let status = pending.operation.Status().map_err(native_error)?;
        if status == AsyncStatus::Started {
            self.pending = Some(pending);
            return Ok(());
        }
        // GetResults is called only after completion and propagates errors.
        let stream = pending.operation.GetResults().map_err(native_error)?;
        let content_type = stream.ContentType().map_err(native_error)?;
        self.engine = Some(SpeechPlayback::new(
            stream.cast().map_err(native_error)?,
            &content_type,
            pending.rate,
            pending.play_when_ready,
            self.volume,
            self.repeat,
        )?);
        Ok(())
    }

    /// Poll globally, including while the chat tab is hidden.
    pub fn snapshot(&mut self) -> Result<PlaybackSnapshot, String> {
        self.poll()?;
        if let Some(pending) = &self.pending {
            return Ok(PlaybackSnapshot {
                loaded: true,
                loading: true,
                rate: pending.rate,
                volume: self.volume,
                repeat: self.repeat,
                ..Default::default()
            });
        }
        let Some(engine) = &self.engine else {
            return Ok(PlaybackSnapshot {
                volume: self.volume,
                repeat: self.repeat,
                ..Default::default()
            });
        };
        engine.check_error()?;
        let session = engine.player.PlaybackSession().map_err(native_error)?;
        let state = session.PlaybackState().map_err(native_error)?;
        Ok(PlaybackSnapshot {
            loaded: true,
            playing: state == MediaPlaybackState::Playing,
            loading: state == MediaPlaybackState::Opening || state == MediaPlaybackState::Buffering,
            can_seek: session.CanSeek().map_err(native_error)?,
            position_seconds: session.Position().map_err(native_error)?.Duration.max(0) as f64
                / 10_000_000.0,
            duration_seconds: session
                .NaturalDuration()
                .map_err(native_error)?
                .Duration
                .max(0) as f64
                / 10_000_000.0,
            rate: session.PlaybackRate().map_err(native_error)?,
            volume: engine.player.Volume().map_err(native_error)?,
            repeat: engine.player.IsLoopingEnabled().map_err(native_error)?,
        })
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if let Some(pending) = &mut self.pending {
            pending.play_when_ready = false;
            return Ok(());
        }
        let engine = self.engine.as_ref().ok_or("再生する音声がありません。")?;
        engine.player.Pause().map_err(native_error)
    }
    pub fn resume(&mut self) -> Result<(), String> {
        if let Some(pending) = &mut self.pending {
            pending.play_when_ready = true;
            return Ok(());
        }
        let engine = self.engine.as_ref().ok_or("再生する音声がありません。")?;
        engine.check_error()?;
        let session = engine.player.PlaybackSession().map_err(native_error)?;
        let duration = session.NaturalDuration().map_err(native_error)?.Duration;
        if duration > 0 && session.Position().map_err(native_error)?.Duration >= duration {
            session
                .SetPosition(TimeSpan { Duration: 0 })
                .map_err(native_error)?;
        }
        engine.player.Play().map_err(native_error)
    }
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(pending) = self.pending.take() {
            pending.operation.Cancel().map_err(native_error)?;
        }
        if let Some(engine) = &self.engine {
            engine.player.Pause().map_err(native_error)?;
            let session = engine.player.PlaybackSession().map_err(native_error)?;
            if session.Position().map_err(native_error)?.Duration > 0 {
                session
                    .SetPosition(TimeSpan { Duration: 0 })
                    .map_err(native_error)?;
            }
        }
        Ok(())
    }
    pub fn set_rate(&mut self, rate: f64) -> Result<(), String> {
        let rate = validate_playback_rate(rate)?;
        if let Some(pending) = &mut self.pending {
            pending.rate = rate;
            return Ok(());
        }
        if let Some(engine) = &self.engine {
            engine.check_error()?;
            engine
                .player
                .PlaybackSession()
                .map_err(native_error)?
                .SetPlaybackRate(rate)
                .map_err(native_error)?;
        }
        Ok(())
    }
    pub fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
            return Err("音量は0〜100%で指定してください。".into());
        }
        if let Some(engine) = &self.engine {
            engine.player.SetVolume(volume).map_err(native_error)?;
        }
        self.volume = volume;
        Ok(())
    }

    pub fn set_repeat(&mut self, repeat: bool) -> Result<(), String> {
        if let Some(engine) = &self.engine {
            engine
                .player
                .SetIsLoopingEnabled(repeat)
                .map_err(native_error)?;
        }
        self.repeat = repeat;
        Ok(())
    }
    pub fn seek(&mut self, seconds: f64) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("音声を準備中のため、再生位置はまだ変更できません。".into());
        }
        let engine = self.engine.as_ref().ok_or("再生する音声がありません。")?;
        engine.check_error()?;
        let session = engine.player.PlaybackSession().map_err(native_error)?;
        if !session.CanSeek().map_err(native_error)? {
            return Err("この音声の再生位置はまだ変更できません。".into());
        }
        let duration =
            session.NaturalDuration().map_err(native_error)?.Duration as f64 / 10_000_000.0;
        let position = bounded_position(seconds, duration)?;
        session
            .SetPosition(TimeSpan {
                Duration: (position * 10_000_000.0).round() as i64,
            })
            .map_err(native_error)
    }
    pub fn seek_relative(&mut self, seconds: f64) -> Result<(), String> {
        let snapshot = self.snapshot()?;
        self.seek(snapshot.position_seconds + seconds)
    }
}
impl Drop for Speaker {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // One mono 16-bit PCM sample at 8 kHz; no Windows audio device is needed to parse it.
    const PCM_WAV: &[u8] = b"RIFF\x26\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x40\x1f\0\0\x80\x3e\0\0\x02\0\x10\0data\x02\0\0\0\0\0";

    #[test]
    fn wav_validation_checks_signature_declared_length_and_chunks() {
        validate_wav(PCM_WAV).unwrap();
        let mut bad_signature = PCM_WAV.to_vec();
        bad_signature[..4].copy_from_slice(b"RIFX");
        let mut bad_type = PCM_WAV.to_vec();
        bad_type[8..12].copy_from_slice(b"AVI ");
        let mut bad_chunk_length = PCM_WAV.to_vec();
        bad_chunk_length[40..44].copy_from_slice(&100u32.to_le_bytes());
        for invalid in [
            &b""[..],
            &b"RIFF\x04\0\0\0WAVE"[..],
            &PCM_WAV[..PCM_WAV.len() - 1],
            bad_signature.as_slice(),
            bad_type.as_slice(),
            bad_chunk_length.as_slice(),
        ] {
            assert!(validate_wav(invalid).is_err());
        }
    }

    #[test]
    fn invalid_wav_does_not_change_unloaded_player_state() {
        let mut speaker = Speaker {
            volume: 0.5,
            repeat: false,
            engine: None,
            pending: None,
            voices: vec![],
        };
        speaker.set_volume(0.7).unwrap();
        speaker.set_repeat(true).unwrap();
        assert!(speaker.play_wav(b"invalid", 1.0).is_err());
        assert!(speaker.play_wav(PCM_WAV, f64::NAN).is_err());
        let state = speaker.snapshot().unwrap();
        assert!(!state.loaded);
        assert!(!state.loading);
        assert_eq!(state.volume, 0.7);
        assert!(state.repeat);
    }

    #[test]
    fn unloaded_player_is_safe_without_a_voice_or_audio_device() {
        let mut speaker = Speaker {
            volume: 0.5,
            repeat: false,
            engine: None,
            pending: None,
            voices: vec![],
        };
        assert!(!speaker.snapshot().unwrap().loaded);
        for invalid in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
            assert!(speaker.set_volume(invalid).is_err());
        }
        speaker.set_volume(0.7).unwrap();
        speaker.set_repeat(true).unwrap();
        assert!(speaker.snapshot().unwrap().repeat);
        speaker.stop().unwrap();
        let stopped = speaker.snapshot().unwrap();
        assert!(stopped.repeat);
        assert_eq!(stopped.volume, 0.7);
        assert!(speaker.pause().is_err());
        assert!(speaker.resume().is_err());
        assert!(speaker.seek(0.0).is_err());
        assert!(speaker.say(" ", "", false).is_err());
        assert!(speaker.say_at_rate("test", "", f64::NAN).is_err());
    }

    #[test]
    fn windows_playback_controls_initialize_without_playing_audio() {
        let player = MediaPlayer::new().expect("Windows MediaPlayer activation");
        player.SetAutoPlay(false).unwrap();
        player
            .CommandManager()
            .unwrap()
            .SetIsEnabled(false)
            .unwrap();
        let session = player.PlaybackSession().unwrap();
        for volume in [0.0, 0.7, 1.0] {
            player.SetVolume(volume).unwrap();
            assert!((player.Volume().unwrap() - volume).abs() < 0.0001);
        }
        for repeat in [true, false] {
            player.SetIsLoopingEnabled(repeat).unwrap();
            assert_eq!(player.IsLoopingEnabled().unwrap(), repeat);
            player.Pause().unwrap();
            assert_ne!(
                session.PlaybackState().unwrap(),
                MediaPlaybackState::Playing
            );
        }
        for rate in [0.5, 1.0, 4.0] {
            session.SetPlaybackRate(rate).unwrap();
            assert!((session.PlaybackRate().unwrap() - rate).abs() < 0.0001);
        }
        assert_ne!(
            session.PlaybackState().unwrap(),
            MediaPlaybackState::Playing
        );
        player.Close().unwrap();
    }
}
