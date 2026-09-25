use eframe::egui::*;

pub fn home_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("home_body".into()))
}

pub const INK: Color32 = Color32::from_rgb(21, 48, 72);
pub const MUTED: Color32 = Color32::from_rgb(70, 91, 110);
pub const BLUE: Color32 = Color32::from_rgb(0, 101, 198);
pub const BORDER: Color32 = Color32::from_rgb(216, 232, 244);
pub const GREEN: Color32 = Color32::from_rgb(21, 122, 83);
pub const ROSE: Color32 = Color32::from_rgb(169, 51, 79);
pub const PURPLE: Color32 = Color32::from_rgb(110, 61, 176);

#[derive(Clone, Copy)]
pub enum Icon {
    Clock,
    Book,
    Refresh,
    Check,
    File,
    Bulb,
    Leaf,
    Home,
    Chart,
    Chat,
    Gear,
    Plus,
    Copy,
}

pub fn icon(p: &Painter, r: Rect, kind: Icon, color: Color32) {
    let c = r.center();
    let u = r.width().min(r.height()) / 2.0;
    let point = |x: f32, y: f32| c + vec2(x * u, y * u);
    let stroke = Stroke::new((r.width() / 15.0).clamp(1.6, 3.0), color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([point(a.0, a.1), point(b.0, b.1)], stroke);
    };
    match kind {
        Icon::Clock => {
            p.circle_stroke(c, u * 0.82, stroke);
            line((0.0, -0.5), (0.0, 0.05));
            line((0.0, 0.05), (0.42, 0.05));
        }
        Icon::Book => {
            for sign in [-1.0, 1.0] {
                p.add(Shape::line(
                    vec![
                        point(0.0, -0.5),
                        point(0.38 * sign, -0.72),
                        point(0.82 * sign, -0.66),
                        point(0.82 * sign, 0.58),
                        point(0.38 * sign, 0.55),
                        point(0.0, 0.75),
                    ],
                    stroke,
                ));
            }
            line((0.0, -0.5), (0.0, 0.75));
        }
        Icon::Refresh => {
            for offset in [0.0, std::f32::consts::PI] {
                let points = (0..25)
                    .map(|i| {
                        let a = offset + i as f32 / 24.0 * 2.35;
                        c + vec2(a.cos(), a.sin()) * (u * 0.7)
                    })
                    .collect();
                p.add(Shape::line(points, stroke));
            }
            line((0.7, 0.0), (0.9, 0.35));
            line((0.7, 0.0), (0.34, 0.1));
            line((-0.7, 0.0), (-0.9, -0.35));
            line((-0.7, 0.0), (-0.34, -0.1));
        }
        Icon::Check => {
            line((-0.65, 0.0), (-0.17, 0.49));
            line((-0.17, 0.49), (0.72, -0.48));
        }
        Icon::File => {
            p.rect_stroke(
                Rect::from_min_max(point(-0.59, -0.83), point(0.59, 0.83)),
                2,
                stroke,
                StrokeKind::Inside,
            );
            for y in [-0.26, 0.12, 0.48] {
                line((-0.29, y), (0.28, y));
            }
        }
        Icon::Bulb => {
            p.circle_stroke(point(0.0, -0.22), u * 0.48, stroke);
            line((-0.22, 0.23), (-0.22, 0.65));
            line((0.22, 0.23), (0.22, 0.65));
            line((-0.23, 0.66), (0.23, 0.66));
            line((-0.15, 0.88), (0.15, 0.88));
            for a in [-2.8_f32, -2.05, -1.1, -0.35] {
                let v = vec2(a.cos(), a.sin());
                p.line_segment(
                    [
                        point(0.0, -0.22) + v * u * 0.73,
                        point(0.0, -0.22) + v * u * 0.93,
                    ],
                    stroke,
                );
            }
        }
        Icon::Leaf => {
            p.add(Shape::convex_polygon(
                vec![
                    point(-0.63, 0.5),
                    point(-0.8, 0.1),
                    point(-0.45, -0.45),
                    point(0.82, -0.8),
                    point(0.65, 0.25),
                    point(0.15, 0.7),
                ],
                color,
                Stroke::NONE,
            ));
            line((-0.82, 0.94), (0.5, -0.53));
        }
        Icon::Home => {
            p.add(Shape::line(
                vec![point(-0.85, -0.03), point(0.0, -0.85), point(0.85, -0.03)],
                stroke,
            ));
            p.add(Shape::line(
                vec![
                    point(-0.57, -0.13),
                    point(-0.57, 0.73),
                    point(-0.17, 0.73),
                    point(-0.17, 0.24),
                    point(0.2, 0.24),
                    point(0.2, 0.73),
                    point(0.59, 0.73),
                    point(0.59, -0.13),
                ],
                stroke,
            ));
        }
        Icon::Chart => {
            for (x, y) in [(-0.57, -0.1), (0.0, -0.72), (0.57, -0.4)] {
                p.line_segment(
                    [point(x, 0.73), point(x, y)],
                    Stroke::new(stroke.width * 2.2, color),
                );
            }
        }
        Icon::Chat => {
            p.rect_stroke(r.shrink(r.width() * 0.11), 5, stroke, StrokeKind::Inside);
            line((-0.3, 0.78), (-0.65, 1.0));
        }
        Icon::Gear => {
            p.circle_stroke(c, u * 0.6, Stroke::new(stroke.width * 2.2, color));
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let v = vec2(a.cos(), a.sin());
                p.line_segment([c + v * u * 0.65, c + v * u * 0.95], stroke);
            }
        }
        Icon::Plus => {
            p.circle_filled(c, u * 0.9, color);
            let s = Stroke::new(stroke.width, Color32::WHITE);
            p.line_segment([point(-0.45, 0.0), point(0.45, 0.0)], s);
            p.line_segment([point(0.0, -0.45), point(0.0, 0.45)], s);
        }
        Icon::Copy => {
            p.rect_stroke(
                Rect::from_min_max(point(-0.62, -0.36), point(0.36, 0.72)),
                2,
                stroke,
                StrokeKind::Inside,
            );
            p.rect_stroke(
                Rect::from_min_max(point(-0.30, -0.72), point(0.68, 0.36)),
                2,
                stroke,
                StrokeKind::Inside,
            );
        }
    }
}

pub fn badge(ui: &mut Ui, kind: Icon, color: Color32, size: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().circle_filled(
        r.center(),
        size / 2.0,
        color.blend(Color32::from_white_alpha(231)),
    );
    icon(ui.painter(), r.shrink(size * 0.25), kind, color);
}

pub fn scenery(p: &Painter, r: Rect) {
    let p = p.with_clip_rect(r);
    let at = |x: f32, y: f32| pos2(r.left() + r.width() * x, r.top() + r.height() * y);
    p.circle_filled(at(0.8, 0.32), 28.0, Color32::from_rgb(255, 252, 228));
    for (ridge, color) in [
        (
            vec![
                (0.18, 0.77),
                (0.4, 0.35),
                (0.51, 0.54),
                (0.66, 0.24),
                (0.84, 0.54),
                (1.0, 0.3),
            ],
            Color32::from_rgb(215, 235, 248),
        ),
        (
            vec![
                (0.12, 0.93),
                (0.34, 0.63),
                (0.45, 0.72),
                (0.62, 0.49),
                (0.74, 0.69),
                (0.9, 0.4),
                (1.0, 0.51),
            ],
            Color32::from_rgb(192, 220, 241),
        ),
        (
            vec![
                (0.25, 1.0),
                (0.47, 0.8),
                (0.61, 0.87),
                (0.81, 0.67),
                (0.92, 0.78),
                (1.0, 0.62),
            ],
            Color32::from_rgb(165, 205, 229),
        ),
    ] {
        // Individual trapezoids keep non-convex mountain ridges out of convex_polygon.
        for edge in ridge.windows(2) {
            let a = edge[0];
            let b = edge[1];
            p.add(Shape::convex_polygon(
                vec![at(a.0, a.1), at(b.0, b.1), at(b.0, 1.0), at(a.0, 1.0)],
                color,
                Stroke::NONE,
            ));
        }
    }
    for i in 0..6 {
        let x = 0.72 + i as f32 * 0.045;
        let top = 0.78 + (i % 3) as f32 * 0.04;
        p.line_segment(
            [at(x, 1.0), at(x + 0.03, top)],
            Stroke::new(1.5_f32, Color32::from_rgb(114, 169, 161)),
        );
        for j in 0..3 {
            let y = top + j as f32 * 0.06;
            p.add(Shape::convex_polygon(
                vec![
                    at(x + 0.02, y + 0.04),
                    at(x - 0.014, y),
                    at(x - 0.019, y + 0.06),
                ],
                Color32::from_rgb(145, 189, 167),
                Stroke::NONE,
            ));
        }
    }
}

pub fn title(ui: &mut Ui, text: &str, size: f32) -> Response {
    ui.add(
        Label::new(
            RichText::new(text)
                .family(FontFamily::Name("heading".into()))
                .size(size)
                .color(INK),
        )
        .wrap(),
    )
}

pub fn card(
    ui: &mut Ui,
    width: f32,
    height: f32,
    fill: Color32,
    content: impl FnOnce(&mut Ui),
) -> Rect {
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(12)
        .inner_margin(20)
        .show(ui, |ui| {
            // Frames inherit a row's horizontal layout; cards need their own column.
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.set_width((width - 42.0).max(1.0));
                ui.set_min_height(height - 42.0);
                content(ui);
            });
        })
        .response
        .rect
}

pub fn primary(ui: &mut Ui, text: &str, width: f32) -> Response {
    let font = FontId::new(23.0, FontFamily::Name("heading".into()));
    let response = ui.add_sized(
        [width.min(ui.available_width()), 64.0],
        Button::new(
            RichText::new(text)
                .size(23.0)
                .color(Color32::TRANSPARENT)
                .family(FontFamily::Name("heading".into())),
        )
        .fill(BLUE)
        .corner_radius(10),
    );
    centered_ink(
        ui.painter(),
        text,
        font,
        Color32::WHITE,
        response.rect.center(),
    );
    response
}

// Yu Gothic's line box has unequal space above/below its visible glyphs.
// Keep the standard Button's input/accessibility behavior, but center its ink.
fn centered_ink(p: &Painter, text: &str, font: FontId, color: Color32, center: Pos2) {
    let galley = p.layout_no_wrap(text.to_owned(), font, color);
    let bounds = if galley.mesh_bounds.is_finite() {
        galley.mesh_bounds
    } else {
        galley.rect
    };
    p.galley(center - bounds.center().to_vec2(), galley, color);
}

pub fn nav_item(ui: &mut Ui, text: &str, kind: Icon, active: bool) -> Response {
    let font_size = if text == "ホーム" { 16.0 } else { 17.0 };
    let color = if ui.is_enabled() {
        if active {
            BLUE
        } else {
            MUTED
        }
    } else {
        ui.visuals().weak_text_color()
    };
    let fill = if active {
        Color32::from_rgb(225, 241, 255)
    } else {
        Color32::TRANSPARENT
    };
    let label_width = ui
        .painter()
        .layout_no_wrap(text.to_owned(), home_font(font_size), color)
        .size()
        .x;
    let response = ui.add_sized(
        vec2(label_width + 54.0, 42.0),
        Button::new(
            RichText::new(text)
                .size(font_size)
                .color(Color32::TRANSPARENT),
        )
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(8)
        .min_size(vec2(0.0, 42.0)),
    );
    let text_left = response.rect.left() + 40.0;
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), home_font(font_size), color);
    centered_ink(
        ui.painter(),
        text,
        home_font(font_size),
        color,
        pos2(
            text_left + galley.mesh_bounds.width() / 2.0,
            response.rect.center().y,
        ),
    );
    icon(
        ui.painter(),
        Rect::from_center_size(
            pos2(response.rect.left() + 21.0, response.rect.center().y),
            vec2(19.0, 19.0),
        ),
        kind,
        color,
    );
    if active {
        ui.painter().line_segment(
            [
                response.rect.left_bottom() + vec2(7.0, -1.0),
                response.rect.right_bottom() + vec2(-7.0, -1.0),
            ],
            Stroke::new(2.0_f32, BLUE),
        );
    }
    response
}

pub fn size_button(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let response = ui.add(
        Button::new(RichText::new(text).size(16.0).color(Color32::TRANSPARENT))
            .fill(if selected {
                Color32::from_rgb(225, 241, 255)
            } else {
                Color32::WHITE
            })
            .stroke(Stroke::new(1.0_f32, BORDER))
            .min_size(vec2(36.0, 32.0))
            .corner_radius(6),
    );
    centered_ink(
        ui.painter(),
        text,
        home_font(16.0),
        if selected { BLUE } else { MUTED },
        response.rect.center(),
    );
    response
}

pub fn centered_label(ui: &mut Ui, text: &str, size: f32, row_height: f32) -> Response {
    let galley = ui.painter().layout(
        text.to_owned(),
        home_font(size),
        MUTED,
        ui.max_rect().width().max(1.0),
    );
    let (rect, response) = ui.allocate_exact_size(
        vec2(galley.size().x, row_height.max(galley.size().y)),
        Sense::hover(),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), text));
    let bounds = if galley.mesh_bounds.is_finite() {
        galley.mesh_bounds
    } else {
        galley.rect
    };
    let origin = pos2(
        rect.left() - bounds.left(),
        rect.center().y - bounds.center().y,
    );
    ui.painter().galley(origin, galley, MUTED);
    response
}
