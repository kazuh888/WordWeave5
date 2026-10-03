//! Host-owned reading practice. No teaching data, recordings, or results are persisted here.
use qwen_audio::{
    evaluate, ApiKey, AudioInfo, AudioInput, Connection, CredentialStore, ErrorCode,
    EvaluationResult, EvaluationSnapshot, ProviderRequest, ReferenceText, SafeError,
    SendDisposition, ApiHost, Transport, TransportFailure, TransportFuture, REQUESTED_EFFORT,
    REQUESTED_MODEL,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc, Mutex,
};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

pub const WORDWEAVE_QWEN_CREDENTIAL_TARGET: &str = "WordWeave5.QwenReading.Connection.v1";
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
fn local(code: ErrorCode) -> SafeError {
    SafeError::new(code, SendDisposition::NotSent)
}

pub struct ReadingTarget {
    id: String,
    _version: Vec<u8>,
    reference: ReferenceText,
}
impl ReadingTarget {
    pub fn new(id: String, version: Vec<u8>, reference: String) -> Result<Self, SafeError> {
        Ok(Self {
            id,
            _version: version,
            reference: ReferenceText::new(reference)?,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreparedId {
    owner: u64,
    generation: u64,
    serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingPhase {
    Input,
    Confirming,
    Running,
    Completed,
    Failed,
    Cancelled,
    Invalidated,
    Closed,
}
#[derive(Clone, Debug)]
pub struct PreparedPreview {
    pub id: PreparedId,
    pub audio_label: String,
    pub audio_info: AudioInfo,
    pub audio_format: &'static str,
    pub reference: String,
    pub host: String,
    pub requested_model: &'static str,
    pub requested_effort: &'static str,
    pub purpose: &'static str,
    pub recipient: &'static str,
}
pub struct ReadingView {
    pub phase: ReadingPhase,
    pub target_id: String,
    pub audio_info: Option<AudioInfo>,
    pub audio_format: Option<&'static str>,
    pub preview: Option<PreparedPreview>,
    pub result: Option<EvaluationResult>,
    pub error: Option<SafeError>,
    pub disposition: SendDisposition,
}
pub struct SettingsView {
    pub host: String,
    pub credential_present: bool,
}
struct SettingsDraft {
    host: String,
    key: Option<Zeroizing<String>>,
}
struct Prepared {
    snapshot: EvaluationSnapshot,
    preview: PreparedPreview,
}
#[derive(Default)]
struct SendGate {
    cancelled: bool,
    started: bool,
}
struct GatedTransport {
    inner: Arc<dyn Transport>,
    gate: Arc<Mutex<SendGate>>,
}
impl Transport for GatedTransport {
    fn send<'a>(&'a self, request: ProviderRequest) -> TransportFuture<'a> {
        // Start and cancellation share one linearization point. Never hold this lock across await.
        let mut gate = self.gate.lock().unwrap_or_else(|e| e.into_inner());
        if gate.cancelled {
            return Box::pin(async { Err(TransportFailure::Network) });
        }
        gate.started = true;
        self.inner.send(request)
    }
}
struct Worker {
    generation: u64,
    token: CancellationToken,
    gate: Arc<Mutex<SendGate>>,
    receiver: mpsc::Receiver<Result<EvaluationResult, SafeError>>,
    handle: JoinHandle<()>,
}
pub struct ReadingController {
    target: ReadingTarget,
    owner: u64,
    generation: u64,
    serial: u64,
    phase: ReadingPhase,
    store: Arc<dyn CredentialStore>,
    transport: Arc<dyn Transport>,
    connection: Option<Connection>,
    audio: Option<AudioInput>,
    audio_label: String,
    prepared: Option<Prepared>,
    draft: Option<SettingsDraft>,
    worker: Option<Worker>,
    reapers: Vec<JoinHandle<()>>,
    result: Option<EvaluationResult>,
    error: Option<SafeError>,
    disposition: SendDisposition,
}
impl ReadingController {
    pub fn new(
        target: ReadingTarget,
        store: Arc<dyn CredentialStore>,
        transport: Arc<dyn Transport>,
    ) -> Result<Self, SafeError> {
        let owner = NEXT_OWNER
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| local(ErrorCode::InvalidState))?;
        let (connection, error) = match store.load() {
            Ok(c) => (c, None),
            Err(e) => (None, Some(e)),
        };
        Ok(Self {
            target,
            owner,
            generation: 1,
            serial: 0,
            phase: ReadingPhase::Input,
            store,
            transport,
            connection,
            audio: None,
            audio_label: String::new(),
            prepared: None,
            draft: None,
            worker: None,
            reapers: Vec::new(),
            result: None,
            error,
            disposition: SendDisposition::NotSent,
        })
    }
    pub fn view(&self) -> ReadingView {
        ReadingView {
            phase: self.phase,
            target_id: self.target.id.clone(),
            audio_info: self.audio.as_ref().map(|a| a.info().clone()),
            audio_format: self.audio.as_ref().map(|a| a.wire_format()),
            preview: self.prepared.as_ref().map(|p| p.preview.clone()),
            result: self.result.clone(),
            error: self.error.clone(),
            disposition: self.disposition,
        }
    }
    fn editable(&self) -> Result<(), SafeError> {
        match self.phase {
            ReadingPhase::Input | ReadingPhase::Confirming => Ok(()),
            ReadingPhase::Running => Err(local(ErrorCode::Busy)),
            _ => Err(local(ErrorCode::InvalidState)),
        }
    }
    fn advance(&mut self) -> Result<(), SafeError> {
        if let Some(n) = self.generation.checked_add(1) {
            self.generation = n;
            Ok(())
        } else {
            self.invalidate_target();
            Err(local(ErrorCode::InvalidState))
        }
    }
    pub fn clear_audio(&mut self) -> Result<(), SafeError> {
        self.editable()?;
        self.advance()?;
        self.audio = None;
        self.audio_label.clear();
        self.prepared = None;
        self.result = None;
        self.error = None;
        self.disposition = SendDisposition::NotSent;
        self.phase = ReadingPhase::Input;
        Ok(())
    }
    pub fn set_audio(&mut self, label: String, bytes: Vec<u8>) -> Result<(), SafeError> {
        self.clear_audio()?;
        match AudioInput::parse(bytes) {
            Ok(a) => {
                self.audio = Some(a);
                self.audio_label = label;
                Ok(())
            }
            Err(e) => {
                self.error = Some(e.clone());
                Err(e)
            }
        }
    }
    pub fn set_validated_audio(
        &mut self,
        label: String,
        audio: AudioInput,
    ) -> Result<(), SafeError> {
        self.clear_audio()?;
        self.audio = Some(audio);
        self.audio_label = label;
        Ok(())
    }
    pub fn begin_settings(&mut self) -> Result<SettingsView, SafeError> {
        self.editable()?;
        let host = self
            .connection
            .as_ref()
            .map(|c| c.host().as_str().to_owned())
            .unwrap_or_default();
        self.draft = Some(SettingsDraft {
            host: host.clone(),
            key: None,
        });
        Ok(SettingsView {
            host,
            credential_present: self.connection.is_some(),
        })
    }
    pub fn set_settings_draft(
        &mut self,
        host: String,
        key: Option<String>,
    ) -> Result<(), SafeError> {
        self.editable()?;
        if self.draft.is_none() {
            return Err(local(ErrorCode::InvalidState));
        }
        self.draft = Some(SettingsDraft {
            host,
            key: key.map(Zeroizing::new),
        });
        Ok(())
    }
    pub fn save_settings(&mut self) -> Result<(), SafeError> {
        self.editable()?;
        let draft = self
            .draft
            .as_ref()
            .ok_or_else(|| local(ErrorCode::InvalidState))?;
        let host = ApiHost::parse(&draft.host)?;
        let candidate = match draft.key.as_ref() {
            Some(key) => Connection::new(host, ApiKey::new(key.to_string())?),
            None => self
                .connection
                .as_ref()
                .ok_or_else(|| local(ErrorCode::InvalidKey))?
                .with_host(host)?,
        };
        // Persist first: a failed OS write must not change a pending confirmation or in-memory connection.
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| local(ErrorCode::InvalidState))?;
        self.store.save(&candidate)?;
        self.generation = next_generation;
        self.connection = Some(candidate);
        self.draft = None;
        self.prepared = None;
        self.error = None;
        self.phase = ReadingPhase::Input;
        Ok(())
    }
    pub fn cancel_settings(&mut self) {
        self.draft = None;
    }
    pub fn prepare(&mut self) -> Result<PreparedPreview, SafeError> {
        self.editable()?;
        if self.draft.is_some() {
            return Err(local(ErrorCode::InvalidState));
        }
        let audio = self
            .audio
            .clone()
            .ok_or_else(|| local(ErrorCode::AudioEmpty))?;
        let connection = self
            .connection
            .clone()
            .ok_or_else(|| local(ErrorCode::InvalidKey))?;
        let audio_format = audio.wire_format();
        let snapshot = EvaluationSnapshot::new(audio, self.target.reference.clone(), connection)?;
        self.serial = match self.serial.checked_add(1) {
            Some(n) => n,
            None => {
                self.invalidate_target();
                return Err(local(ErrorCode::InvalidState));
            }
        };
        let preview = PreparedPreview {
            id: PreparedId {
                owner: self.owner,
                generation: self.generation,
                serial: self.serial,
            },
            audio_label: self.audio_label.clone(),
            audio_info: snapshot.audio_info().clone(),
            audio_format,
            reference: snapshot.reference().as_str().to_owned(),
            host: snapshot.host().as_str().to_owned(),
            requested_model: REQUESTED_MODEL,
            requested_effort: REQUESTED_EFFORT,
            purpose: "例文の発音・強弱・リズムの改善点を確認する",
            recipient: "この評価画面のみ（教材・成績・Codexへは反映しない）",
        };
        self.prepared = Some(Prepared {
            snapshot,
            preview: preview.clone(),
        });
        self.phase = ReadingPhase::Confirming;
        self.error = None;
        Ok(preview)
    }
    pub fn human_send(
        &mut self,
        id: PreparedId,
        runtime: &tokio::runtime::Handle,
    ) -> Result<(), SafeError> {
        if self.phase == ReadingPhase::Running {
            return Err(local(ErrorCode::Busy));
        }
        if self.phase != ReadingPhase::Confirming
            || self.draft.is_some()
            || self.prepared.as_ref().map(|p| p.preview.id) != Some(id)
        {
            return Err(local(ErrorCode::InvalidState));
        }
        let prepared = self
            .prepared
            .take()
            .ok_or_else(|| local(ErrorCode::InvalidState))?;
        let token = CancellationToken::new();
        let gate = Arc::new(Mutex::new(SendGate::default()));
        let transport: Arc<dyn Transport> = Arc::new(GatedTransport {
            inner: self.transport.clone(),
            gate: gate.clone(),
        });
        let (sender, receiver) = mpsc::channel();
        let child_token = token.clone();
        self.phase = ReadingPhase::Running;
        self.result = None;
        self.error = None;
        let handle = runtime.spawn(async move {
            let result = evaluate(prepared.snapshot, transport, child_token).await;
            let _ = sender.send(result);
        });
        self.worker = Some(Worker {
            generation: self.generation,
            token,
            gate,
            receiver,
            handle,
        });
        Ok(())
    }
    pub fn poll(&mut self) {
        self.reapers.retain(|h| !h.is_finished());
        if self.phase != ReadingPhase::Running {
            return;
        }
        let Some(worker) = self.worker.as_ref() else {
            return;
        };
        let received = match worker.receiver.try_recv() {
            Ok(r) => Some(r),
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(SafeError::new(
                ErrorCode::Network,
                self.worker_disposition(),
            ))),
            Err(mpsc::TryRecvError::Empty) => None,
        };
        if let Some(result) = received {
            if worker.generation != self.generation || worker.token.is_cancelled() {
                return;
            }
            self.disposition = self.worker_disposition();
            match result {
                Ok(r) => {
                    self.result = Some(r);
                    self.phase = ReadingPhase::Completed;
                }
                Err(e) => {
                    self.error = Some(SafeError::new(e.code(), self.disposition));
                    self.phase = ReadingPhase::Failed;
                }
            }
            if let Some(worker) = self.worker.take() {
                self.reapers.push(worker.handle);
            }
        }
    }
    fn worker_disposition(&self) -> SendDisposition {
        self.worker.as_ref().map_or(self.disposition, |w| {
            if w.gate.lock().unwrap_or_else(|e| e.into_inner()).started {
                SendDisposition::MayHaveBeenSent
            } else {
                SendDisposition::NotSent
            }
        })
    }
    fn stop_worker(&mut self) {
        self.generation = self.generation.checked_add(1).unwrap_or(u64::MAX);
        if let Some(worker) = self.worker.take() {
            let mut gate = worker.gate.lock().unwrap_or_else(|e| e.into_inner());
            gate.cancelled = true;
            self.disposition = if gate.started {
                SendDisposition::MayHaveBeenSent
            } else {
                SendDisposition::NotSent
            };
            drop(gate);
            worker.token.cancel();
            worker.handle.abort();
            self.reapers.push(worker.handle);
        }
        self.prepared = None;
        self.draft = None;
        self.result = None;
    }
    pub fn cancel(&mut self) {
        self.stop_worker();
        if !matches!(self.phase, ReadingPhase::Closed | ReadingPhase::Invalidated) {
            self.phase = ReadingPhase::Cancelled;
            self.error = Some(SafeError::new(ErrorCode::Cancelled, self.disposition));
        }
    }
    pub fn invalidate_target(&mut self) {
        self.stop_worker();
        if self.phase != ReadingPhase::Closed {
            self.phase = ReadingPhase::Invalidated;
            self.error = Some(local(ErrorCode::InvalidState));
        }
    }
    pub fn restart(&mut self) -> Result<(), SafeError> {
        if !matches!(
            self.phase,
            ReadingPhase::Completed | ReadingPhase::Failed | ReadingPhase::Cancelled
        ) {
            return Err(local(ErrorCode::InvalidState));
        }
        if !self.reapers.is_empty() {
            return Err(local(ErrorCode::Busy));
        }
        self.advance()?;
        self.phase = ReadingPhase::Input;
        self.result = None;
        self.error = None;
        self.prepared = None;
        self.disposition = SendDisposition::NotSent;
        Ok(())
    }
    pub fn close(&mut self) {
        self.stop_worker();
        self.phase = ReadingPhase::Closed;
        self.audio = None;
        self.audio_label.clear();
        self.connection = None;
        self.error = None;
    }
}
impl Drop for ReadingController {
    fn drop(&mut self) {
        self.close();
        for h in &self.reapers {
            h.abort();
        }
    }
}
