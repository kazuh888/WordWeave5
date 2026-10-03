//! Session-only reading dialog. A decoder may use a short-lived 3GP inspection copy.
//! Paid sends originate only from its send button.
use super::{
    controls::{Button, UiControls},
    WordApp,
};
use crate::media::{CapturedRecording, Recorder, Speaker};
use eframe::egui;
use qwen_audio::{
    AudioLoadJob, CredentialStore, ErrorCode, ReqwestTransport, SafeError, SendDisposition,
    Transport,
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use wordweave5::{
    model::Entry,
    qwen_reading::{
        PreparedId, PreparedPreview, ReadingController, ReadingPhase, ReadingTarget, ReadingView,
    },
};

pub(super) enum QwenAction {
    Cancel,
    Close,
    Send(PreparedId),
    Restart,
    GoToSettings,
    StartRecording,
    FinishRecording,
    CancelRecording,
    ChooseAudio,
    Play,
    StopPlayback,
}
pub(super) struct QwenDialog {
    pub(super) controller: ReadingController,
    pub(super) raw_audio: Option<Vec<u8>>,
    pub(super) notice: Option<String>,
    target_id: String,
    target_version: Vec<u8>,
    reference: String,
    recorder: Option<Recorder>,
    recording_started: Option<Instant>,
    speaker: Option<Speaker>,
    runtime: Option<tokio::runtime::Runtime>,
    pub(super) settings_requested: bool,
    credential_present: bool,
    audio_job: Option<AudioLoadJob>,
    audio_label: String,
    // Display-only receipt. Never used to authorize a send after the prepared snapshot expires.
    sent_preview: Option<PreparedPreview>,
    #[cfg(debug_assertions)]
    preview_tail: bool,
}
pub(super) fn recording_stop_due(elapsed: Duration, device_failed: bool) -> bool {
    elapsed >= Duration::from_secs(30) || device_failed
}
pub(super) fn original_audio_playing(snapshot: Option<&crate::media::PlaybackSnapshot>) -> bool {
    snapshot.is_some_and(|state| state.playing && !state.loading)
}
impl QwenDialog {
    pub(super) fn new(
        entry: &Entry,
        store: Arc<dyn CredentialStore>,
        transport: Arc<dyn Transport>,
    ) -> Result<Self, SafeError> {
        let reference = entry.completed();
        let version = serde_json::to_vec(entry)
            .map_err(|_| SafeError::new(ErrorCode::InvalidState, SendDisposition::NotSent))?;
        let target = ReadingTarget::new(entry.id.clone(), version.clone(), reference.clone())?;
        let mut controller = ReadingController::new(target, store, transport)?;
        let settings = controller.begin_settings()?;
        controller.cancel_settings();
        Ok(Self {
            controller,
            raw_audio: None,
            notice: None,
            target_id: entry.id.clone(),
            target_version: version,
            reference,
            recorder: None,
            recording_started: None,
            speaker: None,
            runtime: None,
            settings_requested: false,
            credential_present: settings.credential_present,
            audio_job: None,
            audio_label: String::new(),
            sent_preview: None,
            #[cfg(debug_assertions)]
            preview_tail: false,
        })
    }
    pub(super) fn check_target(&mut self, entry: Option<&Entry>) {
        let same = entry.is_some_and(|e| {
            e.id == self.target_id
                && e.completed() == self.reference
                && serde_json::to_vec(e).is_ok_and(|v| v == self.target_version)
        });
        if !same
            && !matches!(
                self.controller.view().phase,
                ReadingPhase::Invalidated | ReadingPhase::Closed
            )
        {
            self.controller.invalidate_target();
            self.stop_media();
            self.raw_audio = None;
            self.sent_preview = None;
            self.notice = Some("対象の教材が変更されました。閉じて教材から開き直してください。以前の対象では送信しません。".into());
        }
    }
    pub(super) fn accept_recording(&mut self, captured: CapturedRecording) {
        self.audio_label = "練習の録音.wav".into();
        let warning = captured.warning.is_some();
        match self
            .controller
            .set_audio("練習の録音.wav".into(), captured.wav.clone())
        {
            Ok(()) => self.notice = warning.then(|| {
                "録音機器の問題で途中までの音声を保存しました。この画面で原音を確認してください。"
                    .into()
            }),
            Err(e) => self.notice = Some(user_error(&e).into()),
        }
        self.raw_audio = Some(captured.wav);
    }
    pub(super) fn audio_loading(&self) -> bool {
        self.audio_job.is_some()
    }
    pub(super) fn select_audio(&mut self, path: Option<PathBuf>) {
        let Some(path) = path else {
            return;
        };
        if self.recorder.is_some() {
            return;
        }
        if let Err(e) = self.controller.clear_audio() {
            self.notice = Some(user_error(&e).into());
            return;
        }
        self.raw_audio = None;
        self.audio_job = None;
        if !self.stop_playback() {
            return;
        }
        self.audio_label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "選択した音声".into());
        match AudioLoadJob::start(path) {
            Ok(job) => {
                self.audio_job = Some(job);
                self.notice = None;
            }
            Err(e) => self.notice = Some(user_error(&e).into()),
        }
    }
    fn poll_audio(&mut self) {
        let Some(result) = self.audio_job.as_mut().and_then(|job| job.try_take()) else {
            return;
        };
        self.audio_job = None;
        if !matches!(
            self.controller.view().phase,
            ReadingPhase::Input | ReadingPhase::Confirming
        ) {
            return;
        }
        match result {
            Ok(audio) => {
                let playback = audio.playback_wav().to_vec();
                match self
                    .controller
                    .set_validated_audio(self.audio_label.clone(), audio)
                {
                    Ok(()) => {
                        self.raw_audio = Some(playback);
                        self.notice = None;
                    }
                    Err(e) => self.notice = Some(user_error(&e).into()),
                }
            }
            Err(e) => self.notice = Some(user_error(&e).into()),
        }
    }
    fn stop_playback(&mut self) -> bool {
        if let Some(mut speaker) = self.speaker.take() {
            if speaker.stop().is_err() {
                self.notice =
                    Some("原音の再生を停止できません。音声機能を確認してください。".into());
                return false;
            }
        }
        true
    }
    fn stop_media(&mut self) {
        self.audio_job = None;
        if let Some(recorder) = self.recorder.take() {
            recorder.cancel();
        }
        self.recording_started = None;
        self.stop_playback();
    }
    pub(super) fn close(&mut self) {
        self.controller.close();
        self.stop_media();
        self.raw_audio = None;
        self.sent_preview = None;
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
    fn finish_recording(&mut self) {
        self.recording_started = None;
        if let Some(recorder) = self.recorder.take() {
            match recorder.finish_with_warning() {
                Ok(captured) => self.accept_recording(captured),
                Err(_) => {
                    self.raw_audio = None;
                    let _ = self.controller.clear_audio();
                    self.notice = Some(
                        "録音を取得できません。マイクと入力デバイスを確認してください。".into(),
                    );
                }
            }
        }
    }
    pub(super) fn dispatch_frame(&mut self, actions: &[QwenAction], ctx: &egui::Context) -> bool {
        // Cancellation wins even if a send or ready completion occurs in this same frame.
        if actions.iter().any(|a| matches!(a, QwenAction::Close)) {
            self.close();
            return true;
        }
        if actions.iter().any(|a| matches!(a, QwenAction::Cancel)) {
            self.controller.cancel();
            self.stop_media();
            self.controller.poll();
            return false;
        }
        for action in actions {
            let result = match action {
                QwenAction::Send(id) => {
                    if self.recorder.is_some() || self.audio_loading() || !self.stop_playback() {
                        continue;
                    }
                    if self.runtime.is_none() {
                        match tokio::runtime::Builder::new_multi_thread()
                            .worker_threads(2)
                            .enable_all()
                            .build()
                        {
                            Ok(runtime) => self.runtime = Some(runtime),
                            Err(_) => {
                                self.notice = Some("評価処理を開始できません。閉じて開き直してください。送信していません。".into());
                                continue;
                            }
                        }
                    }
                    let preview = self.controller.view().preview;
                    self.controller
                        .human_send(*id, self.runtime.as_ref().unwrap().handle())
                        .map(|()| {
                            self.sent_preview = preview;
                            self.notice = None;
                        })
                }
                QwenAction::Restart => self.controller.restart().map(|()| {
                    self.sent_preview = None;
                    self.notice = None;
                }),
                QwenAction::GoToSettings => {
                    let view = self.controller.view();
                    if (!self.credential_present
                        || view.error.as_ref().is_some_and(connection_error))
                        && self.recorder.is_none()
                        && !self.audio_loading()
                        && matches!(
                            view.phase,
                            ReadingPhase::Input
                                | ReadingPhase::Confirming
                                | ReadingPhase::Failed
                                | ReadingPhase::Cancelled
                        )
                    {
                        self.settings_requested = true;
                        self.close();
                        return true;
                    }
                    Ok(())
                }
                QwenAction::StartRecording => {
                    if self.audio_loading() {
                        continue;
                    }
                    if !self.stop_playback() {
                        continue;
                    }
                    if let Err(e) = self.controller.clear_audio() {
                        self.notice = Some(user_error(&e).into());
                        continue;
                    }
                    self.raw_audio = None;
                    match Recorder::start() { Ok(r) => { self.recorder = Some(r); self.recording_started = Some(Instant::now()); self.notice = None; },
                        Err(_) => self.notice = Some("録音を開始できません。Windowsのマイク許可と入力デバイスを確認してください。".into()) }
                    Ok(())
                }
                QwenAction::FinishRecording => {
                    self.finish_recording();
                    Ok(())
                }
                QwenAction::CancelRecording => {
                    if let Some(r) = self.recorder.take() {
                        r.cancel();
                    }
                    self.recording_started = None;
                    self.raw_audio = None;
                    self.controller.clear_audio()
                }
                QwenAction::ChooseAudio => {
                    let path = rfd::FileDialog::new()
                        .set_title("例文を読んだ音声を選択")
                        .add_filter("音声ファイル", &["wav", "mp3", "aac", "amr", "3gp", "3gpp"])
                        .pick_file();
                    self.select_audio(path);
                    Ok(())
                }
                QwenAction::Play => {
                    if self.audio_loading() {
                        continue;
                    }
                    if let Some(raw) = self.raw_audio.as_ref() {
                        let speaker = self.speaker.get_or_insert_with(Speaker::new);
                        if speaker.play_wav(raw, 1.0).is_err() {
                            self.notice = Some(
                                "原音を再生できません。音声形式と出力デバイスを確認してください。"
                                    .into(),
                            );
                            self.speaker = None;
                        }
                    }
                    Ok(())
                }
                QwenAction::StopPlayback => {
                    self.stop_playback();
                    Ok(())
                }
                QwenAction::Cancel | QwenAction::Close => unreachable!(),
            };
            if let Err(e) = result {
                self.notice = Some(user_error(&e).into());
            }
        }
        self.poll_audio();
        self.controller.poll();
        ctx.request_repaint_after(Duration::from_millis(100));
        false
    }
    #[cfg(test)]
    pub(super) fn sent_preview_for_test(&self) -> Option<&PreparedPreview> {
        self.sent_preview.as_ref()
    }
    fn preparation_ui(
        &mut self,
        ui: &mut egui::Ui,
        view: &ReadingView,
        actions: &mut Vec<QwenAction>,
        playing: bool,
    ) {
        let recording = self.recorder.is_some();
        let loading = self.audio_loading();
        let editable = matches!(view.phase, ReadingPhase::Input | ReadingPhase::Confirming);
        ui.strong("読む例文（この教材の主な例文）");
        ui.add(egui::Label::new(&self.reference).wrap().selectable(true));
        if editable {
            body_text(ui, "例文を録音するか、対応する音声を選んでください。音声の検査が完了するまで送信できません。");
        }
        if let Some(r) = self.recorder.as_ref() {
            ui.strong(format!(
                "録音中：{:.1}秒 / 最大30秒",
                r.elapsed().as_secs_f64()
            ));
            body_text(ui, "例文を読み、「録音を終了」で原音を確認してください。この録音だけを捨てる場合は「この録音を破棄」を選んでください。");
        }
        ui.horizontal_wrapped(|ui| {
            if editable
                && control(
                    ui,
                    if recording {
                        "録音を終了"
                    } else {
                        "録音を開始（最大30秒）"
                    },
                    "qwen-record",
                    !loading,
                )
            {
                actions.push(if recording {
                    QwenAction::FinishRecording
                } else {
                    QwenAction::StartRecording
                });
            }
            if recording {
                if control(ui, "この録音を破棄", "qwen-discard-recording", true) {
                    actions.push(QwenAction::CancelRecording);
                }
            } else {
                if editable && control(ui, "音声を選ぶ", "qwen-select-wav", !loading) {
                    actions.push(QwenAction::ChooseAudio);
                }
                if control(
                    ui,
                    "原音を聞く",
                    "qwen-play",
                    self.raw_audio.is_some() && !loading,
                ) {
                    actions.push(QwenAction::Play);
                }
            }
            if super::playback_panel::stop_button(ui, "qwen-stop", playing) {
                actions.push(QwenAction::StopPlayback);
            }
        });
        if let Some(info) = &view.audio_info {
            ui.strong(format!(
                "音声：{} / {:.1}秒",
                self.audio_label, info.duration_seconds
            ));
        }
        if loading {
            ui.spinner();
            ui.strong("音声を確認中");
            body_text(ui, "音声の形式と長さを確認しています。中止する場合は「取消し」または「閉じる」を選んでください。");
            if control(ui, "取消し", "qwen-cancel", true) {
                actions.push(QwenAction::Cancel);
            }
        }
        let details = ui.ww_collapsing("対応形式・音声の詳細", |ui| {
            body_text(ui, "WAV（PCM16）・MP3・AAC（ADTS）・AMR・3GP・3GPP（音声のみ）。1・2チャンネル / 8〜48kHz / 60秒以下 / 6MiB以下。録音は最大30秒です。");
            if let Some(info) = &view.audio_info {
                ui.label(format!("選択済み：{} / {}Hz / {}ch / {} bytes", view.audio_format.unwrap_or("").to_ascii_uppercase(), info.sample_rate, info.channels, info.byte_len));
            }
        });
        record_rect(ui.ctx(), "qwen-audio-details", details.header_response.rect);
        if editable {
            if let Some(error) = view.error.as_ref().filter(|e| !connection_error(e)) {
                render_error(
                    ui,
                    error,
                    view.phase,
                    self.raw_audio.is_some() && !recording && !loading,
                );
            }
            if let Some(notice) = &self.notice {
                body_text(ui, notice);
            }
        }
    }
    fn confirmation_ui(
        &mut self,
        ui: &mut egui::Ui,
        view: &ReadingView,
        actions: &mut Vec<QwenAction>,
    ) {
        let editable = matches!(view.phase, ReadingPhase::Input | ReadingPhase::Confirming);
        if !self.credential_present || view.error.as_ref().is_some_and(connection_error) {
            ui.strong("接続が未設定、または接続設定の確認が必要です");
            body_text(ui, "「設定 → AI接続」のQwen欄で設定を確認してください。移動すると、この画面の音声と結果を破棄します。");
            if let Some(error) = view.error.as_ref().filter(|e| connection_error(e)) {
                render_error(ui, error, view.phase, false);
            }
            if self.recorder.is_none()
                && !self.audio_loading()
                && matches!(
                    view.phase,
                    ReadingPhase::Input
                        | ReadingPhase::Confirming
                        | ReadingPhase::Failed
                        | ReadingPhase::Cancelled
                )
                && control(ui, "設定のAI接続を開く", "qwen-settings", true)
            {
                actions.push(QwenAction::GoToSettings);
            }
        }
        let preview = if editable {
            view.preview.as_ref()
        } else {
            self.sent_preview.as_ref()
        };
        if let Some(p) = preview {
            ui.strong(if editable {
                "送信前の確認"
            } else {
                "送信時に確認した内容（閲覧用）"
            });
            ui.strong(format!(
                "音声：{} / {:.1}秒",
                p.audio_label, p.audio_info.duration_seconds
            ));
            ui.add(
                egui::Label::new(format!("送信する英文：{}", p.reference))
                    .wrap()
                    .selectable(true),
            );
            ui.ww_collapsing("送信音声の詳細", |ui| {
                ui.label(format!(
                    "音声の詳細：{} / {}Hz / {}ch / {} bytes",
                    p.audio_format.to_ascii_uppercase(),
                    p.audio_info.sample_rate,
                    p.audio_info.channels,
                    p.audio_info.byte_len
                ));
            });
            if editable {
                body_text(ui, "音声と英文を確認し、送信ボタンを押してください。確認しただけでは送信しません。");
                if control(
                    ui,
                    "確認した内容を送信して評価",
                    "qwen-send",
                    self.recorder.is_none() && !self.audio_loading(),
                ) {
                    actions.push(QwenAction::Send(p.id));
                }
            }
        } else if editable {
            body_text(ui, "①で音声を準備してください。検査済みの音声と有効な接続設定が揃うと、ここに送信内容を表示します。");
        } else {
            ui.label("送信時の確認内容はありません。");
        }
    }
    fn evaluation_ui(
        &mut self,
        ui: &mut egui::Ui,
        view: &ReadingView,
        actions: &mut Vec<QwenAction>,
    ) {
        match view.phase {
            ReadingPhase::Input | ReadingPhase::Confirming => {
                ui.label("②で送信すると、ここに評価結果を表示します。");
            }
            ReadingPhase::Running => {
                ui.spinner();
                ui.strong("音声を評価中");
                body_text(ui, "最大180秒待機します。自動再送はしません。中止する場合は「取消し」または「閉じる」を選んでください。");
                if control(ui, "取消し", "qwen-cancel", true) {
                    actions.push(QwenAction::Cancel);
                }
            }
            ReadingPhase::Completed => {
                if let Some(result) = &view.result {
                    render_result(ui, result, &self.reference);
                }
            }
            ReadingPhase::Failed => {
                ui.strong("評価を完了できませんでした");
            }
            ReadingPhase::Cancelled => {
                ui.strong("処理を取り消しました");
                body_text(ui, "「もう一度練習」から音声と例文を確認し直してください。");
            }
            ReadingPhase::Invalidated => {
                ui.strong("対象の教材が変更されました");
                body_text(
                    ui,
                    "閉じて教材から開き直してください。以前の対象では送信しません。",
                );
            }
            ReadingPhase::Closed => {}
        }
        if matches!(
            view.phase,
            ReadingPhase::Cancelled | ReadingPhase::Failed | ReadingPhase::Invalidated
        ) {
            let text = if view.disposition == SendDisposition::NotSent {
                "送信していません。"
            } else {
                "送信済みの可能性があります。遠隔処理の完了と課金は不明です。自動再送はしません。"
            };
            body_text(ui, text);
            #[cfg(test)]
            ui.ctx().data_mut(|data| {
                data.insert_temp(egui::Id::new("qwen-disposition-text"), text.to_owned())
            });
        }
        if !matches!(view.phase, ReadingPhase::Input | ReadingPhase::Confirming) {
            if let Some(error) = &view.error {
                render_error(ui, error, view.phase, self.raw_audio.is_some());
            }
            if let Some(notice) = &self.notice {
                body_text(ui, notice);
            }
        }
        if matches!(
            view.phase,
            ReadingPhase::Cancelled | ReadingPhase::Failed | ReadingPhase::Completed
        ) && control(ui, "もう一度練習", "qwen-restart", true)
        {
            actions.push(QwenAction::Restart);
        }
    }
    pub(super) fn render(&mut self, ctx: &egui::Context) -> bool {
        #[cfg(test)]
        ctx.data_mut(|data| data.remove::<String>(egui::Id::new("qwen-disposition-text")));
        if self.recorder.as_ref().is_some_and(|r| {
            recording_stop_due(
                self.recording_started
                    .map(|t| t.elapsed())
                    .unwrap_or_else(|| r.elapsed()),
                r.error().is_some(),
            )
        }) {
            self.finish_recording();
        }
        let mut playing = false;
        if let Some(speaker) = self.speaker.as_mut() {
            match speaker.snapshot() {
                Ok(snapshot) => playing = original_audio_playing(Some(&snapshot)),
                Err(_) => {
                    self.notice = Some(
                        "原音の再生中に問題が発生しました。出力デバイスを確認してください。".into(),
                    );
                    self.speaker = None;
                }
            }
        }
        // Preparing a local confirmation is not consent to send. Keep its ID stable on redraw.
        let view = self.controller.view();
        if view.phase == ReadingPhase::Input
            && view.audio_info.is_some()
            && view.error.is_none()
            && self.credential_present
            && self.recorder.is_none()
            && !self.audio_loading()
        {
            if let Err(error) = self.controller.prepare() {
                self.notice = Some(user_error(&error).into());
            }
        }
        let mut actions = Vec::new();
        let screen = ctx.screen_rect();
        let response = egui::Modal::new(egui::Id::new("qwen-reading"))
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                // eframe can replace Context's initial style after creation. Set dialog typography locally.
                for (kind, size) in [(egui::TextStyle::Body, 19.0), (egui::TextStyle::Small, 16.0),
                    (egui::TextStyle::Button, 18.0), (egui::TextStyle::Heading, 24.0)] {
                    if let Some(font) = ui.style_mut().text_styles.get_mut(&kind) { font.size = size; }
                }
                ui.set_width((screen.width() - 48.0).clamp(120.0, 760.0));
                // Area retains its previous/default height. Let the scroll body use the
                // current viewport, rather than keeping an unnecessary ~600px cap.
                ui.set_max_height((screen.height() - 48.0).max(0.0));
                ui.spacing_mut().item_spacing.y = 8.0;
                let content_top = ui.cursor().top();
                ui.heading("例文を読んで発音を確認");
                ui.label("① 音声の準備 → ② 送信内容の確認 → ③ 評価結果");
                let view = self.controller.view();
                let editable = matches!(view.phase, ReadingPhase::Input | ReadingPhase::Confirming);
                ui.strong(if editable {
                    if self.recorder.is_some() || self.audio_loading() || view.audio_info.is_none() {
                        "現在：① 音声を準備する"
                    } else { "現在：② 送信内容を確認する" }
                } else { "現在：③ 評価結果を確認する" });
                // Use the actual wrapped footer text metrics at this width and font size.
                // The footer remains outside the scroll area even at enlarged/narrow sizes.
                let footer_note = "この画面を閉じると録音と結果を破棄します。\n教材・学習記録・チャットには保存しません。";
                let footer_text_height = ui.fonts(|fonts| fonts.layout(
                    footer_note.into(), egui::TextStyle::Body.resolve(ui.style()),
                    ui.visuals().text_color(), ui.available_width(),
                ).size().y);
                let footer_height = 40.0 + footer_text_height + 3.0 * ui.spacing().item_spacing.y + 6.0;
                let header_height = ui.cursor().top() - content_top;
                let scroll = egui::ScrollArea::vertical().id_salt("qwen-reading-body")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);
                #[cfg(debug_assertions)]
                let scroll = if self.preview_tail { scroll.stick_to_bottom(true) } else { scroll };
                let body = scroll
                    .max_height((screen.height() - 48.0 - header_height - footer_height).max(0.0)).min_scrolled_height(0.0)
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width());
                        body_text(ui, "音声と例文をAlibaba Cloudへ送り、発音・強弱・リズムの改善点を確認します。採点や成績変更はしません。");
                        body_text(ui, "※送信後の取り消しは、遠隔処理の停止・課金取り消しを保証しません。");
                        if editable {
                            step(ui, "① 音声を準備する", |ui| self.preparation_ui(ui, &view, &mut actions, playing));
                            step(ui, "② 送信内容を確認する", |ui| self.confirmation_ui(ui, &view, &mut actions));
                        } else {
                            let audio = self.sent_preview.as_ref().map(|p| p.audio_label.clone()).unwrap_or_else(|| self.audio_label.clone());
                            let short_audio = if audio.chars().count() > 32 { format!("{}…", audio.chars().take(32).collect::<String>()) } else { audio };
                            // New sends and terminal transitions start with closed summaries.
                            // A previous result's expanded state must not hide the next result.
                            let receipt_id = self.sent_preview.as_ref().map(|p| p.id);
                            ui.push_id(format!("{receipt_id:?}-{:?}", view.phase), |ui| {
                                let first = ui.ww_collapsing("① 音声を準備する", |ui| self.preparation_ui(ui, &view, &mut actions, playing));
                                record_rect(ctx, "qwen-preparation-summary", first.header_response.rect);
                                #[cfg(test)]
                                ctx.data_mut(|data| data.insert_temp(egui::Id::new("qwen-preparation-summary-widget-id"), first.header_response.id));
                                ui.add(egui::Label::new(format!("音声：{}", if short_audio.is_empty() { "音声なし" } else { &short_audio })).wrap());
                                let second = ui.ww_collapsing("② 送信内容を確認する", |ui| self.confirmation_ui(ui, &view, &mut actions));
                                record_rect(ctx, "qwen-confirmation-summary", second.header_response.rect);
                                #[cfg(test)]
                                ctx.data_mut(|data| data.insert_temp(egui::Id::new("qwen-confirmation-summary-widget-id"), second.header_response.id));
                            });
                        }
                        step(ui, "③ 評価結果を確認する", |ui| self.evaluation_ui(ui, &view, &mut actions));
                        #[cfg(debug_assertions)]
                        if self.preview_tail { ui.scroll_to_cursor(Some(egui::Align::Max)); }
                    });
                record_rect(ctx, "qwen-body", body.inner_rect);
                let footer = ui.scope(|ui| {
                    ui.separator();
                    if control(ui, "閉じる", "qwen-close", true) { actions.push(QwenAction::Close); }
                    ui.label(footer_note);
                });
                record_rect(ctx, "qwen-footer", footer.response.rect);
            });
        if response.should_close() {
            actions.push(QwenAction::Close);
        }
        self.dispatch_frame(&actions, ctx)
    }
}
#[cfg(debug_assertions)]
#[path = "../../tools/qwen-audio/tests/fixtures/reuse/mod.rs"]
pub(super) mod preview_data;
#[cfg(debug_assertions)]
pub(super) fn synthetic_dialog(entry: &Entry, state: &str, tail: bool) -> QwenDialog {
    use preview_data::{audio, Reply, Store, Wire, ASSESSED};
    // This hook exclusively constructs a fake store and fake transport. No OS credentials or HTTP.
    let reply = match state {
        "result" => Reply::Feedback(ASSESSED),
        "mismatch" => Reply::Feedback(
            r#"{"assessment":"reference_mismatch","heard_text":"Do you think we need more time?","summary":"読む予定の例文とは別の英文が聞き取れた。","strengths":[],"improvements":[],"unassessable_reason":null}"#,
        ),
        "unassessable" => Reply::Feedback(
            r#"{"assessment":"unassessable","heard_text":null,"summary":"この音声は評価できない。","strengths":[],"improvements":[],"unassessable_reason":"声を十分に聞き取れなかった。"}"#,
        ),
        "jsonfailed" => Reply::Feedback("{"),
        "fieldsfailed" => Reply::Feedback(r#"{"assessment":"assessed"}"#),
        "responsefailed" => Reply::Feedback(""),
        "failed" => Reply::Status(500),
        _ => Reply::Pending,
    };
    let store = if state == "unconfigured" {
        Store::new(None)
    } else {
        Store::configured()
    };
    let mut dialog = QwenDialog::new(entry, store, Wire::new(reply)).expect("synthetic target");
    dialog.preview_tail = tail;
    if !["input", "settings", "unconfigured"].contains(&state) {
        dialog.accept_recording(CapturedRecording {
            wav: audio(),
            warning: None,
        });
        let preview = dialog.controller.prepare().expect("synthetic preparation");
        if matches!(
            state,
            "running"
                | "result"
                | "failed"
                | "mismatch"
                | "unassessable"
                | "jsonfailed"
                | "fieldsfailed"
                | "responsefailed"
        ) {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap();
            dialog
                .controller
                .human_send(preview.id, runtime.handle())
                .unwrap();
            dialog.sent_preview = Some(preview);
            dialog.runtime = Some(runtime);
        }
    }
    dialog
}
impl Drop for QwenDialog {
    fn drop(&mut self) {
        self.close();
    }
}
fn count(value: Option<u64>) -> String {
    value
        .map(|n| n.to_string())
        .unwrap_or_else(|| "未取得".into())
}
fn step(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(8.0);
    egui::Frame::group(ui.style())
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_max_width(ui.available_width());
                ui.strong(title);
                content(ui);
            });
        });
}
fn body_text(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(text.replace('。', "。\n").trim_end()).wrap());
}
// Presentation text is separate from SafeError's stable serialized protocol messages.
fn user_error(error: &SafeError) -> &'static str {
    use ErrorCode::*;
    match error.code() {
        InvalidReference => "参照英文は空白以外を含む10,000文字以下で入力してください。",
        AudioIo => "音声ファイルを読み取れません。",
        AudioTooLarge => "音声ファイルは6MiB以下にしてください。",
        AudioEmpty => "音声が空です。",
        AudioCorrupt => "音声の構造が壊れている、または途中で形式が変化しています。",
        AudioUnsupported => "対応するWAV・MP3・AAC・AMR・3GP/3GPP音声（1/2チャンネル・8～48kHz）を選択してください。",
        AudioTooLong => "音声は60秒以下にしてください。",
        AudioBackendUnavailable => "圧縮音声の検査・再生に必要なFFmpeg/ffprobeを利用できません。WAVは利用できます。",
        AudioDecodeTimeout => "音声の検査・復号が15秒以内に完了しませんでした。",
        AudioDecodeFailed => "音声の検査・復号に失敗しました。対応する形式・codecを確認してください。",
        AudioCleanup => "検査用の一時音声コピーを削除できませんでした。コピーが残っている可能性があります。",
        InvalidHost => "選択した地域とWorkspace API HostのHTTPS URL形式を確認してください。",
        InvalidKey => "APIキーの形式を確認してください。接続先を変更する場合は新たにキーを入力してください。",
        CredentialRead => "保存済み接続設定を読み取れません。",
        CredentialWrite => "接続設定を保存できません。旧設定を保持しました。",
        Busy => "進行中の評価があります。完了するまでお待ちください。",
        SessionNotFound => "セッションが見つかりません。",
        InvalidState => "現在の状態では操作できません。",
        GuiLaunch => "確認画面を起動できません。",
        GuiDisconnected => "確認画面との接続が切れました。",
        Authentication => "認証に失敗しました。接続設定を確認してください。",
        RateLimited => "APIの利用制限に達しました。",
        Provider => "APIが要求を受理できませんでした。",
        Network => "通信に失敗しました。",
        Timeout => "評価が180秒以内に完了しませんでした。",
        ResponseTooLarge => "API応答が上限を超えました。",
        ResponseInvalid | ResponseJsonInvalid | ResponseFieldsInvalid => "AIの評価結果を読み取れませんでした。音声の良し悪しや例文との一致は判断できません。",
        ProtocolInvalid => "通信形式を確認できません。",
        Cancelled => "評価を取り消しました。",
    }
}
fn record_rect(ctx: &egui::Context, id: &'static str, rect: egui::Rect) {
    #[cfg(test)]
    ctx.data_mut(|data| data.insert_temp(egui::Id::new(id), rect));
    #[cfg(not(test))]
    let _ = (ctx, id, rect);
}
fn connection_error(error: &SafeError) -> bool {
    matches!(
        error.code(),
        ErrorCode::Authentication
            | ErrorCode::InvalidHost
            | ErrorCode::InvalidKey
            | ErrorCode::CredentialRead
    )
}
fn render_error(ui: &mut egui::Ui, error: &SafeError, phase: ReadingPhase, audio_available: bool) {
    egui::Frame::group(ui.style()).inner_margin(12.0).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.set_max_width(ui.available_width());
            let unreadable = matches!(error.code(), ErrorCode::ResponseInvalid | ErrorCode::ResponseJsonInvalid | ErrorCode::ResponseFieldsInvalid | ErrorCode::ProtocolInvalid);
            if unreadable {
                ui.strong("AIの回答を評価結果として読み取れませんでした");
                body_text(ui, "発音が悪いという意味ではありません。音声の良し悪しや例文との一致は判断できません。");
                body_text(ui, match error.code() {
                    ErrorCode::ResponseJsonInvalid => "確認できなかった段階：結果のJSON形式。",
                    ErrorCode::ResponseFieldsInvalid => "確認できなかった段階：結果の必須項目や内容の整合。",
                    ErrorCode::ProtocolInvalid => "確認できなかった段階：通信形式。",
                    _ => "確認できなかった段階：特定できません。原因は不明です。",
                });
            } else {
                ui.strong(user_error(error));
            }
            match phase {
                ReadingPhase::Invalidated => {
                    body_text(ui, "閉じて教材から開き直してください。以前の対象では送信しません。");
                }
                ReadingPhase::Input | ReadingPhase::Confirming if connection_error(error) => {
                    body_text(ui, "「設定のAI接続を開く」からQwen欄を確認してください。");
                }
                ReadingPhase::Input | ReadingPhase::Confirming => {
                    if audio_available { ui.label("①の「原音を聞く」で音声を確認できます。"); }
                    body_text(ui, "対応形式・音声の詳細を確認し、例文を録音するか対応する音声を選んでください。送信には改めて内容の確認が必要です。");
                }
                ReadingPhase::Failed | ReadingPhase::Cancelled | ReadingPhase::Completed => {
                    if audio_available { ui.label("①を開くと「原音を聞く」で音声を確認できます。"); }
                    body_text(ui, "「もう一度練習」から音声と例文を確認し直してください。再送には改めて送信内容の確認が必要です。");
                }
                ReadingPhase::Running => {
                    body_text(ui, "処理を中止する場合は「取消し」または「閉じる」を選んでください。");
                }
                ReadingPhase::Closed => {}
            }
            let details = ui.ww_collapsing("エラーの詳細", |ui| {
                let code = serde_json::to_string(&error.code()).unwrap_or_default();
                ui.label(format!("エラーコード：{}", code.trim_matches('"')));
            });
            record_rect(ui.ctx(), "qwen-error-details", details.header_response.rect);
        });
    });
}
fn render_result(ui: &mut egui::Ui, result: &qwen_audio::EvaluationResult, reference: &str) {
    use qwen_audio::Assessment;
    ui.strong(match result.feedback.assessment {
        Assessment::Assessed => "評価結果",
        Assessment::ReferenceMismatch => "例文と異なる英文が聞き取れた",
        Assessment::Unassessable => "この音声は評価できません",
    });
    ui.label(&result.feedback.summary);
    if result.feedback.assessment == Assessment::ReferenceMismatch {
        ui.strong("読む例文");
        ui.add(egui::Label::new(reference).wrap().selectable(true));
        body_text(ui, "この例文の発音評価は行っていません。①を開いて原音を確認し、「もう一度練習」から表示された例文を録り直すか、対応する音声を選んでください。聞き取りはAIの推定です。");
    } else if result.feedback.assessment == Assessment::Unassessable {
        body_text(
            ui,
            "①を開いて原音を確認し、「もう一度練習」から録り直すか、別の音声を選んでください。",
        );
    } else {
        body_text(
            ui,
            "改善点を確認し、「もう一度練習」から練習を続けてください。",
        );
    }
    if let Some(reason) = &result.feedback.unassessable_reason {
        ui.label(reason);
    }
    if let Some(heard) = &result.feedback.heard_text {
        ui.strong("AIが聞き取った英文（推定）");
        ui.add(egui::Label::new(heard).wrap().selectable(true));
    }
    if !result.feedback.strengths.is_empty() {
        ui.strong("良かった点");
        for text in &result.feedback.strengths {
            ui.label(format!("・{text}"));
        }
    }
    for item in &result.feedback.improvements {
        ui.add_space(8.0);
        ui.strong(format!("練習する箇所：{}", item.reference_excerpt));
        ui.label(&item.observation);
        ui.label(format!("練習方法：{}", item.practice));
    }
    body_text(
        ui,
        "AIの助言であり、発音採点の正確さを保証するものではありません。",
    );
    let details = ui.ww_collapsing("応答モデル・使用量", |ui| {
        ui.label(format!(
            "応答モデル：{} / 応答effort：{}",
            result.actual_model.as_deref().unwrap_or("未取得"),
            result.actual_effort.as_deref().unwrap_or("未取得")
        ));
        if let Some(usage) = &result.usage {
            ui.label(format!(
                "応答使用量：入力 {} / 出力 {} / 合計 {} tokens",
                count(usage.prompt_tokens),
                count(usage.completion_tokens),
                count(usage.total_tokens)
            ));
        } else {
            body_text(ui, "応答使用量：未取得。費用は推測しません。");
        }
    });
    record_rect(
        ui.ctx(),
        "qwen-response-details",
        details.header_response.rect,
    );
}
fn control(ui: &mut egui::Ui, text: &str, id: &'static str, enabled: bool) -> bool {
    let response = ui.add_enabled(
        enabled,
        Button::new(text).min_size(egui::vec2(90.0, 40.0)).wrap(),
    );
    #[cfg(test)]
    ui.ctx().data_mut(|data| {
        data.insert_temp(egui::Id::new(id), response.rect);
        data.insert_temp(egui::Id::new(format!("{id}-widget-id")), response.id);
    });
    #[cfg(not(test))]
    let _ = id;
    response.clicked()
}
impl WordApp {
    pub(super) fn open_qwen_reading(&mut self, entry: &Entry) {
        if self.qwen_dialog.is_some() {
            return;
        }
        if self.recorder.is_some()
            || self.pending.is_some()
            || self.batch_running
            || self.session.is_some()
            || self.chat_material_open
            || self.chat_media_open
            || self.chat_context_open
            || self.pending_import.is_some()
            || self.pending_restore.is_some()
            || self.backup_restore.is_some()
            || self.color_editor.is_some()
            || self.settings_changed()
            || self.unsaved_chat_audio.is_some()
            || self.annotation.frozen()
            || !self.annotation.text.is_empty()
            || self.media_preview.is_some()
            || self.file_preview.is_some()
        {
            self.notify_blocked(
                "他の録音・処理・未保存の入力を完了してから音声評価を開いてください。",
            );
            return;
        }
        if self.speaker.stop().is_err() {
            self.notify_error("読み上げを停止できないため、音声評価を開けません。".into());
            return;
        }
        self.speech_visible = false;
        let result = ReqwestTransport::new().and_then(|transport| {
            QwenDialog::new(entry, self.qwen_store.clone(), Arc::new(transport))
        });
        match result {
            Ok(dialog) => self.qwen_dialog = Some(dialog),
            Err(e) => self.notify_error(user_error(&e).into()),
        }
    }
    pub(super) fn qwen_reading_window(&mut self, ctx: &egui::Context) {
        // A closing viewport cannot reliably present a new notification. Leave the flag for main.
        if !ctx.input(|i| i.viewport().close_requested())
            && qwen_audio::take_audio_cleanup_warning()
        {
            self.notify_error(
                "検査用の一時音声コピーを削除できませんでした。元の音声ファイルは変更していません。"
                    .into(),
            );
        }
        let mut navigate = false;
        if let Some(dialog) = self.qwen_dialog.as_mut() {
            let entry = self.deck.iter().find(|e| e.id == dialog.target_id);
            dialog.check_target(entry);
            if dialog.render(ctx) {
                navigate = dialog.settings_requested;
                self.qwen_dialog = None;
            }
        }
        if navigate {
            self.go_to_qwen_settings();
        }
    }
}
