#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use eframe::egui;
use qwen_audio::gui::*;
use qwen_audio::*;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use zeroize::{Zeroize, Zeroizing};

struct ChildPipe {
    tx: mpsc::Sender<ChildMessage>,
    rx: mpsc::Receiver<ParentMessage>,
    id: SessionId,
    waiting: Option<Instant>,
    closing: bool,
    can_close: bool,
}
struct NativeApp {
    controller: Option<GuiController>,
    fixture: Option<GuiModel>,
    runtime: tokio::runtime::Runtime,
    pipe: Option<ChildPipe>,
    audio_load: Option<AudioLoadJob>,
    audio_load_label: String,
    settings: bool,
    settings_host: String,
    settings_key: Zeroizing<String>,
    capture: Option<PathBuf>,
    capture_requested: bool,
    started: Instant,
    close_after: Option<Duration>,
}
impl NativeApp {
    fn cancel_audio_load(&mut self) {
        self.audio_load = None;
        self.audio_load_label.clear();
    }
    fn start_audio_load(&mut self, path: PathBuf) {
        let Some(controller) = self.controller.as_mut() else {
            return;
        };
        if let Err(error) = controller.clear_audio() {
            self.report(error);
            return;
        }
        self.cancel_audio_load();
        let label = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        match AudioLoadJob::start(path) {
            Ok(job) => {
                self.audio_load = Some(job);
                self.audio_load_label = label;
            }
            Err(error) => self.report(error),
        }
    }
    fn poll_audio_load(&mut self) {
        let editable = self
            .controller
            .as_ref()
            .is_some_and(|c| matches!(c.model().phase, GuiPhase::AwaitingUser));
        let closing = self
            .pipe
            .as_ref()
            .is_some_and(|p| p.closing || p.can_close || p.waiting.is_some());
        if !editable || closing {
            self.cancel_audio_load();
            return;
        }
        let result = self.audio_load.as_mut().and_then(AudioLoadJob::try_take);
        if let Some(result) = result {
            self.audio_load = None;
            let label = std::mem::take(&mut self.audio_load_label);
            match result {
                Ok(audio) => {
                    if let Some(controller) = self.controller.as_mut() {
                        if let Err(error) = controller.select_audio(audio, label) {
                            self.report(error);
                        }
                    }
                }
                Err(error) => self.report(error),
            }
        }
    }
    fn report(&mut self, e: SafeError) {
        if let Some(c) = self.controller.as_mut() {
            c.notice(e.message());
        }
    }
    fn child_send(&mut self, message: ChildMessage) {
        if self
            .pipe
            .as_ref()
            .is_some_and(|p| p.tx.try_send(message).is_err())
        {
            self.report(SafeError::new(
                ErrorCode::GuiDisconnected,
                SendDisposition::NotSent,
            ));
        }
    }
}
impl eframe::App for NativeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(40));
        if let Some(pipe) = self.pipe.as_mut() {
            while let Ok(message) = pipe.rx.try_recv() {
                match message {
                    ParentMessage::SendGranted { session_id } if session_id == pipe.id => {
                        if let Some(c) = self.controller.as_mut() {
                            if c.grant(&self.runtime.handle()).is_err() {
                                self.audio_load = None;
                                self.audio_load_label.clear();
                                c.stop_worker();
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                            }
                        }
                    }
                    ParentMessage::Committed { view } if view.session_id == pipe.id => {
                        self.audio_load = None;
                        self.audio_load_label.clear();
                        pipe.waiting = None;
                        if self
                            .controller
                            .as_mut()
                            .is_some_and(|c| c.committed(&view).is_err())
                        {
                            pipe.can_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                        }
                        if pipe.closing {
                            pipe.can_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    ParentMessage::Cancel { session_id } if session_id == pipe.id => {
                        self.audio_load = None;
                        self.audio_load_label.clear();
                        if let Some(c) = self.controller.as_mut() {
                            c.stop_worker();
                        }
                        pipe.waiting = Some(Instant::now());
                    }
                    ParentMessage::Close => {
                        self.audio_load = None;
                        self.audio_load_label.clear();
                        if let Some(c) = self.controller.as_mut() {
                            c.stop_worker();
                        }
                        pipe.can_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                    }
                    _ => {
                        self.audio_load = None;
                        self.audio_load_label.clear();
                        if let Some(c) = self.controller.as_mut() {
                            c.stop_worker();
                        }
                        pipe.can_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close)
                    }
                }
            }
            if pipe.rx.is_closed()
                || pipe
                    .waiting
                    .is_some_and(|t| t.elapsed() > Duration::from_secs(5))
            {
                self.audio_load = None;
                self.audio_load_label.clear();
                pipe.can_close = true;
                if let Some(c) = self.controller.as_mut() {
                    c.stop_worker()
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close)
            }
        }
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            self.cancel_audio_load();
            if let Some(c) = self.controller.as_mut() {
                c.stop_worker();
            }
            if let Some(p) = self.pipe.as_mut() {
                if !p.can_close {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    if !p.closing {
                        p.closing = true;
                        p.waiting = Some(Instant::now());
                        let _ = p.tx.try_send(ChildMessage::Closed {
                            session_id: p.id.clone(),
                        });
                    }
                }
            }
        }
        if let Some(c) = self.controller.as_mut() {
            if self.pipe.is_none() {
                let _ = c.poll();
            } else if let Some(proposal) = c.take_proposal() {
                if let Some(p) = self.pipe.as_mut() {
                    p.waiting = Some(Instant::now());
                    let _ = p.tx.try_send(proposal);
                }
            }
        }
        let mut action = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.settings);
            if let Some(c) = self.controller.as_mut() {
                action = c.render_with_loading(ui, self.audio_load.is_some())
            } else if let Some(f) = self.fixture.as_mut() {
                let _ = render(ui, f);
            }
        });
        let accepting_actions = !close_requested
            && !self
                .pipe
                .as_ref()
                .is_some_and(|p| p.closing || p.can_close || p.waiting.is_some());
        if let Some(action) = action.filter(|_| accepting_actions) {
            match action {
                GuiAction::SelectAudio => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("音声", &["wav", "mp3", "aac", "amr", "3gp", "3gpp"])
                        .pick_file()
                    {
                        self.start_audio_load(path);
                    }
                }
                GuiAction::OpenSettings => {
                    if let Some(c) = self.controller.as_mut() {
                        match c.begin_settings() {
                            Ok(view) => {
                                self.settings_host = view.host_label;
                                self.settings_key.zeroize();
                                self.settings = true
                            }
                            Err(e) => self.report(e),
                        }
                    }
                }
                GuiAction::Send if self.audio_load.is_none() => {
                    if self.pipe.is_some() {
                        let intent = self.controller.as_mut().unwrap().prepare_intent();
                        match intent {
                            Ok(message) => self.child_send(message),
                            Err(e) => self.report(e),
                        }
                    } else if let Some(c) = self.controller.as_mut() {
                        if let Err(e) = c.send(self.runtime.handle()) {
                            self.report(e)
                        }
                    }
                }
                GuiAction::Send => {}
                GuiAction::Cancel => {
                    self.cancel_audio_load();
                    if let Some(p) = self.pipe.as_mut() {
                        if let Some(c) = self.controller.as_mut() {
                            c.stop_worker()
                        }
                        p.waiting = Some(Instant::now());
                        let _ = p.tx.try_send(ChildMessage::Closed {
                            session_id: p.id.clone(),
                        });
                    } else if let Some(c) = self.controller.as_mut() {
                        let _ = c.cancel();
                    }
                }
                GuiAction::NewSession => {
                    if let Some(c) = self.controller.as_mut() {
                        let _ = c.new_session();
                    }
                }
            }
        }
        if self.settings {
            let mut open = true;
            let mut save = false;
            let mut cancel = false;
            let previous_region = self.controller.as_ref().unwrap().settings_region();
            let mut region = previous_region;
            egui::Window::new("接続設定")
                .open(&mut open)
                .resizable(true)
                .max_width((ctx.screen_rect().width() - 32.0).max(180.0))
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label(format!("地域: {}", region.label()));
                    ui.horizontal_wrapped(|ui| {
                        for candidate in Region::ALL {
                            if qwen_audio::gui::settings_button(ui, candidate.label()).clicked() {
                                region = candidate;
                            }
                        }
                    });
                    if region != previous_region {
                        self.settings_host.clear();
                        self.settings_key.zeroize();
                    }
                    ui.label("Workspace API Host（選択した地域のHTTPS URL）");
                    ui.add(egui::Label::new(region.example_url()).wrap());
                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings_host)
                            .desired_width(ui.available_width()),
                    );
                    ui.add(egui::Label::new("APIキー（同じ接続先で未入力なら保存済みキーを保持。地域・Workspaceを変える場合はキーを入力し直す）").wrap());
                    ui.add(
                        egui::TextEdit::singleline(&mut *self.settings_key)
                            .password(true)
                            .desired_width(ui.available_width()),
                    );
                    ui.add(egui::Label::new("保存先: Windows 資格情報マネージャー。保存だけでは送信しない。地域の選択は利用権・接続成功や全処理の所在地を保証しない。").wrap());
                    if let Some(notice) = &self.controller.as_ref().unwrap().model().notice {
                        ui.add(egui::Label::new(notice).wrap());
                    }
                    save = qwen_audio::gui::settings_button(ui, "保存").clicked();
                    cancel = qwen_audio::gui::settings_button(ui, "取消し").clicked();
                });
            if region != previous_region {
                if let Err(error) = self
                    .controller
                    .as_mut()
                    .unwrap()
                    .set_settings_region(region)
                {
                    self.report(error);
                }
            }
            if save {
                let key = if self.settings_key.is_empty() {
                    None
                } else {
                    Some(self.settings_key.to_string())
                };
                let result = self
                    .controller
                    .as_mut()
                    .unwrap()
                    .set_settings_draft_for_region(self.settings_host.clone(), key, region)
                    .and_then(|_| self.controller.as_mut().unwrap().save_settings());
                match result {
                    Ok(()) => {
                        self.settings = false;
                        self.settings_key.zeroize()
                    }
                    Err(e) => self.report(e),
                }
            }
            if cancel || !open {
                self.settings = false;
                self.settings_key.zeroize();
                self.controller.as_mut().unwrap().cancel_settings();
            }
        }
        if !close_requested {
            self.poll_audio_load();
        }
        let leaving_window = close_requested
            || self.pipe.as_ref().is_some_and(|p| p.closing || p.can_close)
            || self.close_after.is_some_and(|d| self.started.elapsed() > d);
        if !leaving_window && take_audio_cleanup_warning() {
            if let Some(controller) = self.controller.as_mut() {
                controller.notice("検査用の一時音声コピーを削除できなかった。音声が一時領域に残っている可能性がある。");
            }
        }
        if self.capture.is_some()
            && !self.capture_requested
            && self.started.elapsed() > Duration::from_millis(500)
        {
            self.capture_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                if let Some(path) = self.capture.take() {
                    let pixels: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                    let _ = image::save_buffer(
                        path,
                        &pixels,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    );
                }
            }
        }
        if self.close_after.is_some_and(|d| self.started.elapsed() > d) {
            self.cancel_audio_load();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
#[cfg(debug_assertions)]
fn fixture(phase: &str) -> Result<GuiModel, SafeError> {
    let mut r = EvaluationResult {
        feedback: Feedback {
            assessment: Assessment::Assessed,
            heard_text: Some("The sun is bright.".into()),
            summary: "文全体が聞き取りやすい。強弱とリズムを練習する。".into(),
            strengths: vec!["語尾まで読んでいる。".into()],
            improvements: vec![Improvement {
                reference_excerpt: "bright".into(),
                observation: "brightを少し強くすると意味が伝わりやすい。".into(),
                practice: "brightを強め、前後の語を軽く読んで練習する。".into(),
            }],
            unassessable_reason: None,
        },
        requested_model: REQUESTED_MODEL.into(),
        requested_effort: REQUESTED_EFFORT.into(),
        actual_model: None,
        actual_effort: None,
        usage: None,
    };
    let singapore = phase == "input-singapore";
    if phase == "mismatch" {
        r.feedback = Feedback {
            assessment: Assessment::ReferenceMismatch,
            heard_text: Some("I went to school yesterday.".into()),
            summary: "別の英文が聞き取れた。表示された例文を読み直して再録音する。".into(),
            strengths: Vec::new(),
            improvements: Vec::new(),
            unassessable_reason: None,
        };
    }
    let phase = match phase {
        "input" | "input-singapore" => GuiPhase::AwaitingUser,
        "running" => GuiPhase::Running,
        "result" | "mismatch" => GuiPhase::Completed(r),
        "error" => GuiPhase::Failed(SafeError::new(
            ErrorCode::Network,
            SendDisposition::MayHaveBeenSent,
        )),
        _ => {
            return Err(SafeError::new(
                ErrorCode::InvalidState,
                SendDisposition::NotSent,
            ))
        }
    };
    Ok(GuiModel {
        reference_text: "The sun is bright.".into(),
        file_label: "synthetic-fixture.wav".into(),
        audio_info: Some(AudioInfo {
            byte_len: 32044,
            sample_rate: 16000,
            channels: 1,
            frames: 16000,
            duration_seconds: 1.0,
        }),
        audio_format: Some("wav"),
        host_label: format!(
            "https://synthetic.{}.maas.aliyuncs.com/compatible-mode/v1",
            if singapore {
                Region::Singapore.id()
            } else {
                Region::Tokyo.id()
            }
        ),
        credential_present: true,
        mcp_origin: true,
        phase,
        notice: Some("合成画面。資格情報・通信は構築していない。".into()),
    })
}
fn run() -> Result<(), SafeError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| SafeError::new(ErrorCode::GuiLaunch, SendDisposition::NotSent))?;
    if args == ["--mcp"] {
        let launcher = Arc::new(NativeGuiLauncher::new()?);
        return runtime.block_on(serve_mcp(tokio::io::stdin(), tokio::io::stdout(), launcher));
    }
    let mut fixture_model = None;
    let mut width = 800.0;
    let mut zoom = 1.0;
    let mut capture = None;
    let mut close_after = None;
    if args.first().is_some_and(|a| a == "--ui-fixture") {
        #[cfg(not(debug_assertions))]
        return Err(SafeError::new(
            ErrorCode::InvalidState,
            SendDisposition::NotSent,
        ));
        #[cfg(debug_assertions)]
        {
            fixture_model = Some(fixture(args.get(1).map(String::as_str).unwrap_or(""))?);
            let mut i = 2;
            while i < args.len() {
                let v = args.get(i + 1).ok_or_else(|| {
                    SafeError::new(ErrorCode::InvalidState, SendDisposition::NotSent)
                })?;
                match args[i].as_str() {
                    "--width" => {
                        width = v
                            .parse::<f32>()
                            .ok()
                            .filter(|v| (320.0..=2000.0).contains(v))
                            .ok_or_else(|| {
                                SafeError::new(ErrorCode::InvalidState, SendDisposition::NotSent)
                            })?
                    }
                    "--zoom" => {
                        zoom = v
                            .parse::<f32>()
                            .ok()
                            .filter(|v| (1.0..=2.0).contains(v))
                            .ok_or_else(|| {
                                SafeError::new(ErrorCode::InvalidState, SendDisposition::NotSent)
                            })?
                    }
                    "--close-after-ms" => {
                        close_after = Some(Duration::from_millis(v.parse().map_err(|_| {
                            SafeError::new(ErrorCode::InvalidState, SendDisposition::NotSent)
                        })?))
                    }
                    "--capture" => {
                        let p = PathBuf::from(v);
                        let parent =
                            p.parent()
                                .and_then(|p| p.canonicalize().ok())
                                .ok_or_else(|| {
                                    SafeError::new(
                                        ErrorCode::InvalidState,
                                        SendDisposition::NotSent,
                                    )
                                })?;
                        let allowed = ["QWEN-AUDIO-001", "QWEN-FORMATS-001", "QWEN-SETTINGS-001"]
                            .iter()
                            .filter_map(|task| {
                                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                    .join("../../tasks")
                                    .join(task)
                                    .canonicalize()
                                    .ok()
                            })
                            .any(|allowed| parent.starts_with(allowed));
                        if !allowed || p.extension().and_then(|s| s.to_str()) != Some("png") {
                            return Err(SafeError::new(
                                ErrorCode::InvalidState,
                                SendDisposition::NotSent,
                            ));
                        }
                        capture = Some(p)
                    }
                    _ => {
                        return Err(SafeError::new(
                            ErrorCode::InvalidState,
                            SendDisposition::NotSent,
                        ))
                    }
                }
                i += 2;
            }
        }
    } else if !args.is_empty() && args != ["--gui-child"] {
        return Err(SafeError::new(
            ErrorCode::InvalidState,
            SendDisposition::NotSent,
        ));
    }
    let mut pipe = None;
    let mut boot = None;
    if args == ["--gui-child"] {
        let (id, reference) = runtime.block_on(async {
            let mut input = tokio::io::stdin();
            let line = qwen_audio::read_child_boot(&mut input).await?;
            match line {
                ParentMessage::Boot {
                    session_id,
                    reference_text,
                } => {
                    ReferenceText::new(reference_text.clone())?;
                    Ok((session_id, reference_text))
                }
                _ => Err(SafeError::new(
                    ErrorCode::ProtocolInvalid,
                    SendDisposition::NotSent,
                )),
            }
        })?;
        let (tx, mut out) = mpsc::channel(8);
        let (incoming, rx) = mpsc::channel(8);
        runtime.spawn(async move {
            let mut stdout = tokio::io::stdout();
            while let Some(msg) = out.recv().await {
                if qwen_audio::write_child_event(&mut stdout, &msg)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
        runtime.spawn(async move {
            let mut stdin = tokio::io::stdin();
            loop {
                match qwen_audio::read_child_boot(&mut stdin).await {
                    Ok(m) => {
                        if incoming.send(m).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        boot = Some((id.clone(), reference));
        pipe = Some(ChildPipe {
            tx,
            rx,
            id,
            waiting: None,
            closing: false,
            can_close: false,
        });
    }
    let controller = if fixture_model.is_some() {
        None
    } else {
        let credentials = Arc::new(WindowsCredentialStore::new());
        let transport = Arc::new(ReqwestTransport::new()?);
        Some(if let Some((id, reference)) = boot {
            GuiController::child(reference, id, credentials, transport)?
        } else {
            GuiController::new(String::new(), credentials, transport)?
        })
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, 800.0])
            .with_min_inner_size([320.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Qwen 音読練習",
        options,
        Box::new(move |cc| {
            let font = setup_fonts(&cc.egui_ctx);
            cc.egui_ctx.all_styles_mut(|style| {
                style
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
                style
                    .text_styles
                    .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
                style
                    .text_styles
                    .insert(egui::TextStyle::Heading, egui::FontId::proportional(22.0));
                style.spacing.button_padding = egui::vec2(12.0, 8.0);
                style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            });
            cc.egui_ctx.set_zoom_factor(zoom);
            let mut app = NativeApp {
                controller,
                fixture: fixture_model,
                runtime,
                pipe,
                audio_load: None,
                audio_load_label: String::new(),
                settings: false,
                settings_host: String::new(),
                settings_key: Zeroizing::new(String::new()),
                capture,
                capture_requested: false,
                started: Instant::now(),
                close_after,
            };
            if !font {
                app.report(SafeError::new(
                    ErrorCode::GuiLaunch,
                    SendDisposition::NotSent,
                ));
            }
            if let Some(p) = &app.pipe {
                let _ = p.tx.try_send(ChildMessage::Ready {
                    session_id: p.id.clone(),
                });
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|_| SafeError::new(ErrorCode::GuiLaunch, SendDisposition::NotSent))
}
fn main() {
    let result = run();
    shutdown_audio_jobs();
    if take_audio_cleanup_warning() {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title("一時音声コピーの削除")
            .set_description("検査用の一時音声コピーを削除できなかった。音声が一時領域に残っている可能性がある。")
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
    if let Err(e) = result {
        eprintln!("{}", e.message());
        std::process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct SyntheticStore;
    impl CredentialStore for SyntheticStore {
        fn load(&self) -> Result<Option<Connection>, SafeError> {
            Ok(Some(Connection::new(
                TokyoHost::parse("https://fixture.ap-northeast-1.maas.aliyuncs.com").unwrap(),
                ApiKey::new("synthetic-unused-key".into()).unwrap(),
            )))
        }
        fn save(&self, _: &Connection) -> Result<(), SafeError> {
            unreachable!("this fixture never writes credentials")
        }
    }
    #[derive(Default)]
    struct NoNetwork(AtomicUsize);
    impl Transport for NoNetwork {
        fn send<'a>(&'a self, _: ProviderRequest) -> TransportFuture<'a> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(TransportFailure::Network) })
        }
    }
    fn app(transport: Arc<NoNetwork>) -> NativeApp {
        NativeApp {
            controller: Some(
                GuiController::new(
                    "The sun is bright.".into(),
                    Arc::new(SyntheticStore),
                    transport,
                )
                .unwrap(),
            ),
            fixture: None,
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            pipe: None,
            audio_load: None,
            audio_load_label: String::new(),
            settings: false,
            settings_host: String::new(),
            settings_key: Zeroizing::new(String::new()),
            capture: None,
            capture_requested: false,
            started: Instant::now(),
            close_after: None,
        }
    }
    fn assert_ipc_discards_load(closing: bool, waiting: bool) {
        let transport = Arc::new(NoNetwork::default());
        let mut app = app(transport.clone());
        let controller = app.controller.as_mut().unwrap();
        controller
            .select_audio(
                AudioInput::parse(include_bytes!("../tests/fixtures/formats/tone.wav").to_vec())
                    .unwrap(),
                "previous.wav".into(),
            )
            .unwrap();
        app.start_audio_load(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/formats/tone.wav"),
        );
        assert!(app.audio_load.is_some());
        assert!(app
            .controller
            .as_ref()
            .unwrap()
            .model()
            .audio_info
            .is_none());
        let (tx, _out) = mpsc::channel(1);
        let (_incoming, rx) = mpsc::channel(1);
        let id = SessionManager::new().start().unwrap().session_id;
        app.pipe = Some(ChildPipe {
            tx,
            rx,
            id,
            waiting: waiting.then(Instant::now),
            closing,
            can_close: false,
        });
        app.poll_audio_load();
        assert!(app.audio_load.is_none());
        assert!(app.audio_load_label.is_empty());
        let controller = app.controller.as_mut().unwrap();
        assert!(controller.model().audio_info.is_none());
        assert!(controller.model().audio_format.is_none());
        assert_eq!(
            controller.send(app.runtime.handle()).unwrap_err().code(),
            ErrorCode::AudioEmpty
        );
        assert_eq!(transport.0.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn ipc_closing_discards_current_load_without_restoring_previous_audio() {
        assert_ipc_discards_load(true, false);
    }
    #[test]
    fn ipc_terminal_wait_discards_current_load_without_sending() {
        assert_ipc_discards_load(false, true);
    }
}
