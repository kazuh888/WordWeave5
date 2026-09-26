use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpeechButton {
    Back,
    Toggle,
    Forward,
}

fn selected_speech_button(
    previous: Option<SpeechButton>,
    back: bool,
    toggle: bool,
    stop: bool,
    forward: bool,
) -> Option<SpeechButton> {
    if stop {
        None
    } else if back {
        Some(SpeechButton::Back)
    } else if toggle {
        Some(SpeechButton::Toggle)
    } else if forward {
        Some(SpeechButton::Forward)
    } else {
        previous
    }
}

fn check_speech_start(recording: bool) -> Result<(), String> {
    if recording {
        Err("録音中は読み上げを開始できない。録音を終了またはキャンセルしてください。".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_clicked_button_is_exclusive_and_stop_clears_it() {
        let mut selected = None;
        for (back, toggle, forward, expected) in [
            (true, false, false, SpeechButton::Back),
            (false, true, false, SpeechButton::Toggle),
            (false, false, true, SpeechButton::Forward),
        ] {
            selected = selected_speech_button(selected, back, toggle, false, forward);
            assert_eq!(selected, Some(expected));
            assert_eq!(
                selected_speech_button(selected, false, false, false, false),
                selected
            );
        }
        assert_eq!(
            selected_speech_button(selected, false, false, true, false),
            None
        );
    }

    #[test]
    fn playback_preferences_survive_stop_close_and_storage_reopen() {
        let (_, mut app, root) = super::super::harness_tests::fixture();
        app.change_speech_volume(0.37).unwrap();
        app.change_speech_repeat(true).unwrap();
        app.speech_visible = true;
        assert!(app.stop_speech());
        assert!(!app.speech_visible);
        let stopped = app.speaker.snapshot().unwrap();
        assert_eq!(stopped.volume, 0.37);
        assert!(stopped.repeat);
        assert_eq!(app.progress.settings.speech_volume, 0.37);
        assert!(app.progress.settings.speech_repeat);
        app.persist();
        drop(app);
        let storage = Storage::at(root.join("data")).unwrap();
        let restored = storage.load().unwrap();
        assert_eq!(restored.settings.speech_volume, 0.37);
        assert!(restored.settings.speech_repeat);
        drop(storage);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn toggling_play_pause_keeps_button_rectangles_fixed() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        ctx.set_zoom_factor(0.8);
        for width in [1150.0, 500.0] {
            let mut previous = None;
            for playing in [false, true, false] {
                let mut rects = Vec::new();
                for frame in 0..3 {
                    let _out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 950.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            app.speech_panel(
                                ctx,
                                &media::PlaybackSnapshot {
                                    loaded: true,
                                    playing,
                                    can_seek: true,
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    if frame < 2 {
                        continue;
                    }
                    rects.clear();
                    for key in ["back", "toggle", "stop", "forward", "close"] {
                        rects.push(
                            ctx.data(|d| {
                                d.get_temp::<egui::Rect>(egui::Id::new(("speech-control", key)))
                            })
                            .unwrap(),
                        );
                    }
                }
                if let Some(before) = previous {
                    assert_eq!(rects, before);
                }
                previous = Some(rects);
            }
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn speech_panel_has_readable_controls_and_preserves_stop_action() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        let before = serde_json::to_value(&app.progress).unwrap();
        for width in [1150.0, 500.0] {
            for zoom in [0.8, 1.6] {
                ctx.set_zoom_factor(zoom);
                let snapshot = media::PlaybackSnapshot {
                    loaded: true,
                    can_seek: true,
                    duration_seconds: 12.0,
                    ..Default::default()
                };
                let mut frame = |events| {
                    let mut actions = (false, false, None);
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 950.0),
                            )),
                            events,
                            ..Default::default()
                        },
                        |ctx| {
                            let a = app.speech_panel(ctx, &snapshot);
                            actions = (a.stop, a.close, a.repeat);
                        },
                    );
                    (out, actions)
                };
                for _ in 0..3 {
                    frame(vec![]);
                }
                let (out, _) = frame(vec![]);
                for shape in &out.shapes {
                    if let egui::Shape::Text(t) = &shape.shape {
                        if t.galley.text().is_empty() {
                            continue;
                        }
                        let ink = t.galley.mesh_bounds.translate(t.pos.to_vec2());
                        assert!(
                            ink.right() <= ctx.screen_rect().right() + 1.5,
                            "clipped label: {} {ink:?}",
                            t.galley.text()
                        );
                    }
                }
                let button = ctx
                    .data(|d| d.get_temp::<egui::Rect>(egui::Id::new(("speech-control", "stop"))))
                    .unwrap();
                assert!(button.height() >= 40.0, "{button:?}");
                assert!(
                    button.right() <= ctx.screen_rect().right() + 1.0,
                    "{button:?}"
                );
                let pos = button.center();
                frame(vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]);
                assert!(
                    frame(vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    }])
                    .1
                     .0
                );
                let repeat_pos = ctx
                    .data(|d| d.get_temp::<egui::Rect>(egui::Id::new(("speech-control", "repeat"))))
                    .unwrap()
                    .center();
                frame(vec![
                    egui::Event::PointerMoved(repeat_pos),
                    egui::Event::PointerButton {
                        pos: repeat_pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]);
                assert_eq!(
                    frame(vec![egui::Event::PointerButton {
                        pos: repeat_pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    }])
                    .1
                     .2,
                    Some(true)
                );
                let close_pos = out
                    .shapes
                    .iter()
                    .find_map(|s| match &s.shape {
                        egui::Shape::Text(t) if t.galley.text() == "×" => {
                            Some(t.galley.mesh_bounds.translate(t.pos.to_vec2()).center())
                        }
                        _ => None,
                    })
                    .unwrap();
                frame(vec![
                    egui::Event::PointerMoved(close_pos),
                    egui::Event::PointerButton {
                        pos: close_pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]);
                assert!(
                    frame(vec![egui::Event::PointerButton {
                        pos: close_pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    }])
                    .1
                     .1
                );
            }
        }
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), before);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn speech_panel_compacts_at_standard_width_and_wraps_without_clipping() {
        let (ctx, mut app, root) = super::super::harness_tests::fixture();
        ctx.set_zoom_factor(1.0);
        let snapshot = media::PlaybackSnapshot {
            loaded: true,
            can_seek: true,
            duration_seconds: 125.0,
            volume: 0.75,
            rate: 1.25,
            ..Default::default()
        };
        let mut panel_heights = Vec::new();
        for width in [1150.0, 500.0] {
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 950.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        app.speech_panel(ctx, &snapshot);
                    },
                );
            }
            let rect = |key| {
                ctx.data(|d| d.get_temp::<egui::Rect>(egui::Id::new(("speech-control", key))))
                    .unwrap()
            };
            let panel = rect("panel");
            panel_heights.push(panel.height());
            for key in [
                "back", "toggle", "stop", "forward", "volume", "rate", "repeat", "close", "seek",
            ] {
                let control = rect(key);
                assert!(
                    panel.contains_rect(control),
                    "{key} is outside {panel:?}: {control:?}"
                );
                assert!(
                    control.right() <= width + 1.0,
                    "{key} is clipped: {control:?}"
                );
            }
            assert!(
                rect("rate").width() < 100.0,
                "rate selector is not compact: {:?}",
                rect("rate")
            );
            let close = rect("close");
            assert!(
                (32.0..=34.0).contains(&close.width()),
                "close width: {close:?}"
            );
            assert!(
                (32.0..=34.0).contains(&close.height()),
                "close height: {close:?}"
            );
            assert!(rect("seek").center().y > rect("toggle").center().y);
        }
        assert!(
            (110.0..=145.0).contains(&panel_heights[0]),
            "standard panel height: {}",
            panel_heights[0]
        );
        assert!(
            panel_heights[1] > panel_heights[0],
            "narrow panel must wrap: {panel_heights:?}"
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn speech_start_is_rejected_while_recording() {
        assert!(super::check_speech_start(true).is_err());
        assert!(super::check_speech_start(false).is_ok());
    }
}

impl WordApp {
    pub(super) fn check_speech_start(&self) -> Result<(), String> {
        check_speech_start(self.recorder.is_some())
    }
    pub(super) fn speech_rate(&self) -> f64 {
        self.progress
            .settings
            .speech_rate
            .unwrap_or(if self.progress.settings.slow_speech {
                0.8
            } else {
                1.0
            })
    }

    pub(super) fn change_speech_rate(&mut self, rate: f64) -> Result<(), String> {
        // Validate even without loaded audio; the player applies the same range.
        if !rate.is_finite() || !(0.5..=4.0).contains(&rate) {
            return Err("読み上げ速度は0.5〜4倍で指定してください。".into());
        }
        if self.speech_visible {
            self.speaker.set_rate(rate)?;
        }
        if self.settings_editor.preview_speech {
            self.settings_editor.draft.speech_rate = Some(rate);
        } else {
            self.progress.settings.speech_rate = Some(rate);
            self.dirty = true;
        }
        if let Some(operation) = &self.speech_operation {
            operation.event(DiagnosticStage::Play, DiagnosticEvent::RateChanged);
        }
        Ok(())
    }

    pub(super) fn change_speech_volume(&mut self, volume: f64) -> Result<(), String> {
        self.speaker.set_volume(volume)?;
        self.progress.settings.speech_volume = volume;
        self.dirty = true;
        Ok(())
    }

    pub(super) fn change_speech_repeat(&mut self, repeat: bool) -> Result<(), String> {
        self.speaker.set_repeat(repeat)?;
        self.progress.settings.speech_repeat = repeat;
        self.dirty = true;
        Ok(())
    }

    pub(super) fn stop_speech(&mut self) -> bool {
        match self.speaker.stop() {
            Ok(()) => {
                if let Some(operation) = self.speech_operation.take() {
                    operation.event(DiagnosticStage::Play, DiagnosticEvent::Stopped);
                }
                self.speech_visible = false;
                self.speech_selected = None;
                true
            }
            Err(error) => {
                if let Some(operation) = &self.speech_operation {
                    operation.fail(DiagnosticStage::Play, DiagnosticError::Unavailable);
                }
                self.message = error;
                false
            }
        }
    }

    // Rendering is shared with the isolated native preview; no audio or storage writes here.
    pub(super) fn speech_panel(
        &mut self,
        ctx: &egui::Context,
        snapshot: &media::PlaybackSnapshot,
    ) -> super::playback_panel::Actions {
        let actions = super::playback_panel::show(
            ctx,
            snapshot,
            self.speech_selected,
            if self.settings_editor.preview_speech {
                self.settings_editor.draft.speech_rate.unwrap_or(
                    if self.settings_editor.draft.slow_speech { 0.8 } else { 1.0 })
            } else { self.speech_rate() },
            self.fatal.is_none(),
        );
        self.speech_selected = selected_speech_button(
            self.speech_selected,
            actions.back,
            actions.toggle,
            actions.stop,
            actions.forward,
        );
        actions
    }

    pub(super) fn speech_controls(&mut self, ctx: &egui::Context) {
        if !self.speech_visible {
            return;
        }
        let snapshot: media::PlaybackSnapshot = match self.speaker.snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                if let Some(operation) = self.speech_operation.take() {
                    operation.fail(DiagnosticStage::Play, DiagnosticError::Unavailable);
                }
                self.stop_speech();
                self.speech_visible = false;
                self.message = error;
                return;
            }
        };
        if !snapshot.loaded {
            self.speech_visible = false;
            return;
        }
        if snapshot.loading || snapshot.playing {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        let super::playback_panel::Actions {
            back,
            toggle,
            stop,
            forward,
            seek,
            rate: rate_change,
            close,
            volume,
            repeat,
        } = self.speech_panel(ctx, &snapshot);
        if close {
            self.stop_speech();
            return;
        }
        let action = if stop {
            Some((self.speaker.stop(), DiagnosticEvent::Stopped))
        } else if toggle {
            Some((
                if snapshot.playing {
                    self.speaker.pause()
                } else {
                    self.speaker.resume()
                },
                if snapshot.playing {
                    DiagnosticEvent::Paused
                } else {
                    DiagnosticEvent::Resumed
                },
            ))
        } else if back || forward {
            Some((
                self.speaker.seek_relative(if back { -5.0 } else { 5.0 }),
                DiagnosticEvent::Seeked,
            ))
        } else {
            seek.map(|position| (self.speaker.seek(position), DiagnosticEvent::Seeked))
        };
        if let Some((result, event)) = action {
            match result {
                Ok(()) => {
                    if let Some(operation) = &self.speech_operation {
                        operation.event(DiagnosticStage::Play, event);
                    }
                }
                Err(error) => {
                    if let Some(operation) = &self.speech_operation {
                        operation.fail(DiagnosticStage::Play, DiagnosticError::Unavailable);
                    }
                    self.message = error;
                }
            }
        }
        if let Some(rate) = rate_change {
            if let Err(error) = self.change_speech_rate(rate) {
                self.message = error;
            }
        }
        if let Some(volume) = volume {
            if let Err(error) = self.change_speech_volume(volume) {
                self.message = error;
            }
        }
        if !stop {
            if let Some(repeat) = repeat {
                if let Err(error) = self.change_speech_repeat(repeat) {
                    self.message = error;
                }
            }
        }
    }

    pub(super) fn recording_controls(&mut self, ui: &mut egui::Ui, finish_label: &str) {
        let Some(recorder) = &self.recorder else {
            return;
        };
        let paused = recorder.is_paused();
        ui.label(format!(
            "{} {:.1}秒 / 最大30秒",
            if paused {
                "録音一時停止中"
            } else {
                "録音中"
            },
            recorder.elapsed().as_secs_f32()
        ));
        ui.label("入力音量");
        ui.add(egui::ProgressBar::new(if paused { 0.0 } else { recorder.level() })
            .desired_height(22.0));
        let (mut toggle, mut finish) = (false, false);
        ui.horizontal_wrapped(|ui| {
            toggle = ui
                .ww_button(if paused {
                    "録音を再開"
                } else {
                    "録音を一時停止"
                })
                .clicked();
            finish = ui.ww_button(finish_label).clicked();
            if ui.ww_button("録音をキャンセル…").clicked() {
                self.recording_cancel_confirm = true;
            }
        });
        if toggle {
            if let Some(recorder) = self.recorder.as_mut() {
                let result = if paused {
                    recorder.resume()
                } else {
                    recorder.pause()
                };
                if let Some(operation) = &self.recording_operation {
                    if result.is_ok() {
                        operation.event(
                            DiagnosticStage::Record,
                            if paused {
                                DiagnosticEvent::Resumed
                            } else {
                                DiagnosticEvent::Paused
                            },
                        );
                    } else {
                        operation.fail(DiagnosticStage::Record, DiagnosticError::Unavailable);
                    }
                }
                self.message = match result {
                    Ok(()) => if paused {
                        "録音を再開した。"
                    } else {
                        "録音を一時停止した。停止時間は録音に含めない。"
                    }
                    .into(),
                    Err(error) => error,
                };
            }
        }
        if finish {
            self.stop_recording();
        }
    }

    pub(super) fn cancel_recording(&mut self) {
        if let Some(recorder) = self.recorder.take() {
            recorder.cancel();
        }
        if let Some(operation) = self.recording_operation.take() {
            operation.event(DiagnosticStage::Record, DiagnosticEvent::Cancelled);
        }
        self.chat_recording_id = None;
        self.recording_cancel_confirm = false;
        self.message = "今回の録音をキャンセルした。保存済みの録音は削除していない。".into();
    }

    pub(super) fn recording_cancel_dialog(&mut self, ctx: &egui::Context) {
        if self.recorder.is_none() {
            self.recording_cancel_confirm = false;
        }
        if !self.recording_cancel_confirm {
            return;
        }
        let (mut discard, mut keep) = (false, false);
        egui::Window::new("録音のキャンセルを確認")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ux::dialog_body(ui);
                ui.label("今回収録している音声を保存せず破棄する。この操作は取り消せない。");
                ui.label("過去に保存した録音・添付・教材は変更しない。");
                ui.horizontal(|ui| {
                    discard = ui.ww_button("今回の録音を破棄").clicked();
                    keep = ui.ww_button("録音に戻る").clicked();
                });
            });
        if discard {
            self.cancel_recording();
        }
        if keep {
            self.recording_cancel_confirm = false;
        }
    }
}
