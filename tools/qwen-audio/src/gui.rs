use crate::error::err;
use crate::*;
use eframe::egui;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;
pub enum GuiPhase {
    AwaitingUser,
    Running,
    Completed(EvaluationResult),
    Failed(SafeError),
    Cancelled,
}
pub struct GuiModel {
    pub reference_text: String,
    pub file_label: String,
    pub audio_info: Option<AudioInfo>,
    pub audio_format: Option<&'static str>,
    pub host_label: String,
    pub credential_present: bool,
    pub mcp_origin: bool,
    pub phase: GuiPhase,
    pub notice: Option<String>,
}
#[derive(Clone, Copy)]
pub enum GuiAction {
    SelectAudio,
    OpenSettings,
    Send,
    Cancel,
    NewSession,
}
fn label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.add(egui::Label::new(text).wrap());
}
fn region_label(host: &str) -> &'static str {
    ApiHost::parse(host)
        .map(|host| host.region().label())
        .unwrap_or("地域未設定")
}
pub fn render(ui: &mut egui::Ui, model: &mut GuiModel) -> Option<GuiAction> {
    render_with_loading(ui, model, false)
}
fn render_with_loading(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    loading: bool,
) -> Option<GuiAction> {
    let mut action = None;
    let awaiting = matches!(model.phase, GuiPhase::AwaitingUser);
    let running = matches!(model.phase, GuiPhase::Running);
    ui.heading("Qwen 音読練習");
    label(
        ui,
        "AIによる練習用フィードバック。客観的な発音点数ではない。",
    );
    label(
        ui,
        format!(
            "要求: {} / 推論 effort: {}",
            REQUESTED_MODEL, REQUESTED_EFFORT
        ),
    );
    label(
        ui,
        match &model.phase {
            GuiPhase::AwaitingUser if loading => "状態: 音声を読み込み・検査中",
            GuiPhase::AwaitingUser => "状態: 入力・確認待ち",
            GuiPhase::Running => "状態: 評価中",
            GuiPhase::Completed(result) => match result.feedback.assessment {
                Assessment::Assessed => "状態: 評価完了",
                Assessment::ReferenceMismatch => "状態: 別の英文が聞き取れた",
                Assessment::Unassessable => "状態: 音声を評価できない",
            },
            GuiPhase::Failed(_) => "状態: 評価失敗",
            GuiPhase::Cancelled => "状態: 取消し",
        },
    );
    if (awaiting || running) && crate::controls::button(ui, "取消し", true).clicked() {
        action = Some(GuiAction::Cancel);
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_max_width(ui.available_width());
        if !awaiting && !running {
            match &model.phase {
                GuiPhase::Completed(result) => result_card(ui, result, &model.reference_text),
                GuiPhase::Failed(error) => {
                    ui.heading("評価できなかった理由");
                    label(ui, error.message());
                    if error.disposition() == SendDisposition::MayHaveBeenSent {
                        label(ui, "サーバーでの完了・課金状態は不明である。");
                    }
                }
                GuiPhase::Cancelled => label(ui, "評価を取り消した。送信開始後のサーバー処理・課金取消しは保証できない。"),
                _ => {}
            }
            if model.mcp_origin {
                label(ui, "この構造化結果はCodexから取得可能である。次のCodex評価を開始するとこの結果画面を閉じる。再評価はCodexで新しく開始する。");
            } else if crate::controls::button(ui,"もう一度練習",true).clicked() {
                action=Some(GuiAction::NewSession);
            }
            if let Some(notice)=&model.notice { label(ui,notice); }
            ui.collapsing("確認した入力と接続先", |ui| {
                label(ui,format!("{} / {}",region_label(&model.host_label),model.host_label));
                label(ui,&model.file_label);
                label(ui,&model.reference_text);
            });
            return;
        }
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.vertical(|ui| {
                label(ui, if model.credential_present { "接続設定: 保存済み" } else { "接続設定: 未設定" });
                label(ui, format!("{} / {}", region_label(&model.host_label), model.host_label));
                if crate::controls::button(ui, "接続設定", awaiting).clicked() {
                    action = Some(GuiAction::OpenSettings);
                }
            });
        });
        input_card(ui, model, awaiting, &mut action);
        consent_card(ui, model, awaiting && !loading, &mut action);
        if let Some(notice) = &model.notice { label(ui, notice); }
    });
    action
}
fn input_card(
    ui: &mut egui::Ui,
    model: &mut GuiModel,
    awaiting: bool,
    action: &mut Option<GuiAction>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical(|ui| {
            label(
                ui,
                "音声: WAV / MP3 / AAC / AMR / 3GP / 3GPP。1/2チャンネル、8～48kHz、60秒以下、全体6MiB以下。WAVはPCM16、AACはADTS、3GP/3GPPは音声のみ。",
            );
            if crate::controls::button(ui, "音声を選択", awaiting).clicked() {
                *action = Some(GuiAction::SelectAudio);
            }
            label(ui, &model.file_label);
            if let Some(format) = model.audio_format {
                label(ui, format!("検査した形式: {}", format.to_ascii_uppercase()));
            }
            if let Some(info) = &model.audio_info {
                label(
                    ui,
                    format!(
                        "{:.3}秒 / {}Hz / {}ch / {} bytes",
                        info.duration_seconds, info.sample_rate, info.channels, info.byte_len
                    ),
                );
            }
            label(
                ui,
                "選択時の音声を保持する。最新内容を使う場合は再選択する。",
            );
            label(
                ui,
                format!(
                    "参照英文（10,000文字以下）: {}文字",
                    model.reference_text.chars().count()
                ),
            );
            ui.add_enabled(
                awaiting,
                egui::TextEdit::multiline(&mut model.reference_text)
                    .desired_width(f32::INFINITY)
                    .desired_rows(5),
            );
        });
    });
}
fn consent_card(
    ui: &mut egui::Ui,
    model: &GuiModel,
    awaiting: bool,
    action: &mut Option<GuiAction>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical(|ui| {
            label(ui, format!("音声と上記の参照英文をAlibaba Cloud（{}）へ送信する。有料APIによる従量課金が発生する。回答上限は4096 tokensであり、思考を含む総量や料金の上限ではない。", region_label(&model.host_label)));
            if model.mcp_origin {
                label(ui, "構造化評価結果・聞き取った英文はCodexから取得可能となる。次のCodex評価を開始するとこの結果画面を閉じる。");
            }
            if crate::controls::button(ui, "評価を送信",
                awaiting && model.credential_present && model.audio_info.is_some()).clicked() {
                *action = Some(GuiAction::Send);
            }
        });
    });
}
fn result_card(ui: &mut egui::Ui, result: &EvaluationResult, reference: &str) {
    let feedback = &result.feedback;
    ui.heading(match feedback.assessment {
        Assessment::Assessed => "結果",
        Assessment::ReferenceMismatch => "別の英文が聞き取れた",
        Assessment::Unassessable => "音声を評価できない",
    });
    label(ui, &feedback.summary);
    if feedback.assessment == Assessment::ReferenceMismatch {
        label(ui, "表示された例文");
        label(ui, reference);
    }
    label(ui, "聞き取った英文（モデルの推定）");
    label(ui, feedback.heard_text.as_deref().unwrap_or("評価不能"));
    if let Some(reason) = &feedback.unassessable_reason {
        label(ui, reason);
    }
    if feedback.assessment == Assessment::ReferenceMismatch {
        label(
            ui,
            "表示された例文を読み直して再録音する。例文の発音の良否は評価していない。",
        );
    } else if feedback.assessment == Assessment::Unassessable {
        label(ui, "音声を確認し、再録音または再選択する。");
    }
    if feedback.assessment == Assessment::Assessed {
        ui.heading("良い点");
        for strength in &feedback.strengths {
            label(ui, strength);
        }
        ui.heading("改善・次の練習");
        for improvement in &feedback.improvements {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.vertical(|ui| {
                    label(ui, &improvement.reference_excerpt);
                    label(ui, &improvement.observation);
                    label(ui, &improvement.practice);
                });
            });
        }
    }
    label(
        ui,
        format!(
            "応答 model: {} / effort: {}",
            result.actual_model.as_deref().unwrap_or("未取得"),
            result.actual_effort.as_deref().unwrap_or("未取得")
        ),
    );
    if let Some(usage) = &result.usage {
        let value = |v: Option<u64>| v.map(|v| v.to_string()).unwrap_or("未取得".into());
        label(
            ui,
            format!(
                "使用 tokens: 入力 {} / 出力 {} / 合計 {}",
                value(usage.prompt_tokens),
                value(usage.completion_tokens),
                value(usage.total_tokens)
            ),
        );
    } else {
        label(ui, "トークン使用量: 未取得");
    }
}
pub struct SettingsView {
    pub host_label: String,
    pub credential_present: bool,
}
struct SettingsDraft {
    host: String,
    key: Option<Zeroizing<String>>,
    region: Region,
}
pub struct GuiController {
    model: GuiModel,
    audio: Option<AudioInput>,
    connection: Option<Connection>,
    credentials: Arc<dyn CredentialStore>,
    transport: Arc<dyn Transport>,
    draft: Option<SettingsDraft>,
    sessions: SessionManager,
    id: SessionId,
    worker: Option<tokio::task::JoinHandle<Result<EvaluationResult, SafeError>>>,
    cancel: CancellationToken,
    pub(crate) supervisor: bool,
    pub(crate) proposal_sent: bool,
}
impl GuiController {
    pub fn new(
        reference: String,
        credentials: Arc<dyn CredentialStore>,
        transport: Arc<dyn Transport>,
    ) -> Result<Self, SafeError> {
        let mut sessions = SessionManager::new();
        let id = sessions.start()?.session_id;
        let (connection, notice) = match credentials.load() {
            Ok(c) => (c, None),
            Err(e) => (None, Some(e.message().into())),
        };
        let model = GuiModel {
            reference_text: reference,
            file_label: "未選択".into(),
            audio_info: None,
            audio_format: None,
            host_label: connection
                .as_ref()
                .map(|c| c.host().as_str().into())
                .unwrap_or_default(),
            credential_present: connection.is_some(),
            mcp_origin: false,
            phase: GuiPhase::AwaitingUser,
            notice,
        };
        Ok(Self {
            model,
            audio: None,
            connection,
            credentials,
            transport,
            draft: None,
            sessions,
            id,
            worker: None,
            cancel: CancellationToken::new(),
            supervisor: false,
            proposal_sent: false,
        })
    }
    pub fn model(&self) -> &GuiModel {
        &self.model
    }
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<GuiAction> {
        render(ui, &mut self.model)
    }
    #[doc(hidden)]
    pub fn render_with_loading(&mut self, ui: &mut egui::Ui, loading: bool) -> Option<GuiAction> {
        render_with_loading(ui, &mut self.model, loading)
    }
    #[doc(hidden)]
    pub fn notice(&mut self, text: &str) {
        self.model.notice = Some(text.into());
    }
    #[doc(hidden)]
    pub fn stop_worker(&mut self) {
        self.cancel.cancel();
        if let Some(w) = self.worker.take() {
            w.abort()
        }
    }
    fn editable(&self) -> Result<(), SafeError> {
        if !matches!(self.model.phase, GuiPhase::AwaitingUser) {
            return Err(err(ErrorCode::InvalidState));
        }
        Ok(())
    }
    pub fn set_reference(&mut self, text: String) -> Result<(), SafeError> {
        self.editable()?;
        self.model.reference_text = text;
        Ok(())
    }
    pub fn select_audio(&mut self, audio: AudioInput, file_label: String) -> Result<(), SafeError> {
        self.editable()?;
        self.model.audio_info = Some(audio.info().clone());
        self.model.audio_format = Some(audio.wire_format());
        self.model.file_label = std::path::Path::new(&file_label)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        self.audio = Some(audio);
        Ok(())
    }
    pub fn clear_audio(&mut self) -> Result<(), SafeError> {
        self.editable()?;
        self.audio = None;
        self.model.audio_info = None;
        self.model.audio_format = None;
        self.model.file_label = "未選択".into();
        Ok(())
    }
    pub fn begin_settings(&mut self) -> Result<SettingsView, SafeError> {
        self.editable()?;
        self.draft = Some(SettingsDraft {
            host: self.model.host_label.clone(),
            key: None,
            region: self
                .connection
                .as_ref()
                .map(|c| c.host().region())
                .unwrap_or_default(),
        });
        Ok(SettingsView {
            host_label: self.model.host_label.clone(),
            credential_present: self.connection.is_some(),
        })
    }
    pub fn set_settings_draft(
        &mut self,
        host: String,
        key: Option<String>,
    ) -> Result<(), SafeError> {
        let region = ApiHost::parse(&host)?.region();
        self.set_settings_draft_for_region(host, key, region)
    }
    pub fn settings_region(&self) -> Region {
        self.draft.as_ref().map(|d| d.region).unwrap_or_default()
    }
    pub fn set_settings_region(&mut self, region: Region) -> Result<(), SafeError> {
        self.editable()?;
        let draft = self
            .draft
            .as_mut()
            .ok_or_else(|| err(ErrorCode::InvalidState))?;
        if draft.region != region {
            draft.region = region;
            draft.host.clear();
            draft.key = None;
        }
        Ok(())
    }
    pub fn set_settings_draft_for_region(
        &mut self,
        host: String,
        key: Option<String>,
        region: Region,
    ) -> Result<(), SafeError> {
        self.editable()?;
        if self.draft.is_none() {
            return Err(err(ErrorCode::InvalidState));
        }
        self.draft = Some(SettingsDraft {
            host,
            key: key.map(Zeroizing::new),
            region,
        });
        Ok(())
    }
    pub fn save_settings(&mut self) -> Result<(), SafeError> {
        self.editable()?;
        let draft = self
            .draft
            .as_ref()
            .ok_or_else(|| err(ErrorCode::InvalidState))?;
        let host = ApiHost::parse_for_region(&draft.host, draft.region)?;
        let connection = match &draft.key {
            Some(k) if !k.is_empty() => Connection::new(host, ApiKey::new(k.to_string())?),
            Some(_) | None => self
                .connection
                .as_ref()
                .ok_or_else(|| err(ErrorCode::InvalidKey))?
                .with_host(host)?,
        };
        self.credentials.save(&connection)?;
        self.model.host_label = connection.host().as_str().into();
        self.model.credential_present = true;
        self.connection = Some(connection);
        self.draft = None;
        self.model.notice = Some("接続設定を保存した".into());
        Ok(())
    }
    pub fn cancel_settings(&mut self) {
        self.draft = None;
    }
    fn snapshot(&self) -> Result<EvaluationSnapshot, SafeError> {
        if self.draft.is_some() {
            return Err(err(ErrorCode::InvalidState));
        }
        EvaluationSnapshot::new(
            self.audio
                .clone()
                .ok_or_else(|| err(ErrorCode::AudioEmpty))?,
            ReferenceText::new(self.model.reference_text.clone())?,
            self.connection
                .clone()
                .ok_or_else(|| err(ErrorCode::InvalidKey))?,
        )
    }
    pub fn send(&mut self, runtime: &tokio::runtime::Handle) -> Result<SessionView, SafeError> {
        self.editable()?;
        if self.supervisor {
            return Err(err(ErrorCode::InvalidState));
        }
        self.spawn(runtime)
    }
    fn spawn(&mut self, runtime: &tokio::runtime::Handle) -> Result<SessionView, SafeError> {
        let snapshot = self.snapshot()?;
        let view = self.sessions.mark_running(&self.id)?;
        self.model.phase = GuiPhase::Running;
        self.cancel = CancellationToken::new();
        self.worker = Some(runtime.spawn(evaluate(
            snapshot,
            self.transport.clone(),
            self.cancel.clone(),
        )));
        Ok(view)
    }
    fn apply(
        &mut self,
        result: Result<EvaluationResult, SafeError>,
    ) -> Result<SessionView, SafeError> {
        let view = match result {
            Ok(r) => self.sessions.complete(&self.id, r)?,
            Err(e) if e.code() == ErrorCode::Cancelled => self.sessions.cancel(&self.id)?,
            Err(e) => self.sessions.fail(&self.id, e)?,
        };
        self.show(&view);
        Ok(view)
    }
    fn show(&mut self, view: &SessionView) {
        self.model.phase = match &view.state {
            SessionState::AwaitingUser => GuiPhase::AwaitingUser,
            SessionState::Running => GuiPhase::Running,
            SessionState::Completed { result } => GuiPhase::Completed(result.clone()),
            SessionState::Failed { error } => GuiPhase::Failed(error.clone()),
            SessionState::Cancelled { .. } => GuiPhase::Cancelled,
        };
    }
    pub fn poll(&mut self) -> Result<SessionView, SafeError> {
        if self.supervisor {
            return self.sessions.get(&self.id);
        }
        if self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            let worker = self.worker.take().unwrap();
            let result = futures_util::FutureExt::now_or_never(worker)
                .ok_or_else(|| err(ErrorCode::InvalidState))?
                .unwrap_or_else(|_| Err(crate::error::sent_err(ErrorCode::Network)));
            self.apply(result)?;
        }
        self.sessions.get(&self.id)
    }
    pub async fn wait_for_terminal(&mut self) -> Result<SessionView, SafeError> {
        if self.supervisor {
            return Err(err(ErrorCode::InvalidState));
        }
        if let Some(worker) = self.worker.take() {
            let r = worker
                .await
                .unwrap_or_else(|_| Err(crate::error::sent_err(ErrorCode::Network)));
            self.apply(r)?;
        }
        self.sessions.get(&self.id)
    }
    pub fn cancel(&mut self) -> Result<SessionView, SafeError> {
        let v = self.sessions.cancel(&self.id)?;
        self.cancel.cancel();
        if let Some(w) = self.worker.take() {
            w.abort()
        }
        if !self.supervisor {
            self.show(&v)
        }
        Ok(v)
    }
    pub fn new_session(&mut self) -> Result<SessionView, SafeError> {
        if self.supervisor || !self.sessions.get(&self.id)?.state.terminal() {
            return Err(err(ErrorCode::InvalidState));
        }
        let v = self.sessions.start()?;
        self.id = v.session_id.clone();
        self.model.notice = None;
        self.show(&v);
        self.proposal_sent = false;
        Ok(v)
    }
    #[doc(hidden)]
    pub fn child(
        reference: String,
        id: SessionId,
        credentials: Arc<dyn CredentialStore>,
        transport: Arc<dyn Transport>,
    ) -> Result<Self, SafeError> {
        let mut c = Self::new(reference, credentials, transport)?;
        c.id = id;
        c.supervisor = true;
        c.model.mcp_origin = true;
        Ok(c)
    }
    #[doc(hidden)]
    pub fn prepare_intent(&mut self) -> Result<ChildMessage, SafeError> {
        self.editable()?;
        self.snapshot()?;
        self.model.phase = GuiPhase::Running;
        Ok(ChildMessage::SendIntent {
            session_id: self.id.clone(),
            reference_text: self.model.reference_text.clone(),
        })
    }
    #[doc(hidden)]
    pub fn grant(&mut self, runtime: &tokio::runtime::Handle) -> Result<(), SafeError> {
        if !self.supervisor
            || !matches!(self.model.phase, GuiPhase::Running)
            || self.worker.is_some()
            || self.proposal_sent
            || self.cancel.is_cancelled()
        {
            return Err(err(ErrorCode::InvalidState));
        }
        let snapshot = self.snapshot()?;
        self.cancel = CancellationToken::new();
        self.worker = Some(runtime.spawn(evaluate(
            snapshot,
            self.transport.clone(),
            self.cancel.clone(),
        )));
        Ok(())
    }
    #[doc(hidden)]
    pub fn take_proposal(&mut self) -> Option<ChildMessage> {
        if self.worker.as_ref().is_some_and(|w| w.is_finished()) {
            let w = self.worker.take().unwrap();
            self.proposal_sent = true;
            self.model.notice = Some("結果を確認中".into());
            let r = futures_util::FutureExt::now_or_never(w)
                .unwrap()
                .unwrap_or_else(|_| Err(crate::error::sent_err(ErrorCode::Network)));
            Some(match r {
                Ok(result) => ChildMessage::Completed {
                    session_id: self.id.clone(),
                    result,
                },
                Err(error) => ChildMessage::Failed {
                    session_id: self.id.clone(),
                    error,
                },
            })
        } else {
            None
        }
    }
    #[doc(hidden)]
    pub fn committed(&mut self, v: &SessionView) -> Result<(), SafeError> {
        if v.session_id != self.id || !v.state.terminal() {
            return Err(err(ErrorCode::ProtocolInvalid));
        }
        self.cancel.cancel();
        if let Some(w) = self.worker.take() {
            w.abort()
        }
        self.show(v);
        self.model.notice = None;
        Ok(())
    }
}
impl Drop for GuiController {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(w) = self.worker.take() {
            w.abort()
        }
    }
}

pub fn setup_fonts(ctx: &egui::Context) -> bool {
    for path in [
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/msgothic.ttc",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("japanese".into(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "japanese".into());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("japanese".into());
            ctx.set_fonts(fonts);
            return true;
        }
    }
    false
}
#[doc(hidden)]
pub fn settings_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    crate::controls::button(ui, label, true)
}
