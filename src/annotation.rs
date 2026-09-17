//! Fixed-layout English + ink. The texture shown to the user is the PNG sent to Codex.
use eframe::egui::{self, Color32, TextureHandle};
use image::{Rgba, RgbaImage};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 700;

#[derive(Default)]
pub struct Annotation {
    pub text: String,
    pub strokes: Vec<Vec<[f32; 2]>>,
    background: Option<RgbaImage>,
    texture: Option<TextureHandle>,
}
impl Annotation {
    pub fn freeze_blank(&mut self, ctx: &egui::Context) -> Result<(), String> {
        self.text.clear();
        self.strokes.clear();
        self.background = Some(RgbaImage::from_pixel(
            WIDTH,
            HEIGHT,
            Rgba([255, 255, 255, 255]),
        ));
        self.refresh(ctx)
    }
    pub fn frozen(&self) -> bool {
        self.background.is_some()
    }
    pub fn freeze(&mut self, ctx: &egui::Context) -> Result<(), String> {
        self.background = Some(render_text(&self.text)?);
        self.refresh(ctx)
    }
    fn refresh(&mut self, ctx: &egui::Context) -> Result<(), String> {
        let pixels = self.composite()?;
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [WIDTH as usize, HEIGHT as usize],
            pixels.as_raw(),
        );
        if let Some(t) = &mut self.texture {
            t.set(color, egui::TextureOptions::LINEAR);
        } else {
            self.texture =
                Some(ctx.load_texture("annotation", color, egui::TextureOptions::LINEAR));
        }
        Ok(())
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Some(texture) = &self.texture else {
            return;
        };
        let width = ui.available_width().min(1000.0);
        let size = egui::vec2(width, width * HEIGHT as f32 / WIDTH as f32);
        let response = ui.add(egui::Image::new((texture.id(), size)).sense(egui::Sense::drag()));
        let mut changed = false;
        if response.drag_started() {
            self.strokes.push(Vec::new());
        }
        if (response.dragged() || response.drag_started())
            && self.strokes.iter().map(Vec::len).sum::<usize>() < 30_000
        {
            if let Some(p) = response.interact_pointer_pos() {
                if let Some(stroke) = self.strokes.last_mut() {
                    stroke.push([
                        ((p.x - response.rect.min.x) / size.x).clamp(0.0, 1.0),
                        ((p.y - response.rect.min.y) / size.y).clamp(0.0, 1.0),
                    ]);
                    changed = true;
                }
            }
        }
        ui.horizontal(|ui| {
            if ui.button("一画戻す").clicked() {
                self.strokes.pop();
                changed = true;
            }
            if ui.button("注釈だけ消す").clicked() {
                self.strokes.clear();
                changed = true;
            }
            ui.small("赤ペンで丸・矢印・取り消し線を書く。下の画像のまま送信する。");
        });
        if changed {
            if let Err(error) = self.refresh(ui.ctx()) {
                ui.colored_label(Color32::RED, error);
            }
        }
    }
    fn composite(&self) -> Result<RgbaImage, String> {
        let mut image = self
            .background
            .clone()
            .ok_or("先に英文を固定してください。")?;
        if self.strokes.iter().map(Vec::len).sum::<usize>() > 30_000 {
            return Err("筆跡が上限を超えています。".into());
        }
        for stroke in &self.strokes {
            for (i, b) in stroke.iter().enumerate() {
                let a = stroke[i.saturating_sub(1)];
                if a.iter()
                    .chain(b)
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                {
                    return Err("筆跡の座標が不正です。".into());
                }
                let (ax, ay, bx, by) = (
                    a[0] * (WIDTH - 1) as f32,
                    a[1] * (HEIGHT - 1) as f32,
                    b[0] * (WIDTH - 1) as f32,
                    b[1] * (HEIGHT - 1) as f32,
                );
                let steps = (bx - ax).abs().max((by - ay).abs()).ceil().max(1.0) as u32;
                for step in 0..=steps {
                    let x = (ax + (bx - ax) * step as f32 / steps as f32).round() as i32;
                    let y = (ay + (by - ay) * step as f32 / steps as f32).round() as i32;
                    for dx in -2..=2 {
                        for dy in -2..=2 {
                            if (0..WIDTH as i32).contains(&(x + dx))
                                && (0..HEIGHT as i32).contains(&(y + dy))
                            {
                                image.put_pixel(
                                    (x + dx) as u32,
                                    (y + dy) as u32,
                                    Rgba([180, 35, 35, 255]),
                                );
                            }
                        }
                    }
                }
            }
        }
        Ok(image)
    }
    pub fn files(&self) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        let ink = serde_json::to_vec(&serde_json::json!({"version":1,"source_text":self.text,"width":WIDTH,"height":HEIGHT,"strokes":self.strokes})).map_err(|e| e.to_string())?;
        Ok((
            ink,
            png(self.background.as_ref().ok_or("英文を固定してください。")?)?,
            png(&self.composite()?)?,
        ))
    }
}
fn png(image: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(bytes.into_inner())
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct BitmapInfo {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x: i32,
    y: i32,
    used: u32,
    important: u32,
    color: u32,
}
#[link(name = "gdi32")]
extern "system" {
    fn CreateCompatibleDC(dc: isize) -> isize;
    fn DeleteDC(dc: isize) -> i32;
    fn CreateDIBSection(
        dc: isize,
        info: *const BitmapInfo,
        usage: u32,
        bits: *mut *mut u8,
        section: isize,
        offset: u32,
    ) -> isize;
    fn SelectObject(dc: isize, object: isize) -> isize;
    fn DeleteObject(object: isize) -> i32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escape: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike: u32,
        charset: u32,
        out: u32,
        clip: u32,
        quality: u32,
        pitch: u32,
        face: *const u16,
    ) -> isize;
    fn SetBkMode(dc: isize, mode: i32) -> i32;
    fn SetTextColor(dc: isize, color: u32) -> u32;
    fn GdiFlush() -> i32;
}
#[link(name = "user32")]
extern "system" {
    fn DrawTextW(dc: isize, text: *const u16, len: i32, rect: *mut Rect, format: u32) -> i32;
}

fn render_text(text: &str) -> Result<RgbaImage, String> {
    if text.trim().is_empty() || text.chars().count() > 2000 || text.contains('\0') {
        return Err("注釈対象の英文を1〜2,000文字で入力してください。".into());
    }
    let wide: Vec<u16> = text.encode_utf16().collect();
    let face: Vec<u16> = "Meiryo\0".encode_utf16().collect();
    // Fixed 32-bit, top-down bitmap: no screenshot or unrelated window content is captured.
    unsafe {
        let dc = CreateCompatibleDC(0);
        if dc == 0 {
            return Err("描画領域を作成できません。".into());
        }
        let info = BitmapInfo {
            size: 40,
            width: WIDTH as i32,
            height: -(HEIGHT as i32),
            planes: 1,
            bit_count: 32,
            compression: 0,
            size_image: 0,
            x: 0,
            y: 0,
            used: 0,
            important: 0,
            color: 0,
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(dc, &info, 0, &mut bits, 0, 0);
        if bitmap == 0 || bits.is_null() {
            DeleteDC(dc);
            return Err("画像を確保できません。".into());
        }
        let old_bitmap = SelectObject(dc, bitmap);
        let font = CreateFontW(-30, 0, 0, 0, 400, 0, 0, 0, 128, 0, 0, 4, 0, face.as_ptr());
        let old_font = if font != 0 { SelectObject(dc, font) } else { 0 };
        let result = (|| {
            if font == 0 || old_bitmap == 0 || old_font == 0 {
                return Err("描画フォントを準備できません。".into());
            }
            let len = (WIDTH * HEIGHT * 4) as usize;
            std::slice::from_raw_parts_mut(bits, len).fill(255);
            SetBkMode(dc, 1);
            SetTextColor(dc, 0x002D2319);
            let mut rect = Rect {
                left: 24,
                top: 24,
                right: WIDTH as i32 - 24,
                bottom: HEIGHT as i32 - 24,
            };
            let height = DrawTextW(
                dc,
                wide.as_ptr(),
                wide.len() as i32,
                &mut rect,
                0x10 | 0x800 | 0x400,
            );
            if height <= 0 || rect.bottom > HEIGHT as i32 - 24 || rect.right > WIDTH as i32 - 24 {
                return Err("英文が画像に収まりません。短い範囲を選んでください。".into());
            }
            let mut rect = Rect {
                left: 24,
                top: 24,
                right: WIDTH as i32 - 24,
                bottom: HEIGHT as i32 - 24,
            };
            if DrawTextW(
                dc,
                wide.as_ptr(),
                wide.len() as i32,
                &mut rect,
                0x10 | 0x800,
            ) == 0
                || GdiFlush() == 0
            {
                return Err("英文を描画できません。".into());
            }
            let mut data = std::slice::from_raw_parts(bits, len).to_vec();
            for p in data.chunks_exact_mut(4) {
                p.swap(0, 2);
                p[3] = 255;
            }
            RgbaImage::from_raw(WIDTH, HEIGHT, data).ok_or_else(|| "画像の寸法が不正です。".into())
        })();
        if old_font != 0 {
            SelectObject(dc, old_font);
        }
        if font != 0 {
            DeleteObject(font);
        }
        SelectObject(dc, old_bitmap);
        DeleteObject(bitmap);
        DeleteDC(dc);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_background_and_ink_survive_as_distinct_files() {
        let mut a = Annotation::default();
        a.text = "Could you make it?\n丸・矢印の対象".into();
        a.freeze(&egui::Context::default()).unwrap();
        let before = a.files().unwrap().1;
        a.strokes = vec![vec![[0.1, 0.1], [0.3, 0.1]]];
        let (ink, background, composite) = a.files().unwrap();
        assert_eq!(before, background);
        assert_ne!(background, composite);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&ink).unwrap()["width"],
            1200
        );
        assert_eq!(image::load_from_memory(&composite).unwrap().width(), 1200);
        a.strokes[0][0][0] = f32::NAN;
        assert!(a.files().is_err());
    }
    #[test]
    fn invalid_or_overflowing_source_is_not_silently_clipped() {
        assert!(render_text("").is_err());
        assert!(render_text("a\0b").is_err());
        assert!(render_text(&"a\n".repeat(200)).is_err());
    }
}
