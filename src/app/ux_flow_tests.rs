use super::harness_tests::{fixture, frame, shape_text};
use super::*;

#[test]
fn short_course_respects_a_zero_daily_new_limit() {
    let (_, mut app, root) = fixture();
    app.progress.settings.new_per_day = 0;
    app.start(2);
    assert!(
        app.current.is_none(),
        "two-minute course must not bypass zero-new setting"
    );
    assert!(app.session.is_none());
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

fn study_frame(
    ctx: &egui::Context,
    app: &mut WordApp,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1120.0, 2200.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.study(ui));
        },
    )
}

fn click_study_button(ctx: &egui::Context, app: &mut WordApp, label: &str) {
    study_frame(ctx, app, vec![]);
    let output = study_frame(ctx, app, vec![]);
    let position = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                Some(text.pos + text.galley.size() * 0.5)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("Button was not drawn: {label}"));
    for pressed in [true, false] {
        study_frame(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
}

fn make_review_due(app: &mut WordApp, index: usize) -> String {
    let key = Skill::Recall.key(&app.deck[index].id);
    app.progress.record(
        key.clone(),
        Grade::Good,
        false,
        "keyboard",
        now() - 172800,
        "2000-01-01",
        600,
        false,
    );
    key
}

fn check_current_answer(app: &mut WordApp, matching: bool) {
    let task = app.current.as_ref().unwrap().clone();
    let entry = app.deck[task.index].clone();
    app.answer = if matching {
        entry.answer().to_owned()
    } else {
        "a different natural answer".into()
    };
    app.check_study_answer(&entry, task.skill);
    assert!(app.revealed);
    assert_eq!(app.matched, Some(matching));
}

#[test]
fn starting_from_home_resumes_without_discarding_the_current_answer() {
    let (_, mut app, root) = fixture();
    app.start(5);
    app.answer = "keep my current answer".into();
    let key = app.key();
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(41);
    app.page = Page::Home;
    app.start(5);
    assert_eq!(app.answer, "keep my current answer");
    assert_eq!(app.key(), key);
    assert_eq!(
        app.session.as_ref().unwrap().elapsed,
        Duration::from_secs(41)
    );
    assert!(app.page == Page::Study);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn finishing_presents_a_summary_and_never_double_credits_time() {
    let (ctx, mut app, root) = fixture();
    app.start(5);
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(65);
    app.finish();
    assert!(
        app.page == Page::Study,
        "finish must show a result, not return silently home"
    );
    assert!(app.session.is_none());
    assert_eq!(app.progress.study_seconds.get(&today()), Some(&65));
    app.finish();
    assert_eq!(app.progress.study_seconds.get(&today()), Some(&65));
    frame(&ctx, &mut app, false);
    let out = frame(&ctx, &mut app, false);
    let mut text = String::new();
    for shape in out.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(text.contains("今回の学習"), "{text}");
    assert!(text.contains("今日はここで終了"), "{text}");
    assert!(text.contains("もう少し学習する"), "{text}");
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_finish_keeps_session_for_export_instead_of_claiming_success() {
    let (_, mut app, root) = fixture();
    app.start(5);
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(65);
    let path = app.storage.as_ref().unwrap().dir.join("progress.json");
    std::fs::rename(&path, path.with_extension("saved")).unwrap();
    std::fs::create_dir(&path).unwrap();
    app.finish();
    assert!(app.fatal.is_some());
    assert!(
        app.session.is_some(),
        "retain session when persistence failed"
    );
    assert_eq!(app.progress.study_seconds.get(&today()), Some(&65));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reaching_the_time_budget_keeps_the_answer_until_an_explicit_grade() {
    let (ctx, mut app, root) = fixture();
    app.progress.settings.skills = vec![Skill::Recall];
    app.progress.settings.new_per_day = 0;
    let key = make_review_due(&mut app, 0);
    app.start(5);
    app.answer = app.deck[0].answer().to_owned();
    app.wav = Some(vec![1, 2, 3]);
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(301);
    frame(&ctx, &mut app, false);
    assert_eq!(app.key(), key);
    assert_eq!(app.answer, app.deck[0].answer());
    assert_eq!(app.wav, Some(vec![1, 2, 3]));
    assert!(app.session_summary.is_none());
    assert_eq!(
        app.progress.reviews.len(),
        1,
        "time alone must not record an answer"
    );
    let elapsed = app.session.as_ref().unwrap().elapsed.as_secs();
    assert!(elapsed >= 301);

    click_study_button(&ctx, &mut app, "回答を確認");
    assert!(app.revealed);
    assert_eq!(app.progress.reviews.len(), 1, "checking is not a grade");
    click_study_button(&ctx, &mut app, "自力でできた → 次へ");
    assert!(app.session.is_none());
    assert!(app.current.is_none());
    assert_eq!(app.progress.reviews.len(), 2);
    assert_eq!(app.progress.reviews.last().unwrap().key, key);
    assert_eq!(app.progress.reviews.last().unwrap().grade, Grade::Good);
    let summary = app.session_summary.as_ref().unwrap();
    assert_eq!(summary.reason, session_end::EndReason::Time);
    assert_eq!(summary.answers, 1);
    assert_eq!(summary.independent, 1);
    assert_eq!(summary.reviewed, 1);
    assert_eq!(summary.seconds, elapsed);
    frame(&ctx, &mut app, false);
    assert_eq!(
        app.progress.reviews.len(),
        2,
        "redrawing the result must not record again"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn continuing_respects_the_daily_new_limit_and_existing_review_due_dates() {
    let (ctx, mut app, root) = fixture();
    app.progress.settings.skills = vec![Skill::Recall];
    app.progress.settings.new_per_day = 1;
    app.start(5);
    let new_key = app.key();
    assert!(app.current.as_ref().unwrap().introduce);
    click_study_button(&ctx, &mut app, "確認した・別の問題のあとで思い出す");
    assert_eq!(app.key(), new_key);
    assert!(!app.current.as_ref().unwrap().introduce);
    check_current_answer(&mut app, true);
    click_study_button(&ctx, &mut app, "自力でできた → 次へ");
    assert!(app.session.is_none());
    assert_eq!(app.progress.reviews.len(), 1);
    assert!(app.progress.reviews[0].first);
    assert!(
        app.progress.reviews[0].assisted,
        "immediate recall after introduction stays assisted"
    );
    assert_eq!(app.progress.reviews[0].grade, Grade::Hard);
    assert_eq!(app.session_summary.as_ref().unwrap().reviewed, 0);
    let new_due = app.progress.memories[&new_key].due;
    let saved = serde_json::to_value(&app.progress).unwrap();
    click_study_button(&ctx, &mut app, "もう少し学習する");
    assert!(
        app.session.is_none(),
        "continuation is disabled when no eligible tasks remain"
    );
    assert_eq!(serde_json::to_value(&app.progress).unwrap(), saved);

    let due_key = make_review_due(&mut app, 1);
    click_study_button(&ctx, &mut app, "もう少し学習する");
    assert!(app.session_summary.is_none());
    assert_eq!(app.key(), due_key);
    assert!(!app.current.as_ref().unwrap().introduce);
    assert!(
        app.session.as_ref().unwrap().queue.is_empty(),
        "the consumed new allowance must not reset"
    );
    assert_eq!(app.progress.memories[&new_key].due, new_due);
    check_current_answer(&mut app, true);
    click_study_button(&ctx, &mut app, "自力でできた → 次へ");
    let summary = app.session_summary.as_ref().unwrap();
    assert_eq!(
        summary.answers, 1,
        "the second result excludes the first session"
    );
    assert_eq!(summary.reviewed, 1);
    assert_eq!(summary.reason, session_end::EndReason::NoTasks);
    assert_eq!(
        app.progress
            .reviews
            .iter()
            .filter(|review| review.date == today() && review.first)
            .count(),
        1
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn short_default_course_disables_empty_continuation_and_keeps_the_result() {
    for minutes in [1, 2] {
        let (ctx, mut app, root) = fixture();
        app.progress.settings.minutes = minutes;
        app.progress.settings.skills = vec![Skill::Recall];
        app.progress.settings.new_per_day = 3;
        app.start(minutes);
        click_study_button(&ctx, &mut app, "確認した・別の問題のあとで思い出す");
        check_current_answer(&mut app, true);
        click_study_button(&ctx, &mut app, "自力でできた → 次へ");
        assert!(app.session.is_none());
        assert_eq!(app.session_summary.as_ref().unwrap().answers, 1);
        let saved = serde_json::to_value(&app.progress).unwrap();
        click_study_button(&ctx, &mut app, "もう少し学習する");
        assert!(app.session.is_none());
        assert_eq!(
            app.session_summary.as_ref().unwrap().answers,
            1,
            "disabled continuation must preserve the previous result at {minutes} minutes"
        );
        assert_eq!(serde_json::to_value(&app.progress).unwrap(), saved);

        // A due review remains eligible even after the short-course new allowance is used.
        let due_key = make_review_due(&mut app, 1);
        click_study_button(&ctx, &mut app, "もう少し学習する");
        assert_eq!(app.key(), due_key);
        assert!(!app.current.as_ref().unwrap().introduce);
        assert!(app.session.as_ref().unwrap().queue.is_empty());
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn zero_answer_manual_end_and_no_eligible_tasks_have_distinct_results() {
    let (ctx, mut app, root) = fixture();
    app.start(5);
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(23);
    click_study_button(&ctx, &mut app, "ここで終える");
    let summary = app.session_summary.as_ref().unwrap();
    assert_eq!(summary.reason, session_end::EndReason::Manual);
    assert_eq!(summary.seconds, 23);
    assert_eq!(
        (summary.answers, summary.independent, summary.reviewed),
        (0, 0, 0)
    );
    assert!(summary.expressions.is_empty());
    assert!(summary.next_due.is_none());
    let output = study_frame(&ctx, &mut app, vec![]);
    let mut text = String::new();
    for shape in output.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(text.contains("まだ回答は記録していない"), "{text}");

    app.progress.settings.new_per_day = 0;
    app.start(5);
    assert!(app.session.is_none());
    assert!(app.current.is_none());
    let summary = app.session_summary.as_ref().unwrap();
    assert_eq!(summary.reason, session_end::EndReason::NoTasks);
    assert_eq!(summary.seconds, 0);
    assert_eq!(summary.answers, 0);
    assert!(app.progress.reviews.is_empty());
    assert_eq!(app.progress.study_seconds.get(&today()), Some(&23));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn summary_counts_session_answers_unique_reviews_and_self_assessed_success() {
    let (ctx, mut app, root) = fixture();
    app.progress.settings.skills = vec![Skill::Recall];
    app.progress.settings.new_per_day = 1;
    let repeated_key = make_review_due(&mut app, 0);
    make_review_due(&mut app, 1);
    app.start(5);
    assert_eq!(app.key(), repeated_key);
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(73);
    app.card_start = Instant::now() - Duration::from_secs(90);
    check_current_answer(&mut app, true);
    app.grade(Grade::Good);
    check_current_answer(&mut app, false);
    app.grade(Grade::Good);

    assert!(app.current.as_ref().unwrap().introduce);
    let new_id = app.deck[app.current.as_ref().unwrap().index].id.clone();
    app.progress.memories.get_mut(&repeated_key).unwrap().due = now() - 1;
    app.session.as_mut().unwrap().queue.push_back(Task {
        index: 0,
        skill: Skill::Recall,
        introduce: false,
    });
    click_study_button(&ctx, &mut app, "確認した・別の問題のあとで思い出す");
    assert_eq!(app.key(), repeated_key);
    check_current_answer(&mut app, true);
    app.grade(Grade::Good);
    assert_eq!(app.key(), Skill::Recall.key(&new_id));
    check_current_answer(&mut app, true);
    app.grade(Grade::Easy);

    let records = &app.progress.reviews[2..];
    assert_eq!(records.len(), 4);
    assert!(!records[0].self_assessed);
    assert!(records[1].self_assessed);
    assert!(!records[1].assisted);
    assert!(records[3].first && records[3].assisted);
    assert_eq!(records[3].grade, Grade::Hard);
    assert!(records.iter().map(|review| review.seconds).sum::<u32>() >= 90);
    let summary = app.session_summary.as_ref().unwrap();
    assert_eq!(
        summary.answers, 4,
        "historical records are outside this session"
    );
    assert_eq!(
        summary.independent, 3,
        "self-assessed success counts; assisted success does not"
    );
    assert_eq!(
        summary.reviewed, 2,
        "repeated reviews count once and the first answer is excluded"
    );
    assert_eq!(summary.expressions.len(), 3);
    assert_eq!(
        summary.seconds, 73,
        "active study time is not the sum of Review.seconds"
    );
    assert_eq!(
        summary.next_due,
        app.progress
            .memories
            .values()
            .map(|memory| memory.due)
            .min()
    );
    assert_eq!(app.progress.study_seconds.get(&today()), Some(&73));
    let output = study_frame(&ctx, &mut app, vec![]);
    let mut text = String::new();
    for shape in output.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(
        text.contains("ヒントなしでできたと記録：3回（自己評価を含む）"),
        "{text}"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn end_confirmation_pauses_time_and_can_return_to_the_unrecorded_answer() {
    let (ctx, mut app, root) = fixture();
    app.progress.settings.skills = vec![Skill::Recall];
    app.progress.settings.new_per_day = 0;
    let key = make_review_due(&mut app, 0);
    app.start(5);
    app.answer = "keep this unrecorded answer".into();
    app.wav = Some(vec![4, 5, 6]);
    app.session.as_mut().unwrap().elapsed = Duration::from_secs(17);
    click_study_button(&ctx, &mut app, "ここで終える");
    assert!(app.study_end_confirm);
    app.last_frame = Instant::now() - Duration::from_secs(2);
    frame(&ctx, &mut app, false);
    assert_eq!(
        app.session.as_ref().unwrap().elapsed,
        Duration::from_secs(17)
    );
    assert_eq!(app.key(), key);
    assert_eq!(app.progress.reviews.len(), 1);

    click_study_button(&ctx, &mut app, "問題に戻る");
    assert!(!app.study_end_confirm);
    assert_eq!(app.answer, "keep this unrecorded answer");
    assert_eq!(app.wav, Some(vec![4, 5, 6]));
    app.last_frame = Instant::now() - Duration::from_secs(2);
    frame(&ctx, &mut app, false);
    assert_eq!(
        app.session.as_ref().unwrap().elapsed,
        Duration::from_secs(18)
    );
    click_study_button(&ctx, &mut app, "ここで終える");
    click_study_button(&ctx, &mut app, "未記録の回答を破棄して終了");
    assert!(app.session.is_none());
    assert_eq!(
        app.progress.reviews.len(),
        1,
        "discarding must not fabricate a grade"
    );
    assert_eq!(app.session_summary.as_ref().unwrap().answers, 0);
    assert_eq!(app.session_summary.as_ref().unwrap().seconds, 18);
    assert_eq!(app.progress.study_seconds.get(&today()), Some(&18));
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
