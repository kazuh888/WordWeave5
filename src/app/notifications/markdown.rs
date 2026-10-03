//! Read-only CommonMark/GFM rendering. The caller retains the exact source for copying.
use eframe::egui::{
    self,
    text::{LayoutJob, TextFormat},
    Color32, FontFamily, FontId,
};
use pulldown_cmark::{Alignment, Event, Options, Parser, Tag};

enum Kind<'a> {
    Tagged(Tag<'a>),
    Leaf(Event<'a>),
}

struct Node<'a> {
    kind: Kind<'a>,
    children: Vec<usize>,
}

fn parse_nodes<'a>(source: &'a str) -> Vec<Node<'a>> {
    let mut nodes = vec![Node {
        kind: Kind::Tagged(Tag::Paragraph),
        children: Vec::new(),
    }];
    let mut parents = vec![0];
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(tag) => {
                let index = nodes.len();
                nodes.push(Node {
                    kind: Kind::Tagged(tag),
                    children: Vec::new(),
                });
                nodes[*parents.last().unwrap()].children.push(index);
                parents.push(index);
            }
            Event::End(_) => {
                parents.pop();
            }
            other => {
                let index = nodes.len();
                nodes.push(Node {
                    kind: Kind::Leaf(other),
                    children: Vec::new(),
                });
                nodes[*parents.last().unwrap()].children.push(index);
            }
        }
    }
    nodes
}

#[derive(Clone, Copy, Default)]
struct Style {
    strong: bool,
    emphasis: bool,
    strike: bool,
    code: bool,
}

fn append(job: &mut LayoutJob, value: &str, font: &FontId, color: Color32, style: Style) {
    let mut format = TextFormat {
        font_id: font.clone(),
        color,
        ..Default::default()
    };
    if style.strong {
        format.font_id.family = FontFamily::Name("heading".into());
    }
    if style.emphasis {
        format.italics = true;
    }
    if style.strike {
        format.strikethrough = egui::Stroke::new(1.0_f32, color);
    }
    if style.code {
        format.font_id.family = FontFamily::Monospace;
        format.background = Color32::from_gray(235);
    }
    job.append(value, 0.0, format);
}

fn plain(nodes: &[Node<'_>], roots: &[usize]) -> String {
    let mut text = String::new();
    let mut pending: Vec<usize> = roots.iter().rev().copied().collect();
    while let Some(index) = pending.pop() {
        match &nodes[index].kind {
            Kind::Tagged(_) => pending.extend(nodes[index].children.iter().rev().copied()),
            Kind::Leaf(
                Event::Text(value)
                | Event::Code(value)
                | Event::Html(value)
                | Event::InlineHtml(value),
            ) => text.push_str(value),
            Kind::Leaf(Event::SoftBreak | Event::HardBreak) => text.push(' '),
            _ => {}
        }
    }
    text
}

fn inlines(
    nodes: &[Node<'_>],
    roots: &[usize],
    job: &mut LayoutJob,
    font: &FontId,
    color: Color32,
    style: Style,
) {
    let mut pending: Vec<(usize, Style)> =
        roots.iter().rev().map(|index| (*index, style)).collect();
    while let Some((index, style)) = pending.pop() {
        match &nodes[index].kind {
            Kind::Tagged(Tag::Image { dest_url, .. }) => {
                let alt = plain(nodes, &nodes[index].children);
                append(
                    job,
                    &format!("画像: {alt} ({dest_url})"),
                    font,
                    color,
                    style,
                );
            }
            Kind::Tagged(tag) => {
                let mut child_style = style;
                match tag {
                    Tag::Strong => child_style.strong = true,
                    Tag::Emphasis => child_style.emphasis = true,
                    Tag::Strikethrough => child_style.strike = true,
                    _ => {}
                }
                pending.extend(
                    nodes[index]
                        .children
                        .iter()
                        .rev()
                        .map(|child| (*child, child_style)),
                );
            }
            Kind::Leaf(event) => match event {
                Event::Text(value) | Event::Html(value) | Event::InlineHtml(value) => {
                    append(job, value, font, color, style)
                }
                Event::Code(value) => append(
                    job,
                    value,
                    font,
                    color,
                    Style {
                        code: true,
                        ..style
                    },
                ),
                Event::SoftBreak => append(job, " ", font, color, style),
                Event::HardBreak => append(job, "\n", font, color, style),
                Event::TaskListMarker(checked) => append(
                    job,
                    if *checked { "[x] " } else { "[ ] " },
                    font,
                    color,
                    style,
                ),
                _ => {}
            },
        }
    }
}

fn label(ui: &mut egui::Ui, nodes: &[Node<'_>], roots: &[usize], heading: bool, prefix: &str) {
    let mut font = ui
        .style()
        .override_font_id
        .clone()
        .unwrap_or_else(|| egui::TextStyle::Body.resolve(ui.style()));
    if heading {
        font.size += 2.0;
    }
    let mut job = LayoutJob::default();
    append(
        &mut job,
        prefix,
        &font,
        ui.visuals().text_color(),
        Style::default(),
    );
    inlines(
        nodes,
        roots,
        &mut job,
        &font,
        ui.visuals().text_color(),
        Style {
            strong: heading,
            ..Style::default()
        },
    );
    ui.add(egui::Label::new(job).wrap().selectable(true));
}

fn code_block(ui: &mut egui::Ui, nodes: &[Node<'_>], roots: &[usize], id: egui::Id) {
    let value = plain(nodes, roots);
    let width = ui.available_width().max(1.0);
    egui::ScrollArea::horizontal()
        .id_salt(id)
        .auto_shrink([false, true])
        .min_scrolled_width(0.0)
        .show(ui, |ui| {
            ui.set_max_width(width);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(value)
                        .monospace()
                        .background_color(Color32::from_gray(235)),
                )
                .extend()
                .selectable(true),
            );
        });
}

fn table(
    ui: &mut egui::Ui,
    nodes: &[Node<'_>],
    index: usize,
    alignments: &[Alignment],
    id: egui::Id,
) {
    let rows: Vec<(&[usize], bool)> = nodes[index]
        .children
        .iter()
        .filter_map(|child| match &nodes[*child].kind {
            Kind::Tagged(Tag::TableHead) => Some((nodes[*child].children.as_slice(), true)),
            Kind::Tagged(Tag::TableRow) => Some((nodes[*child].children.as_slice(), false)),
            _ => None,
        })
        .collect();
    let columns = alignments
        .len()
        .max(rows.iter().map(|(cells, _)| cells.len()).max().unwrap_or(0));
    if columns == 0 {
        return;
    }
    let viewport = ui.available_width().max(1.0);
    let cell_width = ((viewport - 16.0) / columns as f32).clamp(120.0, 240.0);
    egui::ScrollArea::horizontal()
        .id_salt(id)
        .auto_shrink([false, true])
        .min_scrolled_width(0.0)
        .show(ui, |ui| {
            ui.set_min_width(columns as f32 * (cell_width + 8.0));
            egui::Grid::new(id.with("grid"))
                .striped(true)
                .show(ui, |ui| {
                    for (cells, header) in rows {
                        for column in 0..columns {
                            let alignment = match alignments.get(column) {
                                Some(Alignment::Right) => egui::Align::RIGHT,
                                Some(Alignment::Center) => egui::Align::Center,
                                _ => egui::Align::LEFT,
                            };
                            ui.with_layout(egui::Layout::top_down(alignment), |ui| {
                                ui.set_width(cell_width);
                                if let Some(cell) = cells.get(column) {
                                    if matches!(nodes[*cell].kind, Kind::Tagged(Tag::TableCell)) {
                                        label(ui, nodes, &nodes[*cell].children, header, "");
                                    }
                                }
                            });
                        }
                        ui.end_row();
                    }
                });
        });
}

#[derive(Clone, Copy)]
struct Depth {
    quote: usize,
    list: usize,
}

fn prefix(depth: Depth, marker: &str) -> String {
    let mut value = if depth.quote > 3 {
        format!("│×{} ", depth.quote)
    } else {
        "│ ".repeat(depth.quote)
    };
    let nested = depth.list.saturating_sub(1);
    if nested > 3 {
        value.push_str(&format!("↳{nested} "));
    }
    value.push_str(marker);
    value
}

fn indented(ui: &mut egui::Ui, depth: Depth, draw: impl FnOnce(&mut egui::Ui)) {
    let available = ui.available_width().max(1.0);
    let levels = depth.list.saturating_sub(1).min(3) + depth.quote.min(3);
    let offset = (levels as f32 * 12.0).min(available * 0.25);
    ui.horizontal_top(|ui| {
        ui.add_space(offset);
        ui.allocate_ui_with_layout(
            egui::vec2((available - offset).max(1.0), 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            draw,
        );
    });
}

fn hanging_indent(ui: &mut egui::Ui, marker: &str, draw: impl FnOnce(&mut egui::Ui)) {
    let font = ui
        .style()
        .override_font_id
        .clone()
        .unwrap_or_else(|| egui::TextStyle::Body.resolve(ui.style()));
    let marker_width = ui
        .painter()
        .layout_no_wrap(marker.to_owned(), font, ui.visuals().text_color())
        .size()
        .x;
    let available = ui.available_width().max(1.0);
    let offset = marker_width.min(available * 0.25);
    ui.horizontal_top(|ui| {
        ui.add_space(offset);
        ui.allocate_ui_with_layout(
            egui::vec2((available - offset).max(1.0), 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            draw,
        );
    });
}

enum Task {
    Node(usize, Depth, String),
    Inline(Vec<usize>, Depth, String, Option<String>),
}

fn item_tasks(nodes: &[Node<'_>], index: usize, depth: Depth, marker: String) -> Vec<Task> {
    let mut result = Vec::new();
    let mut run = Vec::new();
    let mut first = true;
    for child in &nodes[index].children {
        match &nodes[*child].kind {
            Kind::Tagged(Tag::Paragraph) => {
                run.extend(nodes[*child].children.iter().copied());
                result.push(Task::Inline(
                    std::mem::take(&mut run),
                    depth,
                    if first { marker.clone() } else { String::new() },
                    (!first).then(|| marker.clone()),
                ));
                first = false;
            }
            Kind::Tagged(
                Tag::Strong
                | Tag::Emphasis
                | Tag::Strikethrough
                | Tag::Link { .. }
                | Tag::Image { .. },
            )
            | Kind::Leaf(
                Event::Text(_)
                | Event::Code(_)
                | Event::SoftBreak
                | Event::HardBreak
                | Event::TaskListMarker(_)
                | Event::Html(_)
                | Event::InlineHtml(_),
            ) => run.push(*child),
            _ => {
                if !run.is_empty() {
                    result.push(Task::Inline(
                        std::mem::take(&mut run),
                        depth,
                        if first { marker.clone() } else { String::new() },
                        (!first).then(|| marker.clone()),
                    ));
                    first = false;
                }
                if first {
                    result.push(Task::Inline(Vec::new(), depth, marker.clone(), None));
                    first = false;
                }
                result.push(Task::Node(*child, depth, String::new()));
            }
        }
    }
    if !run.is_empty() || first {
        result.push(Task::Inline(
            run,
            depth,
            if first { marker.clone() } else { String::new() },
            (!first).then_some(marker),
        ));
    }
    result
}

fn blocks(ui: &mut egui::Ui, nodes: &[Node<'_>], id: egui::Id) {
    let root = Depth { quote: 0, list: 0 };
    let mut pending: Vec<Task> = nodes[0]
        .children
        .iter()
        .rev()
        .map(|index| Task::Node(*index, root, String::new()))
        .collect();
    while let Some(task) = pending.pop() {
        match task {
            Task::Inline(roots, depth, marker, continuation) => indented(ui, depth, |ui| {
                let draw =
                    |ui: &mut egui::Ui| label(ui, nodes, &roots, false, &prefix(depth, &marker));
                if let Some(marker) = continuation {
                    hanging_indent(ui, &marker, draw);
                } else {
                    draw(ui);
                }
            }),
            Task::Node(index, depth, marker) => match &nodes[index].kind {
                Kind::Tagged(Tag::Paragraph | Tag::TableCell | Tag::HtmlBlock) => {
                    indented(ui, depth, |ui| {
                        label(
                            ui,
                            nodes,
                            &nodes[index].children,
                            false,
                            &prefix(depth, &marker),
                        )
                    })
                }
                Kind::Tagged(Tag::Heading { .. }) => indented(ui, depth, |ui| {
                    label(
                        ui,
                        nodes,
                        &nodes[index].children,
                        true,
                        &prefix(depth, &marker),
                    )
                }),
                Kind::Tagged(Tag::CodeBlock(_)) => {
                    indented(ui, depth, |ui| {
                        let lead = prefix(depth, &marker);
                        if !lead.is_empty() {
                            ui.label(lead);
                        }
                        code_block(ui, nodes, &nodes[index].children, id.with(index));
                    });
                }
                Kind::Tagged(Tag::Table(alignments)) => {
                    indented(ui, depth, |ui| {
                        let lead = prefix(depth, &marker);
                        if !lead.is_empty() {
                            ui.label(lead);
                        }
                        table(ui, nodes, index, alignments, id.with(index));
                    });
                }
                Kind::Tagged(Tag::BlockQuote(_)) => {
                    let next = Depth {
                        quote: depth.quote + 1,
                        ..depth
                    };
                    pending.extend(
                        nodes[index]
                            .children
                            .iter()
                            .rev()
                            .map(|child| Task::Node(*child, next, marker.clone())),
                    );
                }
                Kind::Tagged(Tag::List(start)) => {
                    let next = Depth {
                        list: depth.list + 1,
                        ..depth
                    };
                    for (item_index, child) in nodes[index].children.iter().enumerate().rev() {
                        let item_marker = start
                            .map(|first| format!("{}. ", first + item_index as u64))
                            .unwrap_or_else(|| "• ".into());
                        pending.push(Task::Node(*child, next, item_marker));
                    }
                }
                Kind::Tagged(Tag::Item) => {
                    pending.extend(item_tasks(nodes, index, depth, marker).into_iter().rev());
                }
                Kind::Tagged(_) => pending.extend(
                    nodes[index]
                        .children
                        .iter()
                        .rev()
                        .map(|child| Task::Node(*child, depth, marker.clone())),
                ),
                Kind::Leaf(Event::Rule) => {
                    indented(ui, depth, |ui| {
                        ui.separator();
                    });
                }
                Kind::Leaf(_) => indented(ui, depth, |ui| {
                    label(ui, nodes, &[index], false, &prefix(depth, &marker))
                }),
            },
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, id: egui::Id, source: &str) {
    let nodes = parse_nodes(source);
    blocks(ui, &nodes, id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paint(source: &str) -> String {
        let (ctx, app, root) = super::super::super::harness_tests::fixture();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default()
                .show(ctx, |ui| show(ui, egui::Id::new("gfm-test"), source));
        });
        let mut drawn = String::new();
        for shape in &output.shapes {
            super::super::super::harness_tests::shape_text(&shape.shape, &mut drawn);
        }
        drop(app);
        std::fs::remove_dir_all(root).unwrap();
        drawn
    }

    #[test]
    fn u5_markdown_paint_renders_supported_heading_list_emphasis_and_code() {
        let drawn =
            paint("# 合成見出し\n- **重要**と*斜体*、`code`\n1. 次へ\n```\n**コード内**\n```");
        for expected in [
            "合成見出し",
            "• 重要と斜体、code",
            "1. 次へ",
            "**コード内**",
        ] {
            assert!(drawn.contains(expected), "{expected:?} was lost: {drawn}");
        }
        assert!(
            !drawn.contains("**重要**"),
            "bold marker remained in readable display: {drawn}"
        );
    }

    #[test]
    fn u5_markdown_paint_preserves_unclosed_markers_literal_newline_escapes_and_unsafe_markup() {
        let source = "未閉鎖**強調\n実改行\\n文字列\n![画像](https://invalid.example/a.png) <script>alert(1)</script> 🦊";
        let drawn = paint(source);
        for expected in [
            "未閉鎖**強調",
            "実改行\\n文字列",
            "画像",
            "https://invalid.example/a.png",
            "<script>alert(1)</script>",
            "🦊",
        ] {
            assert!(
                drawn.contains(expected),
                "unsupported/source text was dropped: {drawn}"
            );
        }
        assert!(
            !drawn.contains("![画像]("),
            "image reference must use safe readable text"
        );
    }

    #[test]
    fn gfm_table_paints_cells_without_separator_syntax() {
        let drawn = paint("| 項目 | 説明 |\n| --- | --- |\n| **重要** | `a\\|b` と x\\|y |");
        for expected in ["項目", "説明", "重要", "a|b", "x|y"] {
            assert!(
                drawn.contains(expected),
                "table cell lost {expected:?}: {drawn}"
            );
        }
        assert!(
            !drawn.contains("| --- | --- |"),
            "GFM delimiter row must render as table structure: {drawn}"
        );
        assert!(
            !drawn.contains("x\\|y"),
            "escaped pipe must be displayed as cell text: {drawn}"
        );
    }
}
