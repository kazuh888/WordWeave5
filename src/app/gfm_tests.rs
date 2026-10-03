//! U6 acceptance tests. Inputs and expected text come from gfm-spec G01–G08.
use super::*;

fn draw(ctx: &egui::Context, app: &mut WordApp, events: Vec<egui::Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 1000.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| app.update_ui(ctx),
    )
}

fn painted(output: &egui::FullOutput) -> String {
    let mut text = String::new();
    for shape in &output.shapes {
        super::harness_tests::shape_text(&shape.shape, &mut text);
    }
    text
}

fn painted_in(output: &egui::FullOutput, area: egui::Rect) -> String {
    fn collect(shape: &egui::Shape, area: egui::Rect, text: &mut String) {
        match shape {
            egui::Shape::Text(glyphs) => {
                let rect = egui::Rect::from_min_size(glyphs.pos, glyphs.galley.size());
                if area.contains(rect.center()) {
                    text.push_str(glyphs.galley.text());
                    text.push('\n');
                }
            }
            egui::Shape::Vec(parts) => {
                for part in parts {
                    collect(part, area, text);
                }
            }
            _ => {}
        }
    }
    let mut text = String::new();
    for shape in &output.shapes {
        collect(&shape.shape, area, &mut text);
    }
    text
}

fn visible_label(output: &egui::FullOutput, label: &str) -> Option<egui::Rect> {
    fn find(shape: &egui::Shape, label: &str) -> Option<egui::Rect> {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == label => {
                Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
            }
            egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, label)),
            _ => None,
        }
    }
    output.shapes.iter().find_map(|shape| {
        let rect = find(&shape.shape, label)?;
        shape.clip_rect.contains_rect(rect).then_some(rect)
    })
}

fn click(ctx: &egui::Context, app: &mut WordApp, pos: egui::Pos2) -> egui::FullOutput {
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    draw(ctx, app, vec![egui::Event::PointerMoved(pos), button(true)]);
    draw(
        ctx,
        app,
        vec![egui::Event::PointerMoved(pos), button(false)],
    )
}

fn visible_chat_copy(ctx: &egui::Context, app: &mut WordApp, id: egui::Id) -> egui::Rect {
    for _ in 0..20 {
        draw(ctx, app, vec![]);
        let rect = ctx
            .data(|data| data.get_temp::<egui::Rect>(id))
            .expect("chat copy control exists");
        let clip = ctx
            .data(|data| data.get_temp::<egui::Rect>(id.with("clip")))
            .expect("chat copy clip exists");
        if clip.contains(rect.center()) {
            return rect;
        }
        let direction = if rect.center().y < clip.top() {
            120.0
        } else {
            -120.0
        };
        draw(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, direction),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    panic!("chat copy control never became visible by transcript scrolling");
}

fn visible_text_containing(output: &egui::FullOutput, fragment: &str) -> Option<egui::Rect> {
    fn find(shape: &egui::Shape, fragment: &str) -> Option<egui::Rect> {
        match shape {
            egui::Shape::Text(text) if text.galley.text().contains(fragment) => {
                Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
            }
            egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, fragment)),
            _ => None,
        }
    }
    output.shapes.iter().find_map(|shape| {
        let rect = find(&shape.shape, fragment)?;
        shape.clip_rect.contains_rect(rect).then_some(rect)
    })
}

fn visible_glyph_position(output: &egui::FullOutput, fragment: &str) -> Option<egui::Pos2> {
    fn find(shape: &egui::Shape, clip: egui::Rect, fragment: &str) -> Option<egui::Pos2> {
        match shape {
            egui::Shape::Text(text) => text.galley.rows.iter().find_map(|row| {
                let content: String = row.glyphs.iter().map(|glyph| glyph.chr).collect();
                let byte_index = content.find(fragment)?;
                let glyph_index = content[..byte_index].chars().count();
                let point = text.pos + row.glyphs[glyph_index].pos.to_vec2();
                clip.contains(point).then_some(point)
            }),
            egui::Shape::Vec(parts) => parts.iter().find_map(|part| find(part, clip, fragment)),
            _ => None,
        }
    }
    output
        .shapes
        .iter()
        .find_map(|shape| find(&shape.shape, shape.clip_rect, fragment))
}

fn painted_sections(output: &egui::FullOutput) -> Vec<(String, egui::text::TextFormat)> {
    fn collect(shape: &egui::Shape, result: &mut Vec<(String, egui::text::TextFormat)>) {
        match shape {
            egui::Shape::Text(text) => {
                for section in &text.galley.job.sections {
                    if let Some(content) = text.galley.job.text.get(section.byte_range.clone()) {
                        result.push((content.to_owned(), section.format.clone()));
                    }
                }
            }
            egui::Shape::Vec(parts) => {
                for part in parts {
                    collect(part, result);
                }
            }
            _ => {}
        }
    }
    let mut result = Vec::new();
    for shape in &output.shapes {
        collect(&shape.shape, &mut result);
    }
    result
}

fn register(ctx: &egui::Context, app: &mut WordApp) {
    let mut output = draw(ctx, app, vec![]);
    for _ in 0..4 {
        output = draw(ctx, app, vec![]);
    }
    let button = visible_label(&output, "内容を確認して教材に登録")
        .expect("approved proposal must expose the registration action");
    click(ctx, app, button.center());
}

#[test]
fn gfm_chat_decorates_ai_only_and_copies_the_exact_saved_answer() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    app.page = Page::Chat;
    let original = " **重要**\r\n二行目\\n\t🦊 ";
    app.progress.chats[0]
        .complete(
            original.into(),
            original.into(),
            wordweave5::execution::Execution::default(),
        )
        .unwrap();
    let chat_id = app.progress.chats[0].id.clone();
    let before = serde_json::to_vec(&app.progress.chats[0]).unwrap();
    let mut output = draw(&ctx, &mut app, vec![]);
    for _ in 0..3 {
        output = draw(&ctx, &mut app, vec![]);
    }
    let user_id = egui::Id::new(("chat-copy-question", &chat_id, 0));
    let copy_id = egui::Id::new(("chat-copy-answer", &chat_id, 0));
    let user_bubble = ctx
        .data(|data| data.get_temp::<egui::Rect>(user_id.with("bubble")))
        .expect("user bubble in real transcript");
    let ai_bubble = ctx
        .data(|data| data.get_temp::<egui::Rect>(copy_id.with("bubble")))
        .expect("AI bubble in real transcript");
    let user_text = painted_in(&output, user_bubble);
    let ai_text = painted_in(&output, ai_bubble);
    assert!(
        user_text.contains("**重要**"),
        "user text must stay literal: {user_text}"
    );
    assert!(
        ai_text.contains("重要") && !ai_text.contains("**重要**"),
        "saved AI answer must be decorated: {ai_text}"
    );
    let rect = visible_chat_copy(&ctx, &mut app, copy_id);
    let copied = click(&ctx, &mut app, rect.center());
    assert!(
        copied.platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if text == original)
        ),
        "AI copy must emit original CRLF, whitespace, tab, literal slash-n and Unicode"
    );
    assert_eq!(serde_json::to_vec(&app.progress.chats[0]).unwrap(), before);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_unsafe_markup_is_readable_but_does_not_open_external_targets() {
    let (ctx, app, root) = super::harness_tests::fixture();
    let source = "[ローカル](file:///synthetic/only)\n\n![代替](https://invalid.example/never.png) <script>alert(1)</script>";
    let render = |events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 500.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    notifications::show_markdown(ui, egui::Id::new("gfm-unsafe-fixture"), source);
                });
            },
        )
    };
    let output = render(vec![]);
    let text = painted(&output);
    for fragment in [
        "ローカル",
        "代替",
        "https://invalid.example/never.png",
        "<script>alert(1)</script>",
    ] {
        assert!(
            text.contains(fragment),
            "safe display dropped {fragment:?}: {text}"
        );
    }
    assert!(!output
        .platform_output
        .commands
        .iter()
        .any(|command| matches!(command, egui::OutputCommand::OpenUrl(_))));
    let rect = visible_text_containing(&output, "ローカル").expect("link text is visible");
    let pos = rect.center();
    render(vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    let clicked = render(vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    assert!(
        !clicked
            .platform_output
            .commands
            .iter()
            .any(|command| matches!(command, egui::OutputCommand::OpenUrl(_))),
        "rendered link must be non-operative"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_table_has_aligned_header_and_body_cells_in_the_rendered_ui() {
    let (ctx, app, root) = super::harness_tests::fixture();
    let source = "| 列一 | 列二 |\n| --- | --- |\n| 内容甲 | 内容乙 |";
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 500.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                notifications::show_markdown(ui, egui::Id::new("gfm-grid-fixture"), source);
            });
        },
    );
    let text = painted(&output);
    assert!(
        !text.contains("---"),
        "delimiter must not be displayed as a text row: {text}"
    );
    let left_header = visible_label(&output, "列一").expect("first header cell visible");
    let right_header = visible_label(&output, "列二").expect("second header cell visible");
    let left_body = visible_label(&output, "内容甲").expect("first body cell visible");
    let right_body = visible_label(&output, "内容乙").expect("second body cell visible");
    assert!((left_header.center().y - right_header.center().y).abs() < 5.0);
    assert!((left_body.center().y - right_body.center().y).abs() < 5.0);
    assert!(
        left_body.top() > left_header.bottom(),
        "body must follow the header row"
    );
    assert!(
        right_header.left() > left_header.left() && right_body.left() > left_body.left(),
        "columns must be spatially distinct"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_wrapped_table_row_keeps_following_row_below_every_cell() {
    let long_cell = "合成の長いセルを折り返して読む。".repeat(3);
    let source = format!(
        "| 左列 | 右列 |\n| --- | --- |\n| {long_cell} | 一行目右 |\n| 二行目左 | 二行目右 |"
    );
    for (case, window, zoom) in [
        ("standard", egui::vec2(1150.0, 950.0), 0.8),
        ("large_font", egui::vec2(1150.0, 950.0), 1.6),
        ("narrow", egui::vec2(820.0, 650.0), 1.6),
    ] {
        let (ctx, app, root) = super::harness_tests::fixture();
        ctx.set_zoom_factor(zoom);
        let mut output = None;
        for _ in 0..3 {
            output = Some(ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, window / zoom)),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        notifications::show_markdown(
                            ui,
                            egui::Id::new(("gfm-wrapped-grid", case)),
                            &source,
                        );
                    });
                },
            ));
        }
        let output = output.expect("multiple layout frames were rendered");
        let left_header = visible_label(&output, "左列").expect("left header visible");
        let right_header = visible_label(&output, "右列").expect("right header visible");
        let first_left = visible_label(&output, &long_cell).unwrap_or_else(|| {
            panic!(
                "{case}: wrapped left cell visible; painted={:?}",
                painted(&output)
            )
        });
        let first_right = visible_label(&output, "一行目右").expect("first right cell visible");
        let second_left = visible_label(&output, "二行目左").expect("second left cell visible");
        let second_right = visible_label(&output, "二行目右").expect("second right cell visible");

        assert!(
            first_left.height() > first_right.height() * 1.5,
            "{case}: fixture must actually wrap the long cell: {first_left:?}, {first_right:?}"
        );
        assert!(
            first_left.bottom().max(first_right.bottom())
                <= second_left.top().min(second_right.top()),
            "{case}: following row overlaps a cell above: {first_left:?}, {first_right:?}, {second_left:?}, {second_right:?}"
        );
        assert!(
            left_header.left() < right_header.left()
                && first_left.left() < first_right.left()
                && second_left.left() < second_right.left(),
            "{case}: both columns must retain their order"
        );
        assert!(
            (second_left.top() - second_right.top()).abs() < 5.0,
            "{case}: next-row cells must align vertically"
        );
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn gfm_tight_list_keeps_inline_styles_and_nested_child_paragraph() {
    let (ctx, app, root) = super::harness_tests::fixture();
    let source = "- 親 **重要**と*斜体*、`code`\n  - 子項目\n\n    子段落";
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 600.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                notifications::show_markdown(ui, egui::Id::new("gfm-nested-list"), source);
            });
        },
    );
    let text = painted(&output);
    for fragment in ["親", "重要", "斜体", "code", "子項目", "子段落"] {
        assert!(
            text.contains(fragment),
            "list content lost {fragment:?}: {text}"
        );
    }
    let sections = painted_sections(&output);
    assert!(
        sections.iter().any(|(part, style)| part.contains("重要")
            && style.font_id.family == egui::FontFamily::Name("heading".into())),
        "strong text inside tight item must use the real bold family: {sections:?}"
    );
    assert!(
        sections
            .iter()
            .any(|(part, style)| part.contains("斜体") && style.italics),
        "emphasis inside tight item must retain italic styling: {sections:?}"
    );
    assert!(
        sections
            .iter()
            .any(|(part, style)| part.contains("code")
                && style.background != egui::Color32::TRANSPARENT),
        "inline code inside tight item must retain code styling: {sections:?}"
    );
    let parent = visible_glyph_position(&output, "親").expect("parent item visible");
    let child = visible_glyph_position(&output, "子項目").expect("nested item visible");
    let paragraph = visible_glyph_position(&output, "子段落").expect("child paragraph visible");
    assert!(
        child.y > parent.y && paragraph.y > child.y,
        "nested list and paragraph must remain in reading order"
    );
    assert!(
        child.x > parent.x + 5.0 && paragraph.x >= child.x - 2.0,
        "nested list and paragraph must retain indentation: {parent:?} {child:?} {paragraph:?}"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_list_keeps_nested_rule_and_fenced_code_as_distinct_blocks() {
    let (ctx, app, root) = super::harness_tests::fixture();
    let source = "- 親項目\n\n  ---\n\n  ```text\n  **コード内**\n  ```\n\n- 次項目";
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 600.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                notifications::show_markdown(ui, egui::Id::new("gfm-list-blocks"), source);
            });
        },
    );
    let text = painted(&output);
    for fragment in ["親項目", "**コード内**", "次項目"] {
        assert!(
            text.contains(fragment),
            "nested block content lost {fragment:?}: {text}"
        );
    }
    assert!(
        !text.contains("```"),
        "fence markers must not be shown as code content: {text}"
    );
    assert!(
        !text.contains("---"),
        "rule marker must be drawn as a rule: {text}"
    );
    fn has_rule(shape: &egui::Shape) -> bool {
        match shape {
            egui::Shape::LineSegment { points, .. } => (points[1].x - points[0].x).abs() > 40.0,
            egui::Shape::Vec(parts) => parts.iter().any(has_rule),
            _ => false,
        }
    }
    assert!(
        output.shapes.iter().any(|shape| has_rule(&shape.shape)),
        "nested thematic break must produce a visible rule"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn deep_quote_stack_keeps_tail_and_following_table_without_process_crash() {
    let (ctx, app, root) = super::harness_tests::fixture();
    let source = format!(
        "{}深い引用の末尾\n\n| 列 | 値 |\n| --- | --- |\n| 合成 | 表の末尾 |",
        "> ".repeat(7000)
    );
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 800.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                notifications::show_markdown(ui, egui::Id::new("gfm-deep-quote"), &source);
            });
        },
    );
    let text = painted(&output);
    for fragment in ["深い引用の末尾", "列", "値", "表の末尾"] {
        assert!(
            text.contains(fragment),
            "deep source lost {fragment:?}: {}",
            text.chars().take(250).collect::<String>()
        );
    }
    assert!(
        visible_text_containing(&output, "深い引用の末尾").is_some(),
        "deep quote tail must remain inside its visible viewport"
    );
    assert!(
        visible_text_containing(&output, "表の末尾").is_some(),
        "the following table must remain visible after the deep quote"
    );
    assert!(
        !text.contains("| --- | --- |"),
        "following GFM table regressed to literal syntax"
    );
    drop(output);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_registration_receipt_follows_real_approval_and_survives_dialog_close() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    app.page = Page::Chat;
    app.chat_material_open = true;
    app.progress.chats[0]
        .complete(
            "合成質問".into(),
            "合成回答".into(),
            wordweave5::execution::Execution::default(),
        )
        .unwrap();
    let chat_id = app.progress.chats[0].id.clone();
    let draft = super::harness_tests::material_ui_draft(&app);
    let target = draft.candidate.base.clone();
    let expected_entry = draft.candidate.clone();
    let before_calls = app.progress.ai_calls.clone();
    app.progress.material_draft = Some(draft);
    assert!(
        app.material_registration_receipt.is_none(),
        "proposal alone is not registration"
    );
    register(&ctx, &mut app);
    assert!(
        app.progress.material_draft.is_none(),
        "successful approval consumes proposal"
    );
    assert!(app
        .deck
        .iter()
        .any(|entry| entry.id == expected_entry.id && entry.meaning == expected_entry.meaning));
    let disk = std::fs::read_to_string(root.join("data").join("custom.tsv")).unwrap();
    assert!(
        disk.contains(&expected_entry.meaning),
        "registered meaning must be saved"
    );
    assert_eq!(
        app.material_registration_receipt
            .as_ref()
            .map(|r| r.target_base.as_str()),
        Some(target.as_str())
    );
    assert!(!app.notification_open, "success must not open a new dialog");
    let mut output = draw(&ctx, &mut app, vec![]);
    for _ in 0..3 {
        output = draw(&ctx, &mut app, vec![]);
    }
    let text = painted(&output);
    assert!(
        text.contains(&target) && text.contains("登録した"),
        "receipt must remain visible after the proposal disappears: {text}"
    );
    app.chat_material_open = false;
    // egui keeps a closing Window interactive during its fade. Advance the
    // headless frames until that layer no longer covers the transcript.
    let material_layer =
        egui::LayerId::new(egui::Order::Middle, egui::Id::new("教材の根拠と差分を確認"));
    for _ in 0..16 {
        output = draw(&ctx, &mut app, vec![]);
        let copy_id = egui::Id::new(("chat-copy-answer", &chat_id, 0));
        let rect = ctx
            .data(|data| data.get_temp::<egui::Rect>(copy_id))
            .expect("chat copy control exists after dialog close");
        if ctx.layer_id_at(rect.center()) != Some(material_layer) {
            break;
        }
    }
    assert!(
        painted(&output).contains(&target),
        "same receipt must remain visible in chat after dialog closes"
    );
    let copy_id = egui::Id::new(("chat-copy-answer", &chat_id, 0));
    let copy_rect = visible_chat_copy(&ctx, &mut app, copy_id);
    assert_ne!(
        ctx.layer_id_at(copy_rect.center()),
        Some(material_layer),
        "closing material dialog must release the copy control"
    );
    let copied = click(&ctx, &mut app, copy_rect.center());
    assert!(copied.platform_output.commands.iter().any(
        |command| matches!(command, egui::OutputCommand::CopyText(text) if text == "合成回答")
    ));
    assert!(
        app.material_registration_receipt.is_some(),
        "copy must retain receipt"
    );
    output = draw(&ctx, &mut app, vec![]);
    let before_close = serde_json::to_vec(&app.progress).unwrap();
    let saved_deck = wordweave5::model::deck_text(&app.deck);
    let dismiss =
        visible_label(&output, "登録結果を閉じる").expect("receipt has an explicit dismiss action");
    click(&ctx, &mut app, dismiss.center());
    assert!(app.material_registration_receipt.is_none());
    assert_eq!(
        serde_json::to_vec(&app.progress).unwrap(),
        before_close,
        "dismissing the receipt is not registration cancellation"
    );
    assert_eq!(wordweave5::model::deck_text(&app.deck), saved_deck);
    assert_eq!(
        app.progress.ai_calls, before_calls,
        "viewing receipt must not call AI"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_new_and_append_approvals_show_receipt_and_append_keeps_learning_state() {
    use wordweave5::material::Mode;
    for mode in [Mode::New, Mode::Append] {
        let (ctx, mut app, root) = super::harness_tests::fixture();
        app.page = Page::Chat;
        app.chat_material_open = true;
        let mut draft = super::harness_tests::material_ui_draft(&app);
        draft.mode = mode;
        draft.source.mode = mode;
        draft.reasons.clear();
        let old = app.deck[0].clone();
        let memory_key = format!("{}:recall", old.id);
        let before_memory = if mode == Mode::Append {
            let memory = wordweave5::scheduler::Memory {
                due: 300,
                last: 100,
                stability: 4.0,
                reviews: 7,
                lapses: 1,
                cue_chars: 0,
            };
            app.progress.memories.insert(memory_key.clone(), memory);
            app.progress
                .deck_versions
                .insert(old.id.clone(), old.fingerprint());
            draft.candidate = old.clone();
            draft
                .candidate
                .replacements
                .push(wordweave5::model::Replacement {
                    phrase: "synthetic companion".into(),
                    meaning: "合成の言い換え".into(),
                    conditions: "合成の利用条件".into(),
                });
            Some(serde_json::to_vec(app.progress.memories.get(&memory_key).unwrap()).unwrap())
        } else {
            draft.baseline = None;
            draft.candidate.id = "gfm_clarity_001".into();
            draft.candidate.base = "clarity".into();
            draft.candidate.examples = vec![
                wordweave5::model::Example {
                    english: "Clarity helps the team.".into(),
                    japanese: "明瞭さはチームを助ける。".into(),
                    note: "合成".into(),
                },
                wordweave5::model::Example {
                    english: "Write with clarity.".into(),
                    japanese: "明瞭に書く。".into(),
                    note: "合成".into(),
                },
            ];
            draft.source.entry_id = draft.candidate.id.clone();
            None
        };
        draft.generated = Some(draft.candidate.clone());
        assert!(
            draft.ready(&app.deck, false).is_ok(),
            "fixture must be registrable in {mode:?}"
        );
        let expected = draft.candidate.clone();
        app.progress.material_draft = Some(draft);
        register(&ctx, &mut app);
        assert!(
            app.progress.material_draft.is_none(),
            "{mode:?} proposal consumed"
        );
        assert_eq!(
            app.material_registration_receipt
                .as_ref()
                .map(|r| r.target_base.as_str()),
            Some(expected.base.as_str()),
            "{mode:?} needs target-specific success"
        );
        assert!(
            !app.notification_open,
            "{mode:?} success must not open an alert"
        );
        assert!(app.deck.iter().any(|entry| entry.id == expected.id
            && entry.replacements.len() == expected.replacements.len()
            && entry.examples.len() == expected.examples.len()));
        if let Some(before) = before_memory {
            assert_eq!(
                serde_json::to_vec(app.progress.memories.get(&memory_key).unwrap()).unwrap(),
                before,
                "append must preserve existing review state"
            );
            assert_eq!(
                app.deck.iter().find(|e| e.id == old.id).unwrap().meaning,
                old.meaning,
                "append must preserve the existing explanation"
            );
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn gfm_next_accepted_registration_failure_clears_prior_success_and_keeps_proposal() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    app.page = Page::Chat;
    app.chat_material_open = true;
    let draft = super::harness_tests::material_ui_draft(&app);
    let target = draft.candidate.base.clone();
    let expected_draft = serde_json::to_vec(&draft).unwrap();
    let before_deck = wordweave5::model::deck_text(&app.deck);
    app.progress.material_draft = Some(draft);
    app.notify_material_registered("前回の合成語".into());
    app.storage = None;
    register(&ctx, &mut app);
    assert!(
        app.material_registration_receipt.is_none(),
        "the accepted next operation supersedes the earlier success even when saving fails"
    );
    assert!(
        app.notification_open
            && app.message.contains(&target)
            && app.message.contains("保存先がありません"),
        "failure must identify target and cause"
    );
    assert_eq!(
        serde_json::to_vec(app.progress.material_draft.as_ref().unwrap()).unwrap(),
        expected_draft,
        "retry proposal must be retained after known pre-save failure"
    );
    assert_eq!(wordweave5::model::deck_text(&app.deck), before_deck);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gfm_registration_partial_save_never_reports_success_or_erases_recovery_state() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    app.page = Page::Chat;
    app.chat_material_open = true;
    let draft = super::harness_tests::material_ui_draft(&app);
    let target = draft.candidate.base.clone();
    let expected_draft = serde_json::to_vec(&draft).unwrap();
    let old_deck = wordweave5::model::deck_text(&app.deck);
    app.progress.material_draft = Some(draft);
    // The real commit writes both files, then fails to archive its pending intent.
    // A file at `backups` blocks only that last step in this isolated store.
    std::fs::write(root.join("data").join("backups"), b"synthetic blocker").unwrap();
    register(&ctx, &mut app);
    assert!(
        app.material_registration_receipt.is_none(),
        "an Err with pending recovery must not be presented as success"
    );
    assert!(
        app.notification_open
            && app
                .fatal
                .as_ref()
                .is_some_and(|text| text.contains("再起動時に復旧")),
        "pending commit must explain recovery"
    );
    assert!(wordweave5::commit::pending(&root.join("data")));
    assert_eq!(
        wordweave5::model::deck_text(&app.deck),
        old_deck,
        "in-memory deck remains pre-commit until recovery"
    );
    assert_eq!(
        serde_json::to_vec(app.progress.material_draft.as_ref().unwrap()).unwrap(),
        expected_draft,
        "proposal remains available for recovery inspection"
    );
    let output = draw(&ctx, &mut app, vec![]);
    let text = painted(&output);
    assert!(
        text.contains("復旧") && text.contains(&target),
        "failure must identify recovery and its target: {text}"
    );
    assert!(
        !text.contains("の教材を登録した"),
        "partial save must not claim completion"
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

// U7: the proposal's fixed transcript is a separate Markdown entry point.
#[test]
fn review_detail_fixed_ai_answer_uses_markdown_and_preserves_literal_quote_source() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let mut draft = super::harness_tests::material_ui_draft(&app);
    let question = "**USER_LITERAL** before registration";
    let answer = "**AI_DECORATED**\n\n| Left | Right |\n| --- | --- |\n| CELL_A | CELL_B |";
    draft.source.snapshots[0].exchange.question = question.into();
    draft.source.snapshots[0].exchange.answer = answer.into();
    draft.reasons[0].quotes[0].role = "assistant".into();
    draft.reasons[0].quotes[0].quote = "AI_DECORATED".into();
    let original_draft = serde_json::to_vec(&draft).unwrap();
    let original_progress = serde_json::to_vec(&app.progress).unwrap();
    let render = |app: &mut WordApp, events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200.0, 850.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.material_comparison(ui, &draft));
            },
        )
    };
    let mut output = render(&mut app, vec![]);
    let expand = visible_label(&output, "生成時の固定会話を展開")
        .expect("fixed conversation expansion is visible");
    let button = |pressed| egui::Event::PointerButton {
        pos: expand.center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    render(
        &mut app,
        vec![egui::Event::PointerMoved(expand.center()), button(true)],
    );
    render(&mut app, vec![button(false)]);
    output = render(&mut app, vec![]);
    let decorated = painted(&output);
    for fragment in ["USER_LITERAL", "AI_DECORATED", "CELL_A", "CELL_B"] {
        assert!(
            decorated.contains(fragment),
            "fixed transcript lost {fragment}: {decorated}"
        );
    }
    assert!(
        decorated.contains(question),
        "the learner's fixed question stays literal: {decorated}"
    );
    assert!(
        !decorated.contains("**AI_DECORATED**") && !decorated.contains("| --- | --- |"),
        "the AI answer must render Markdown in the fixed transcript: {decorated}"
    );
    let raw = visible_label(&output, "原文と該当引用を確認")
        .expect("AI answer retains a separate original/quote control");
    let raw_button = |pressed| egui::Event::PointerButton {
        pos: raw.center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    render(
        &mut app,
        vec![egui::Event::PointerMoved(raw.center()), raw_button(true)],
    );
    render(&mut app, vec![raw_button(false)]);
    output = render(&mut app, vec![]);
    assert!(
        painted(&output).contains(answer),
        "the exact original answer remains inspectable"
    );
    assert!(
        painted_sections(&output).iter().any(|(text, format)| {
            text.contains("AI_DECORATED")
                && format.background == egui::Color32::from_rgb(255, 237, 168)
        }),
        "the selected quote remains visibly highlighted in the original answer"
    );
    assert_eq!(serde_json::to_vec(&draft).unwrap(), original_draft);
    assert_eq!(
        serde_json::to_vec(&app.progress).unwrap(),
        original_progress
    );
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn review_detail_list_and_detail_wheels_are_independent_and_selection_resets_detail() {
    let (ctx, mut app, root) = super::harness_tests::fixture();
    let mut draft = super::harness_tests::material_ui_draft(&app);
    for index in 0..36 {
        draft.candidate.examples.push(wordweave5::model::Example {
            english: format!("Synthetic example {index}."),
            japanese: format!("合成例文{index}"),
            note: "隔離テスト".into(),
        });
    }
    draft.reasons[0].reason = format!("{}RIGHT_TAIL", "Long synthetic detail text. ".repeat(90));
    draft.generated = Some(draft.candidate.clone());
    let original_draft = serde_json::to_vec(&draft).unwrap();
    let render = |app: &mut WordApp, events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1150.0, 520.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.material_comparison(ui, &draft));
            },
        )
    };
    let mut output = render(&mut app, vec![]);
    for _ in 0..2 {
        output = render(&mut app, vec![]);
    }
    let list_before =
        visible_label(&output, "変更：説明・使い方").expect("second change is visible in the list");
    let detail_before = visible_label(&output, "変更理由・該当引用")
        .expect("selected detail is visible beside the list");
    assert!(
        list_before.right() < detail_before.left(),
        "the comparison uses two side-by-side panes"
    );
    let wheel = |point, distance: f32| {
        vec![
            egui::Event::PointerMoved(point),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -distance),
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    render(&mut app, wheel(list_before.center(), 35.0));
    // Point wheel input is smoothed across frames; compare settled paint positions.
    for _ in 0..30 {
        output = render(&mut app, vec![]);
    }
    let list_after_left = visible_label(&output, "変更：説明・使い方")
        .expect("second change remains visible after a short list wheel");
    let detail_after_left = visible_label(&output, "変更理由・該当引用")
        .expect("detail heading remains visible after list wheel");
    assert!(
        list_after_left.top() < list_before.top() - 5.0,
        "left wheel must move the change list"
    );
    assert!(
        (detail_after_left.top() - detail_before.top()).abs() <= 2.0,
        "left wheel must not move the detail"
    );
    render(&mut app, wheel(detail_after_left.center(), 10.0));
    for _ in 0..30 {
        output = render(&mut app, vec![]);
    }
    let list_after_right = visible_label(&output, "変更：説明・使い方")
        .expect("list row remains visible after detail wheel");
    let detail_after_right = visible_label(&output, "変更理由・該当引用")
        .expect("detail heading remains visible after a short detail wheel");
    assert!(
        detail_after_right.top() < detail_after_left.top() - 5.0,
        "right wheel must move only the detail"
    );
    assert!(
        (list_after_right.top() - list_after_left.top()).abs() <= 2.0,
        "right wheel must not move the change list"
    );
    let right_point = egui::pos2(detail_after_right.center().x, 260.0);
    for _ in 0..24 {
        if visible_label(&output, "変更前").is_none() {
            break;
        }
        render(&mut app, wheel(right_point, 35.0));
        output = render(&mut app, vec![]);
    }
    assert!(
        visible_label(&output, "変更前").is_none(),
        "the old A detail must be genuinely scrolled away before testing reset"
    );
    // Finish the right pane's wheel animation before moving the pointer left.
    for _ in 0..30 {
        output = render(&mut app, vec![]);
    }
    let second = visible_label(&output, "変更：説明・使い方")
        .expect("the second list row must remain visible after right scrolling settles");
    assert!(
        (second.top() - list_after_right.top()).abs() <= 2.0,
        "right scrolling must preserve the list position before selection"
    );
    let select = |app: &mut WordApp, point| {
        let button = |pressed| egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        render(app, vec![egui::Event::PointerMoved(point), button(true)]);
        render(app, vec![button(false)]);
        render(app, vec![])
    };
    output = select(&mut app, second.center());
    assert!(
        painted(&output).contains("合成の用法変更理由"),
        "selecting B must show B's detail"
    );
    let first = visible_label(&output, "変更：意味").expect("first change remains reachable");
    output = select(&mut app, first.center());
    assert!(
        visible_label(&output, "変更前").is_some(),
        "returning to A must reset the old detail scroll to the top"
    );
    for _ in 0..60 {
        if visible_glyph_position(&output, "RIGHT_TAIL").is_some() {
            break;
        }
        render(&mut app, wheel(right_point, 35.0));
        output = render(&mut app, vec![]);
    }
    assert!(
        visible_glyph_position(&output, "RIGHT_TAIL").is_some(),
        "the detail tail must be reachable without moving the list"
    );
    // Keep later left wheel input separate from pending right wheel animation.
    for _ in 0..30 {
        output = render(&mut app, vec![]);
    }
    let left_point = egui::pos2(list_before.center().x, 260.0);
    for _ in 0..60 {
        if visible_label(&output, "追加：例文 36").is_some() {
            break;
        }
        render(&mut app, wheel(left_point, 35.0));
        output = render(&mut app, vec![]);
    }
    assert!(
        visible_label(&output, "追加：例文 36").is_some(),
        "the final change must be reachable in the list pane"
    );
    assert_eq!(serde_json::to_vec(&draft).unwrap(), original_draft);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn review_detail_standard_and_narrow_dialog_keep_panes_and_actions_visible() {
    for (case, window, zoom) in [
        ("standard", egui::vec2(1150.0, 950.0), 0.8),
        ("narrow", egui::vec2(820.0, 650.0), 1.6),
    ] {
        let (ctx, mut app, root) = super::harness_tests::fixture();
        ctx.set_zoom_factor(zoom);
        app.page = Page::Chat;
        app.chat_material_open = true;
        app.progress.material_draft = Some(super::harness_tests::material_ui_draft(&app));
        let original = serde_json::to_vec(&app.progress).unwrap();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, window / zoom);
        let mut output = None;
        for _ in 0..4 {
            output = Some(ctx.run(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ctx| app.update_ui(ctx),
            ));
        }
        let output = output.unwrap();
        for label in [
            "チャットを教材に反映",
            "変更箇所",
            "変更理由・該当引用",
            "内容を確認して教材に登録",
            "教材案を破棄",
        ] {
            let rect = visible_label(&output, label)
                .unwrap_or_else(|| panic!("{case}: {label} must be visible"));
            assert!(
                screen.contains_rect(rect),
                "{case}: {label} outside screen: {rect:?}"
            );
        }
        let left = visible_label(&output, "変更箇所").unwrap();
        let right = visible_label(&output, "変更理由・該当引用").unwrap();
        assert!(
            left.right() < right.left(),
            "{case}: both panes must be side by side"
        );
        let heading_top = visible_label(&output, "チャットを教材に反映")
            .unwrap()
            .top();
        let register_top = visible_label(&output, "内容を確認して教材に登録")
            .unwrap()
            .top();
        for point in [left.center(), right.center()] {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events: vec![
                        egui::Event::PointerMoved(point),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, -35.0),
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    ..Default::default()
                },
                |ctx| app.update_ui(ctx),
            );
            let after = ctx.run(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |ctx| app.update_ui(ctx),
            );
            assert!(
                (visible_label(&after, "チャットを教材に反映").unwrap().top() - heading_top).abs()
                    <= 2.0,
                "{case}: header moved during pane scrolling"
            );
            assert!(
                (visible_label(&after, "内容を確認して教材に登録")
                    .unwrap()
                    .top()
                    - register_top)
                    .abs()
                    <= 2.0,
                "{case}: registration footer moved during pane scrolling"
            );
        }
        assert_eq!(serde_json::to_vec(&app.progress).unwrap(), original);
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn review_detail_narrow_footer_stays_visible_when_an_edit_invalidates_ready() {
    use wordweave5::material::Mode;

    let (ctx, mut app, root) = super::harness_tests::fixture();
    ctx.set_zoom_factor(1.6);
    app.page = Page::Chat;
    app.chat_material_open = true;
    let mut draft = super::harness_tests::material_ui_draft(&app);
    draft.mode = Mode::New;
    draft.baseline = None;
    draft.source.mode = Mode::New;
    draft.candidate.id = "isolated-new-same-base".into();
    draft.source.entry_id = draft.candidate.id.clone();
    while draft.candidate.examples.len() < 2 {
        let index = draft.candidate.examples.len();
        draft.candidate.examples.push(wordweave5::model::Example {
            english: format!("Isolated sentence {index}."),
            japanese: format!("隔離例文{index}"),
            note: "隔離した補足".into(),
        });
    }
    draft.generated = Some(draft.candidate.clone());
    assert!(
        draft.ready(&app.deck, true).is_ok(),
        "fixture must start ready"
    );
    assert!(
        draft.ready(&app.deck, false).is_err(),
        "the same-base checkbox controls readiness"
    );
    app.material_same_base = true;
    app.progress.material_draft = Some(draft);
    let original_progress = serde_json::to_vec(&app.progress).unwrap();
    let original_deck = wordweave5::model::deck_text(&app.deck);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(820.0, 650.0) / 1.6);
    let render = |app: &mut WordApp, events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(screen),
                events,
                ..Default::default()
            },
            |ctx| app.update_ui(ctx),
        )
    };
    let mut output = render(&mut app, vec![]);
    for _ in 0..4 {
        output = render(&mut app, vec![]);
    }
    let helper_point = visible_label(&output, "元の会話を表示")
        .expect("proposal helper must have a visible source button")
        .center();
    assert!(visible_label(&output, "内容を確認して教材に登録").is_some());
    assert!(visible_label(&output, "理由を詳しく確認").is_none());

    let checkbox_label = "同じ基本語の別用法として新規登録する";
    for _ in 0..8 {
        if visible_label(&output, checkbox_label).is_some() {
            break;
        }
        render(
            &mut app,
            vec![
                egui::Event::PointerMoved(helper_point),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -35.0),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        for _ in 0..30 {
            output = render(&mut app, vec![]);
        }
    }
    let checkbox = visible_label(&output, checkbox_label)
        .expect("same-base checkbox must be reachable by scrolling the helper pane");
    let point = checkbox.center();
    let button = |pressed| egui::Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    render(
        &mut app,
        vec![egui::Event::PointerMoved(point), button(true)],
    );
    output = render(&mut app, vec![button(false)]);

    assert!(
        !app.material_same_base,
        "real checkbox click must invalidate this proposal"
    );
    let reason = visible_glyph_position(&output, "登録済み")
        .expect("the newly invalid reason must be visible in the same frame");
    let explain = visible_label(&output, "理由を詳しく確認")
        .expect("the invalid proposal must retain its explanation action");
    let register = visible_label(&output, "内容を確認して教材に登録")
        .expect("the disabled registration action must remain visible");
    let discard = visible_label(&output, "教材案を破棄")
        .expect("discard must remain visible after invalidation");
    for rect in [explain, register, discard] {
        assert!(
            screen.contains_rect(rect),
            "footer action left the narrow screen: {rect:?}"
        );
    }
    assert!(
        screen.contains(reason),
        "new rejection reason left the narrow screen"
    );
    assert!(
        reason.y < register.top()
            && explain.bottom() <= register.top()
            && !explain.contains(reason),
        "reason and explanation may share a row but must stay separate from registration"
    );
    assert_eq!(
        serde_json::to_vec(&app.progress).unwrap(),
        original_progress,
        "a readiness edit must preserve the unregistered proposal and learning state"
    );
    assert_eq!(wordweave5::model::deck_text(&app.deck), original_deck);
    drop(app);
    std::fs::remove_dir_all(root).unwrap();
}
