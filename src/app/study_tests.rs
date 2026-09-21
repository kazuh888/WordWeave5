use super::*;

fn frame(ctx: &egui::Context, app: &mut WordApp, mut events: Vec<egui::Event>) {
    // Ordinary test presses include key-up, as separate real keystrokes do.
    let releases: Vec<_> = events.iter().filter_map(|event| match event {
        egui::Event::Key { key, physical_key, pressed: true, repeat: false, modifiers } =>
            Some(egui::Event::Key { key: *key, physical_key: *physical_key,
                pressed: false, repeat: false, modifiers: *modifiers }),
        _ => None,
    }).collect();
    events.extend(releases);
    let _ = ctx.run(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1120.0, 850.0))),
        events,
        ..Default::default()
    }, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| app.study(ui));
    });
}

fn question(app: &mut WordApp, skill: Skill) {
    app.start(5);
    app.current = Some(Task { index: 0, skill, introduce: false });
    app.reset_answer();
    app.answer = "answer".into();
}

fn enter(ctrl: bool, repeat: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Enter, physical_key: None, pressed: true, repeat,
        modifiers: if ctrl { egui::Modifiers::CTRL } else { egui::Modifiers::NONE },
    }
}

#[test]
fn study_answer_gets_focus_once_without_stealing_it_back() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Recall);
    frame(&ctx, &mut app, vec![]);
    assert!(ctx.memory(|m| m.has_focus(egui::Id::new("study-answer-input"))));
    ctx.memory_mut(|m| m.surrender_focus(egui::Id::new("study-answer-input")));
    frame(&ctx, &mut app, vec![]);
    assert!(!ctx.memory(|m| m.has_focus(egui::Id::new("study-answer-input"))));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_recall_enter_checks_but_cannot_record_a_self_grade() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Recall);
    frame(&ctx, &mut app, vec![]);
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(app.revealed);
    assert_eq!(app.answer, "answer");
    frame(&ctx, &mut app, vec![enter(false, true)]);
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(app.progress.reviews.is_empty(), "Enter must not choose a grade");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_multiline_enter_adds_a_line_and_ctrl_enter_checks() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Usage);
    frame(&ctx, &mut app, vec![]);
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(!app.revealed);
    assert_eq!(app.answer.matches('\n').count(), 1);
    frame(&ctx, &mut app, vec![enter(true, false)]);
    assert!(app.revealed);
    assert_eq!(app.answer.matches('\n').count(), 1);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_ime_confirmation_does_not_check_or_insert_newline() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Usage);
    frame(&ctx, &mut app, vec![]);
    frame(&ctx, &mut app, vec![egui::Event::Ime(egui::ImeEvent::Commit("漢字".into())), enter(false, false)]);
    assert!(!app.revealed);
    assert!(!app.answer.contains('\n'));
    frame(&ctx, &mut app, vec![enter(true, false)]);
    assert!(app.revealed);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_paused_or_pending_does_not_check_an_answer() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Recall);
    frame(&ctx, &mut app, vec![]);
    app.session.as_mut().unwrap().paused = true;
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(!app.revealed);
    app.session.as_mut().unwrap().paused = false;
    let (_tx, rx) = mpsc::channel();
    app.pending = Some(Pending { key: app.key(), rx, cancel: None, kind: Activity::Study });
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(!app.revealed);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_empty_enter_and_fatal_state_do_not_reveal_or_record() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Recall);
    app.answer.clear();
    frame(&ctx, &mut app, vec![]);
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(!app.revealed);
    app.answer = "answer".into();
    app.fatal = Some("save failed".into());
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert!(!app.revealed);
    assert!(app.progress.reviews.is_empty());
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn study_tab_then_enter_records_an_explicitly_focused_grade() {
    let (ctx, mut app, root) = harness_tests::fixture();
    question(&mut app, Skill::Recall);
    frame(&ctx, &mut app, vec![]);
    frame(&ctx, &mut app, vec![enter(false, false)]);
    frame(&ctx, &mut app, vec![]);
    frame(&ctx, &mut app, vec![egui::Event::Key {
        key: egui::Key::Tab, physical_key: None, pressed: true, repeat: false,
        modifiers: egui::Modifiers::NONE,
    }]);
    frame(&ctx, &mut app, vec![enter(false, false)]);
    assert_eq!(app.progress.reviews.len(), 1);
    assert_eq!(app.progress.reviews[0].grade, Grade::Again);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
