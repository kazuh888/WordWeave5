use super::*;

fn check_speech_start(recording: bool) -> Result<(), String> {
    if recording { Err("録音中は読み上げを開始できない。録音を終了またはキャンセルしてください。".into()) }
    else { Ok(()) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn speech_start_is_rejected_while_recording() {
        assert!(super::check_speech_start(true).is_err());
        assert!(super::check_speech_start(false).is_ok());
    }
}

impl WordApp {
    pub(super) fn check_speech_start(&self) -> Result<(), String> { check_speech_start(self.recorder.is_some()) }
    pub(super) fn speech_rate(&self) -> f64 {
        self.progress.settings.speech_rate.unwrap_or(if self.progress.settings.slow_speech { 0.8 } else { 1.0 })
    }

    pub(super) fn change_speech_rate(&mut self, rate: f64) -> Result<(), String> {
        // Validate even without loaded audio; the player applies the same range.
        if !rate.is_finite() || !(0.5..=4.0).contains(&rate) { return Err("読み上げ速度は0.5〜4倍で指定してください。".into()); }
        if self.speech_visible { self.speaker.set_rate(rate)?; }
        self.progress.settings.speech_rate = Some(rate);
        self.dirty = true;
        if let Some(operation) = &self.speech_operation { operation.event(DiagnosticStage::Play, DiagnosticEvent::RateChanged); }
        Ok(())
    }

    pub(super) fn stop_speech(&mut self) -> bool {
        match self.speaker.stop() {
            Ok(()) => {
                if let Some(operation) = self.speech_operation.take() { operation.event(DiagnosticStage::Play, DiagnosticEvent::Stopped); }
                self.speech_visible = false;
                true
            }
            Err(error) => {
                if let Some(operation) = &self.speech_operation { operation.fail(DiagnosticStage::Play, DiagnosticError::Unavailable); }
                self.message = error;
                false
            }
        }
    }

    pub(super) fn speech_controls(&mut self, ctx: &egui::Context) {
        if !self.speech_visible { return; }
        let snapshot: media::PlaybackSnapshot = match self.speaker.snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                if let Some(operation) = self.speech_operation.take() { operation.fail(DiagnosticStage::Play, DiagnosticError::Unavailable); }
                self.stop_speech(); self.speech_visible = false; self.message = error;
                return;
            }
        };
        if !snapshot.loaded { self.speech_visible = false; return; }
        if snapshot.loading || snapshot.playing { ctx.request_repaint_after(Duration::from_millis(100)); }
        let (mut back, mut toggle, mut stop, mut forward) = (false, false, false, false);
        let (mut seek, mut rate_change) = (None, None);
        egui::TopBottomPanel::bottom("speech-controls").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("読み上げ");
                ui.label(if snapshot.loading { "準備中…" } else if snapshot.playing { "再生中" } else { "一時停止／停止中" });
                back = ui.add_enabled(snapshot.can_seek && !snapshot.loading, egui::Button::new("5秒戻る")).clicked();
                toggle = ui.add_enabled(!snapshot.loading, egui::Button::new(if snapshot.playing { "一時停止" } else { "再生" })).clicked();
                stop = ui.button("停止（先頭へ）").clicked();
                forward = ui.add_enabled(snapshot.can_seek && !snapshot.loading, egui::Button::new("5秒進む")).clicked();
                let mut rate = self.speech_rate();
                if ui.add_enabled(self.fatal.is_none(), egui::Slider::new(&mut rate, 0.5..=4.0).step_by(0.1).text("倍速")).changed() { rate_change = Some(rate); }
            });
            ui.horizontal(|ui| {
                ui.small(format!("{:.1} / {:.1} 秒", snapshot.position_seconds, snapshot.duration_seconds));
                ui.small(format!("再生速度 {:.1}倍", snapshot.rate));
                let mut position = snapshot.position_seconds;
                if ui.add_enabled(snapshot.can_seek && !snapshot.loading && snapshot.duration_seconds > 0.0,
                    egui::Slider::new(&mut position, 0.0..=snapshot.duration_seconds.max(0.1)).show_value(false).text("再生位置")).changed() { seek = Some(position); }
            });
        });
        let action = if stop { Some((self.speaker.stop(), DiagnosticEvent::Stopped)) }
            else if toggle { Some((if snapshot.playing { self.speaker.pause() } else { self.speaker.resume() },
                if snapshot.playing { DiagnosticEvent::Paused } else { DiagnosticEvent::Resumed })) }
            else if back || forward { Some((self.speaker.seek_relative(if back { -5.0 } else { 5.0 }), DiagnosticEvent::Seeked)) }
            else { seek.map(|position| (self.speaker.seek(position), DiagnosticEvent::Seeked)) };
        if let Some((result, event)) = action {
            match result {
                Ok(()) => { if let Some(operation) = &self.speech_operation { operation.event(DiagnosticStage::Play, event); } }
                Err(error) => {
                    if let Some(operation) = &self.speech_operation { operation.fail(DiagnosticStage::Play, DiagnosticError::Unavailable); }
                    self.message = error;
                }
            }
        }
        if let Some(rate) = rate_change { if let Err(error) = self.change_speech_rate(rate) { self.message = error; } }
    }

    pub(super) fn recording_controls(&mut self, ui: &mut egui::Ui, finish_label: &str) {
        let Some(recorder) = &self.recorder else { return; };
        let paused = recorder.is_paused();
        ui.label(format!("{} {:.1}秒 / 最大30秒", if paused { "録音一時停止中" } else { "録音中" }, recorder.elapsed().as_secs_f32()));
        ui.add(egui::ProgressBar::new(if paused { 0.0 } else { recorder.level() }).text("入力音量"));
        let (mut toggle, mut finish) = (false, false);
        ui.horizontal_wrapped(|ui| {
            toggle = ui.button(if paused { "録音を再開" } else { "録音を一時停止" }).clicked();
            finish = ui.button(finish_label).clicked();
            if ui.button("録音をキャンセル…").clicked() { self.recording_cancel_confirm = true; }
        });
        if toggle {
            if let Some(recorder) = self.recorder.as_mut() {
                let result = if paused { recorder.resume() } else { recorder.pause() };
                if let Some(operation) = &self.recording_operation {
                    if result.is_ok() { operation.event(DiagnosticStage::Record, if paused { DiagnosticEvent::Resumed } else { DiagnosticEvent::Paused }); }
                    else { operation.fail(DiagnosticStage::Record, DiagnosticError::Unavailable); }
                }
                self.message = match result {
                    Ok(()) => if paused { "録音を再開した。" } else { "録音を一時停止した。停止時間は録音に含めない。" }.into(),
                    Err(error) => error,
                };
            }
        }
        if finish { self.stop_recording(); }
    }

    pub(super) fn cancel_recording(&mut self) {
        if let Some(recorder) = self.recorder.take() { recorder.cancel(); }
        if let Some(operation) = self.recording_operation.take() { operation.event(DiagnosticStage::Record, DiagnosticEvent::Cancelled); }
        self.chat_recording_id = None;
        self.recording_cancel_confirm = false;
        self.message = "今回の録音をキャンセルした。保存済みの録音は削除していない。".into();
    }

    pub(super) fn recording_cancel_dialog(&mut self, ctx: &egui::Context) {
        if self.recorder.is_none() { self.recording_cancel_confirm = false; }
        if !self.recording_cancel_confirm { return; }
        let (mut discard, mut keep) = (false, false);
        egui::Window::new("録音のキャンセルを確認").collapsible(false).resizable(false).show(ctx, |ui| {
            ui.label("今回収録している音声を保存せず破棄する。この操作は取り消せない。");
            ui.label("過去に保存した録音・添付・教材は変更しない。");
            ui.horizontal(|ui| {
                discard = ui.button("今回の録音を破棄").clicked();
                keep = ui.button("録音に戻る").clicked();
            });
        });
        if discard { self.cancel_recording(); }
        if keep { self.recording_cancel_confirm = false; }
    }
}
