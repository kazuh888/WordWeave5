use crate::app::controls::UiControls as _;
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};
use image::{Rgb, RgbImage};

#[derive(Default)]
pub struct Ink {
    pub strokes: Vec<Vec<[f32; 2]>>,
}
impl Ink {
    pub fn empty(&self) -> bool {
        self.strokes.iter().all(Vec::is_empty)
    }
    pub fn clear(&mut self) {
        self.strokes.clear();
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let size = Vec2::new(ui.available_width().min(1000.0), 155.0);
        let (response, painter) = ui.allocate_painter(size, Sense::drag());
        let rect = response.rect;
        painter.rect_filled(rect, 6.0, Color32::WHITE);
        painter.line_segment(
            [
                rect.left_bottom() - Vec2::new(0.0, 35.0),
                rect.right_bottom() - Vec2::new(0.0, 35.0),
            ],
            Stroke::new(1.0_f32, Color32::from_gray(215)),
        );
        if response.drag_started() {
            self.strokes.push(Vec::new());
        }
        if response.dragged() || response.drag_started() {
            if let Some(p) = response.interact_pointer_pos() {
                let points: usize = self.strokes.iter().map(Vec::len).sum();
                if points < 30000 {
                    if let Some(stroke) = self.strokes.last_mut() {
                        stroke.push([
                            ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0),
                            ((p.y - rect.min.y) / rect.height()).clamp(0.0, 1.0),
                        ]);
                    }
                }
            }
        }
        for stroke in &self.strokes {
            let points: Vec<Pos2> = stroke
                .iter()
                .map(|p| rect.min + Vec2::new(p[0] * rect.width(), p[1] * rect.height()))
                .collect();
            if points.len() == 1 {
                painter.circle_filled(points[0], 1.8, Color32::from_rgb(35, 50, 65));
            }
            for p in points.windows(2) {
                painter.line_segment(
                    [p[0], p[1]],
                    Stroke::new(2.6_f32, Color32::from_rgb(35, 50, 65)),
                );
            }
        }
        ui.horizontal(|ui| {
            if ui.ww_button("一画戻す").clicked() {
                self.strokes.pop();
            }
            if ui.ww_button("手書きを消す").clicked() {
                self.clear();
            }
            ui.small("ペンで回答を書く。無料では見本と照合して自己評価できる。");
        });
    }
    pub fn png(&self) -> Result<Vec<u8>, String> {
        if self.empty() {
            return Err("手書き欄が空です。".into());
        }
        let (w, h) = (1400, 300);
        let mut img = RgbImage::from_pixel(w, h, Rgb([255, 255, 255]));
        for stroke in &self.strokes {
            for i in 0..stroke.len() {
                let a = stroke[i.saturating_sub(1)];
                let b = stroke[i];
                let (x0, y0) = (a[0] * (w - 1) as f32, a[1] * (h - 1) as f32);
                let (x1, y1) = (b[0] * (w - 1) as f32, b[1] * (h - 1) as f32);
                let n = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as u32;
                for s in 0..=n {
                    let x = (x0 + (x1 - x0) * s as f32 / n as f32).round() as i32;
                    let y = (y0 + (y1 - y0) * s as f32 / n as f32).round() as i32;
                    for dx in -2..=2 {
                        for dy in -2..=2 {
                            if x + dx >= 0 && x + dx < w as i32 && y + dy >= 0 && y + dy < h as i32
                            {
                                img.put_pixel((x + dx) as u32, (y + dy) as u32, Rgb([25, 35, 45]));
                            }
                        }
                    }
                }
            }
        }
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut out, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(out.into_inner())
    }
}
