//! Qwen credentials are edited and explicitly committed independently of Progress.
use super::{controls::UiControls as _, settings_ui::SettingsSection, ux, Page, WordApp};
use eframe::egui;
#[cfg(not(test))]
use qwen_audio::WindowsCredentialStore;
use qwen_audio::{
    ApiHost, ApiKey, Connection, CredentialStore, ErrorCode, ProbeError, ProbeSuccess,
    ProbeTransport, Region, ReqwestProbeTransport, SafeError, SendDisposition,
};
use std::sync::{mpsc, Arc};
use tokio_util::sync::CancellationToken;
#[cfg(not(test))]
use wordweave5::qwen_reading::WORDWEAVE_QWEN_CREDENTIAL_TARGET;
use zeroize::Zeroizing;

pub(super) type Summary = Result<Option<(Region, String)>, SafeError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProbeState {
    Untested,
    Running,
    Completed(Result<ProbeSuccess, ProbeError>),
}

struct ProbeJob {
    revision: u64,
    cancel: CancellationToken,
    receiver: mpsc::Receiver<Result<ProbeSuccess, ProbeError>>,
}

pub(super) fn default_store() -> Arc<dyn CredentialStore> {
    #[cfg(test)]
    {
        super::qwen_reading_ui::preview_data::Store::new(None)
    }
    #[cfg(not(test))]
    {
        Arc::new(
            WindowsCredentialStore::for_target(WORDWEAVE_QWEN_CREDENTIAL_TARGET)
                .expect("constant credential namespace"),
        )
    }
}

pub(super) struct QwenConnectionEditor {
    store: Arc<dyn CredentialStore>,
    baseline: Option<Connection>,
    pub region: Region,
    pub host: String,
    pub key: Zeroizing<String>,
    pub error: Option<SafeError>,
    pub discard_requested: bool,
    pub saved: bool,
    pub probe_state: ProbeState,
    transport: Option<Arc<dyn ProbeTransport>>,
    revision: u64,
    probe_job: Option<ProbeJob>,
    probe_inputs: Option<(Region, String, Zeroizing<String>)>,
    focus_pending: bool,
    #[cfg(debug_assertions)]
    pub preview_tail: bool,
}

impl QwenConnectionEditor {
    pub fn new(store: Arc<dyn CredentialStore>) -> Result<Self, SafeError> {
        let baseline = store.load()?;
        let region = baseline
            .as_ref()
            .map_or(Region::Tokyo, |c| c.host().region());
        let host = baseline
            .as_ref()
            .map(|c| c.host().as_str().to_owned())
            .unwrap_or_default();
        Ok(Self {
            store,
            baseline,
            region,
            host,
            key: Zeroizing::new(String::new()),
            error: None,
            discard_requested: false,
            saved: false,
            probe_state: ProbeState::Untested,
            transport: None,
            revision: 0,
            probe_job: None,
            probe_inputs: None,
            focus_pending: true,
            #[cfg(debug_assertions)]
            preview_tail: false,
        })
    }
    pub fn new_with_transport(
        store: Arc<dyn CredentialStore>,
        transport: Arc<dyn ProbeTransport>,
    ) -> Result<Self, SafeError> {
        let mut editor = Self::new(store)?;
        editor.transport = Some(transport);
        Ok(editor)
    }
    pub fn dirty(&self) -> bool {
        !self.key.is_empty()
            || self.host != self.baseline.as_ref().map_or("", |c| c.host().as_str())
            || self.region
                != self
                    .baseline
                    .as_ref()
                    .map_or(Region::Tokyo, |c| c.host().region())
    }
    pub fn select_region(&mut self, region: Region) {
        if region != self.region {
            self.invalidate_probe();
            self.region = region;
            self.host.clear();
            self.key.clear();
            self.error = None;
        }
    }
    pub fn resolve_connection(&self) -> Result<Connection, SafeError> {
        let host = ApiHost::parse_for_region(&self.host, self.region)?;
        if self.key.is_empty() {
            self.baseline
                .as_ref()
                .ok_or_else(|| SafeError::new(ErrorCode::InvalidKey, SendDisposition::NotSent))?
                .with_host(host)
        } else {
            Ok(Connection::new(host, ApiKey::new(self.key.to_string())?))
        }
    }
    pub fn save(&mut self) -> Result<(), SafeError> {
        self.invalidate_probe();
        let next = self.resolve_connection()?;
        self.store.save(&next)?;
        self.host = next.host().as_str().to_owned();
        self.baseline = Some(next);
        self.key.clear();
        self.error = None;
        self.saved = true;
        Ok(())
    }
    /// Returns true only when no draft needs a discard decision.
    pub fn request_close(&mut self) -> bool {
        self.invalidate_probe();
        if self.dirty() {
            self.discard_requested = true;
            false
        } else {
            true
        }
    }
    pub fn summary(&self) -> Summary {
        Ok(self
            .baseline
            .as_ref()
            .map(|c| (c.host().region(), c.host().as_str().to_owned())))
    }
    pub fn invalidate_probe(&mut self) {
        if let Some(job) = self.probe_job.take() {
            job.cancel.cancel();
        }
        self.revision = self.revision.wrapping_add(1);
        self.probe_inputs = None;
        self.probe_state = ProbeState::Untested;
    }
    fn refresh_probe_inputs(&mut self) {
        if self
            .probe_inputs
            .as_ref()
            .is_some_and(|(region, host, key)| {
                *region != self.region || *host != self.host || **key != *self.key
            })
        {
            self.invalidate_probe();
        }
    }
    pub fn start_probe(&mut self, ctx: &egui::Context) -> Result<(), SafeError> {
        self.refresh_probe_inputs();
        if self.probe_state == ProbeState::Running {
            return Ok(());
        }
        self.invalidate_probe();
        let connection = self.resolve_connection()?;
        self.error = None;
        self.probe_inputs = Some((self.region, self.host.clone(), self.key.clone()));
        let transport = match self.transport.clone() {
            Some(transport) => transport,
            None => match ReqwestProbeTransport::new() {
                Ok(transport) => Arc::new(transport),
                Err(error) => {
                    self.probe_state = ProbeState::Completed(Err(error));
                    return Ok(());
                }
            },
        };
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::channel();
        let repaint = ctx.clone();
        // The worker owns its runtime. Neither polling nor dropping the editor joins it.
        let spawn = std::thread::Builder::new()
            .name("qwen-connection-probe".into())
            .spawn(move || {
                let result = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime.block_on(qwen_audio::probe_connection(
                        connection,
                        transport,
                        worker_cancel,
                    )),
                    Err(_) => Err(ProbeError::Network),
                };
                let _ = sender.send(result);
                repaint.request_repaint();
            });
        match spawn {
            Ok(_) => {
                self.probe_job = Some(ProbeJob {
                    revision: self.revision,
                    cancel,
                    receiver,
                });
                self.probe_state = ProbeState::Running;
            }
            Err(_) => self.probe_state = ProbeState::Completed(Err(ProbeError::Network)),
        }
        Ok(())
    }
    pub fn poll_probe(&mut self) {
        self.refresh_probe_inputs();
        let result = self
            .probe_job
            .as_ref()
            .and_then(|job| match job.receiver.try_recv() {
                Ok(result) => Some((job.revision, result)),
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some((job.revision, Err(ProbeError::Network)))
                }
                Err(mpsc::TryRecvError::Empty) => None,
            });
        if let Some((revision, result)) = result {
            self.probe_job = None;
            if revision == self.revision && self.probe_state == ProbeState::Running {
                self.probe_state = ProbeState::Completed(result);
            }
        }
    }
    pub fn cancel_probe(&mut self) {
        self.invalidate_probe();
        self.probe_inputs = Some((self.region, self.host.clone(), self.key.clone()));
        self.probe_state = ProbeState::Completed(Err(ProbeError::Cancelled));
    }
    pub fn render(&mut self, ui: &mut egui::Ui, input_fill: egui::Color32) -> bool {
        self.poll_probe();
        let mut close = false;
        ui.vertical(|ui| {
                    ui.set_max_width(ui.available_width());
                    if self.discard_requested {
                        ui.separator();
                        ui.strong("未保存のQwen接続設定を破棄するか？");
                        ui.label("保存済みの接続は変更しない。アプリを終了する場合は、この画面を閉じた後にもう一度ウィンドウを閉じてください。");
                        ui.horizontal_wrapped(|ui| {
                            if ui.ww_button("破棄して閉じる").clicked() { close = true; }
                            if ui.ww_button("編集へ戻る").clicked() { self.discard_requested = false; }
                        });
                        return;
                    }
                    ui.label("リージョン（契約した地域）");
                    let mut region = self.region;
                    egui::ComboBox::from_id_salt("qwen-region").selected_text(region.label())
                        .show_ui(ui, |ui| { for item in Region::ALL { ui.ww_selectable_value(&mut region, item, item.label()); } });
                    self.select_region(region);
                    ui.label("API Host（Workspace専用URL）");
                    let host = ui.add(egui::TextEdit::singleline(&mut self.host).id_salt("qwen-host").background_color(input_fill).desired_width(ui.available_width()));
                    record_control(ui, "qwen-host-input", &host);
                    if self.focus_pending && ui.is_enabled() && !ui.is_sizing_pass() {
                        host.scroll_to_me_animation(Some(egui::Align::Center), egui::style::ScrollAnimation::none());
                        if ui.clip_rect().contains_rect(host.rect) && !ui.ctx().will_discard() {
                            host.request_focus();
                            self.focus_pending = false;
                        }
                    }
                    if host.changed() { self.invalidate_probe(); self.error = None; }
                    ui.add(egui::Label::new(format!("形式：{}", self.region.example_url())).wrap().selectable(true));
                    ui.label("リージョンとURLを一致させる。地域またはWorkspaceを変更する場合は、その送信先のAPIキーを入力し直す。");
                    ui.label(if self.baseline.is_some() { "APIキー：保存済み（表示しない）。同じURLなら空欄で維持。" } else { "APIキー：未設定" });
                    let key = ui.add(egui::TextEdit::singleline(&mut *self.key).id_salt("qwen-key").password(true).background_color(input_fill).desired_width(ui.available_width()));
                    record_control(ui, "qwen-key-input", &key);
                    if key.changed() { self.invalidate_probe(); self.error = None; }
                    ui.add_space(8.0);
                    ui.label("編集中の接続を確認する。確認しても保存されない。音声・例文は送信しない。");
                    ui.horizontal_wrapped(|ui| {
                        let check = ui.add_enabled(self.probe_state != ProbeState::Running, super::controls::Button::new("接続を確認"));
                        record_control(ui, "qwen-probe-button", &check);
                        if check.clicked() {
                            if let Err(error) = self.start_probe(ui.ctx()) { self.error = Some(error); }
                        }
                        if self.probe_state == ProbeState::Running {
                            let cancel = ui.ww_button("確認を中止");
                            record_control(ui, "qwen-cancel-probe", &cancel);
                            if cancel.clicked() { self.cancel_probe(); }
                        }
                    });
                    match self.probe_state {
                        ProbeState::Untested => { ui.label("接続確認：未確認"); }
                        ProbeState::Running => { ui.horizontal(|ui| { ui.spinner(); ui.label("接続確認中…（最大30秒）"); }); }
                        ProbeState::Completed(Ok(success)) => { ui.label(success.message()); }
                        ProbeState::Completed(Err(error)) => { ui.label(error.message()); }
                    }
                    if let Some(error) = &self.error { ui.colored_label(egui::Color32::DARK_RED, format!("接続設定を確認してください：{error}")); }
                    ui.ww_collapsing("モデル・保存済み設定の詳細", |ui| {
                        ui.label("モデル：qwen3.8-omni-flash / 要求effort：medium（固定）");
                        ui.label("地域の選択は利用権や処理全体の所在地を保証しない。モデルとWorkspaceの提供範囲はAlibaba Cloudの管理画面で確認する。");
                        render_summary(ui, &self.summary());
                    });
            ui.separator();
            if !self.discard_requested {
                ui.horizontal_wrapped(|ui| {
                    let save = ui.ww_button("Qwen接続を保存");
                    record_control(ui, "qwen-save-button", &save);
                    if save.clicked() {
                        match self.save() { Ok(()) => close = true, Err(error) => self.error = Some(error) }
                    }
                    let cancel = ui.ww_button("Qwen編集をキャンセル");
                    record_control(ui, "qwen-cancel-edit", &cancel);
                    if cancel.clicked() { close = self.request_close(); }
                });
                ui.small("保存だけではAPIへ送信しない。一般設定や単独ツールとは別に保存する。");
            }
            #[cfg(debug_assertions)]
            if self.preview_tail {
                ui.scroll_to_cursor_animation(Some(egui::Align::BOTTOM), egui::style::ScrollAnimation::none());
                self.preview_tail = false;
                self.focus_pending = false;
            }
        });
        close
    }
}

#[cfg(debug_assertions)]
pub(super) fn synthetic_transport() -> Arc<dyn ProbeTransport> {
    struct SyntheticProbe;
    impl ProbeTransport for SyntheticProbe {
        fn send<'a>(&'a self, _: qwen_audio::ProbeRequest) -> qwen_audio::TransportFuture<'a> {
            Box::pin(async {
                Ok(qwen_audio::TransportResponse {
                    status: 200,
                    body: Box::pin(futures_util::stream::iter([Ok(
                        br#"{"success":true,"output":{"models":[{"model":"qwen3.8-omni-flash"}]}}"#
                            .to_vec(),
                    )])),
                })
            })
        }
    }
    Arc::new(SyntheticProbe)
}

#[cfg(debug_assertions)]
pub(super) fn synthetic_editor(
    store: Arc<dyn CredentialStore>,
    state: &str,
) -> QwenConnectionEditor {
    let mut editor = QwenConnectionEditor::new_with_transport(store, synthetic_transport())
        .expect("synthetic credentials");
    editor.probe_state = match state {
        "probe-running" => ProbeState::Running,
        "probe-success" => ProbeState::Completed(Ok(ProbeSuccess)),
        "probe-auth" => ProbeState::Completed(Err(ProbeError::Authentication)),
        "probe-permission" => ProbeState::Completed(Err(ProbeError::PermissionDenied)),
        "probe-unsupported" => ProbeState::Completed(Err(ProbeError::Unsupported)),
        "probe-missing" => ProbeState::Completed(Err(ProbeError::ModelNotFound)),
        "probe-cancelled" => ProbeState::Completed(Err(ProbeError::Cancelled)),
        "probe-timeout" => ProbeState::Completed(Err(ProbeError::Timeout)),
        "probe-invalid" => ProbeState::Completed(Err(ProbeError::ResponseInvalid)),
        "probe-network" => ProbeState::Completed(Err(ProbeError::Network)),
        "probe-rate-limit" => ProbeState::Completed(Err(ProbeError::RateLimited)),
        _ => ProbeState::Untested,
    };
    editor.probe_inputs = Some((editor.region, editor.host.clone(), editor.key.clone()));
    editor
}

impl Drop for QwenConnectionEditor {
    fn drop(&mut self) {
        self.invalidate_probe();
    }
}

fn record_control(ui: &egui::Ui, name: &'static str, response: &egui::Response) {
    #[cfg(test)]
    ui.ctx().data_mut(|data| {
        data.insert_temp(
            egui::Id::new(name),
            (response.rect, response.id, response.enabled()),
        )
    });
    #[cfg(not(test))]
    let _ = (ui, name, response);
}

fn render_summary(ui: &mut egui::Ui, summary: &Summary) {
    match summary {
        Ok(Some((region, host))) => {
            ui.label(format!(
                "保存済み：{} / APIキー保存済み（非表示）",
                region.label()
            ));
            ui.add(egui::Label::new(host).wrap().selectable(true));
        }
        Ok(None) => {
            ui.label("未設定：地域・API Host・APIキーを設定してください。");
        }
        Err(error) => {
            ui.colored_label(egui::Color32::DARK_RED, error.message());
        }
    }
}

impl WordApp {
    pub(super) fn qwen_connection_panel(&mut self, ui: &mut egui::Ui) {
        super::home_art::title(ui, "音読評価：Qwen（Alibaba Cloud）", 22.0);
        ui.label("「読んで発音を確認」だけで使用する。チャット・教材生成には使用しない。");
        ui.label("認証：専用APIキー。Windows資格情報へ保存する。Qwen専用の保存・キャンセルは、一般設定の保存・キャンセルとは別である。");
        if self.qwen_settings.is_some() {
            let input_fill = super::color_theme::blend(ux::TINT, self.effective_tint().input);
            if self
                .qwen_settings
                .as_mut()
                .is_some_and(|editor| editor.render(ui, input_fill))
            {
                self.finish_qwen_settings();
            }
            return;
        }
        if self.qwen_summary.is_none() {
            self.qwen_summary = Some(
                self.qwen_store
                    .load()
                    .map(|v| v.map(|c| (c.host().region(), c.host().as_str().to_owned()))),
            );
        }
        let response = ui.ww_button("Qwen接続設定を編集");
        if self.qwen_settings_focus && ui.is_enabled() && !ui.is_sizing_pass() {
            // Sizing passes may use an oversized clip, not the visible viewport.
            if ui.clip_rect().contains_rect(response.rect)
                && ui.ctx().screen_rect().contains_rect(response.rect)
                && !ui.ctx().will_discard()
            {
                response.request_focus();
                self.qwen_settings_focus = false;
            } else {
                response.scroll_to_me_animation(
                    Some(egui::Align::Center),
                    egui::style::ScrollAnimation::none(),
                );
            }
        }
        #[cfg(test)]
        ui.ctx().data_mut(|d| {
            d.insert_temp(egui::Id::new("qwen-open-settings"), response.rect);
            d.insert_temp(egui::Id::new("qwen-settings-button-id"), response.id);
            d.insert_temp(egui::Id::new("qwen-settings-button-clip"), ui.clip_rect());
        });
        if response.clicked() {
            let editor = match self.qwen_probe_transport.clone() {
                Some(transport) => {
                    QwenConnectionEditor::new_with_transport(self.qwen_store.clone(), transport)
                }
                None => QwenConnectionEditor::new(self.qwen_store.clone()),
            };
            match editor {
                Ok(editor) => self.qwen_settings = Some(editor),
                Err(error) => self.qwen_summary = Some(Err(error)),
            }
        }
        ui.small("編集を開くと、表示中の接続を確認できる。確認結果は保存しない。");
        render_summary(ui, self.qwen_summary.as_ref().expect("loaded"));
    }
    pub(super) fn finish_qwen_settings(&mut self) {
        if let Some(editor) = self.qwen_settings.take() {
            self.qwen_summary = Some(editor.summary());
            self.qwen_settings_focus = true;
            if editor.saved {
                self.message = "Qwenの音読評価用接続設定を保存した。APIへは送信していない。".into();
            }
        }
    }
    pub(super) fn go_to_qwen_settings(&mut self) {
        if let Some(mut dialog) = self.qwen_dialog.take() {
            dialog.close();
        }
        self.page = Page::Settings;
        self.settings_section = SettingsSection::Connection;
        self.codex_path_guidance = false;
        self.daily_limit_guidance = false;
        self.qwen_summary = None;
        self.qwen_settings_focus = true;
        self.begin_settings_edit();
        self.message = "AI接続の「音読評価：Qwen」で接続設定を編集してください。".into();
    }
}
