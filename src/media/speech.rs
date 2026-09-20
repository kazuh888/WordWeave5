//! Local Windows speech synthesis and independently controllable playback.
//! Synthesis is polled, not blocked on the UI thread. No audio is sent to a server.
use std::sync::{Arc, Mutex};
use windows::{
    core::HSTRING,
    Foundation::{AsyncStatus, IAsyncOperation, TimeSpan, TypedEventHandler},
    Media::{
        Core::MediaSource,
        Playback::{
            MediaPlaybackState, MediaPlayer, MediaPlayerAudioCategory, MediaPlayerFailedEventArgs,
        },
        SpeechSynthesis::{SpeechSynthesisStream, SpeechSynthesizer},
    },
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
    _stream: SpeechSynthesisStream,
    failure: Arc<Mutex<Option<String>>>,
}
impl SpeechPlayback {
    fn new(stream: SpeechSynthesisStream, rate: f64, play: bool) -> Result<Self, String> {
        let player = MediaPlayer::new().map_err(native_error)?;
        player.SetAutoPlay(false).map_err(native_error)?;
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
        player.MediaFailed(&TypedEventHandler::new(move |_: &Option<MediaPlayer>, args: &Option<MediaPlayerFailedEventArgs>| {
            let code = args.as_ref().and_then(|a| a.ExtendedErrorCode().ok());
            if let Ok(mut error) = failed.lock() {
                *error = Some(match code {
                    Some(code) => format!("読み上げの再生に失敗した (0x{:08X})。出力デバイスを確認してください。", code.0 as u32),
                    None => "読み上げの再生に失敗した。出力デバイスを確認してください。".into(),
                });
            }
            Ok(())
        })).map_err(native_error)?;
        let source =
            MediaSource::CreateFromStream(&stream, &stream.ContentType().map_err(native_error)?)
                .map_err(native_error)?;
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
        self.engine = Some(SpeechPlayback::new(
            stream,
            pending.rate,
            pending.play_when_ready,
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
                ..Default::default()
            });
        }
        let Some(engine) = &self.engine else {
            return Ok(PlaybackSnapshot::default());
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
        })
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if let Some(pending) = &mut self.pending {
            pending.play_when_ready = false;
            return Ok(());
        }
        let engine = self.engine.as_ref().ok_or("読み上げる音声がありません。")?;
        engine.player.Pause().map_err(native_error)
    }
    pub fn resume(&mut self) -> Result<(), String> {
        if let Some(pending) = &mut self.pending {
            pending.play_when_ready = true;
            return Ok(());
        }
        let engine = self.engine.as_ref().ok_or("読み上げる音声がありません。")?;
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
    pub fn seek(&mut self, seconds: f64) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("音声を準備中のため、再生位置はまだ変更できません。".into());
        }
        let engine = self.engine.as_ref().ok_or("読み上げる音声がありません。")?;
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

    #[test]
    fn unloaded_player_is_safe_without_a_voice_or_audio_device() {
        let mut speaker = Speaker {
            engine: None,
            pending: None,
            voices: vec![],
        };
        assert!(!speaker.snapshot().unwrap().loaded);
        speaker.stop().unwrap();
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
