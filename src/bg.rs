use eframe::egui;
use image::{codecs::gif::GifDecoder, imageops, AnimationDecoder};
use std::f32::consts::{FRAC_PI_2, PI};

static GIF_BYTES: &[u8] = include_bytes!("../assets/background.gif");

pub const TINT: egui::Color32 = egui::Color32::from_rgb(152, 155, 166);
pub const DIM_TOP: f32 = 0.30;
pub const DIM_BOTTOM: f32 = 0.20;

pub struct GifBackground {
    frames: Vec<(egui::TextureHandle, f32)>,
    blurred: Vec<egui::TextureHandle>,
    total_ms: f32,
}

impl GifBackground {
    pub fn load(ctx: &egui::Context) -> Option<Self> {
        let decoder = GifDecoder::new(std::io::Cursor::new(GIF_BYTES)).ok()?;
        let frames = decoder.into_frames().collect_frames().ok()?;
        if frames.is_empty() {
            return None;
        }
        let mut out = Vec::with_capacity(frames.len());
        let mut blurred = Vec::with_capacity(frames.len());
        let mut total = 0.0f32;
        for (i, f) in frames.iter().enumerate() {
            let (num, den) = f.delay().numer_denom_ms();
            let mut ms = if den == 0 { 100.0 } else { num as f32 / den as f32 };
            if ms < 20.0 {
                ms = 100.0; 
            }
            let buf = f.buffer();
            let (w, h) = buf.dimensions();
            let img = egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                buf.as_raw(),
            );
            let tex = ctx.load_texture(format!("bg_gif_{i}"), img, egui::TextureOptions::LINEAR);
            out.push((tex, ms));

            let small = imageops::resize(
                buf,
                (w / 6).max(8),
                (h / 6).max(8),
                imageops::FilterType::Triangle,
            );
            let small = imageops::blur(&small, 1.2);
            let (sw, sh) = small.dimensions();
            let simg = egui::ColorImage::from_rgba_unmultiplied(
                [sw as usize, sh as usize],
                small.as_raw(),
            );
            blurred.push(ctx.load_texture(
                format!("bg_gif_blur_{i}"),
                simg,
                egui::TextureOptions::LINEAR,
            ));
            total += ms;
        }
        Some(Self { frames: out, blurred, total_ms: total.max(1.0) })
    }

    pub fn frame_ms(&self) -> f32 {
        self.total_ms / self.frames.len() as f32
    }

    fn frame_and_uv(&self, full: egui::Rect, now: f64) -> (usize, [f32; 4]) {
        let t_ms = ((now * 1000.0) % self.total_ms as f64) as f32;
        let mut acc = 0.0;
        let mut idx = self.frames.len() - 1;
        for (i, (_, d)) in self.frames.iter().enumerate() {
            if t_ms < acc + d {
                idx = i;
                break;
            }
            acc += d;
        }
        let [tw, th] = self.frames[idx].0.size();
        let (tw, th) = (tw as f32, th as f32);

        let win_aspect = full.width() / full.height().max(1.0);
        let img_aspect = tw / th.max(1.0);
        let drift = (now as f32 * 0.12).sin();
        let (u0, u1, v0, v1) = if win_aspect < img_aspect {
            let f = win_aspect / img_aspect;
            let room = 1.0 - f;
            let c = 0.5 + drift * room * 0.25;
            (c - f * 0.5, c + f * 0.5, 0.0, 1.0)
        } else {
            let f = img_aspect / win_aspect;
            let room = 1.0 - f;
            let c = 0.5 + drift * room * 0.25;
            (0.0, 1.0, c - f * 0.5, c + f * 0.5)
        };
        (idx, [u0, v0, u1, v1])
    }

    pub fn draw(&self, p: &egui::Painter, full: egui::Rect, radius: f32, now: f64) {
        let (idx, uv) = self.frame_and_uv(full, now);
        let tex = &self.frames[idx].0;
        p.add(rounded_image(tex.id(), full, full, radius, uv, TINT));

        crate::theme::rounded_gradient(
            p,
            full,
            radius,
            crate::theme::BG.gamma_multiply(DIM_TOP),
            crate::theme::BG.gamma_multiply(DIM_BOTTOM),
        );
    }

    pub fn blurred_shape(
        &self,
        full: egui::Rect,
        region: egui::Rect,
        radius: f32,
        now: f64,
        alpha: f32,
    ) -> egui::Shape {
        let (idx, uv) = self.frame_and_uv(full, now);
        let tex = &self.blurred[idx];
        rounded_image(tex.id(), region, full, radius, uv, TINT.gamma_multiply(alpha))
    }
}

fn rounded_image(
    tex: egui::TextureId,
    rect: egui::Rect,
    map: egui::Rect,
    r: f32,
    uv: [f32; 4],
    tint: egui::Color32,
) -> egui::Shape {
    let steps = 8;
    let r = r.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let corners = [
        (egui::pos2(rect.right() - r, rect.top() + r), -FRAC_PI_2, 0.0),
        (egui::pos2(rect.right() - r, rect.bottom() - r), 0.0, FRAC_PI_2),
        (egui::pos2(rect.left() + r, rect.bottom() - r), FRAC_PI_2, PI),
        (egui::pos2(rect.left() + r, rect.top() + r), PI, PI + FRAC_PI_2),
    ];
    let mut pts: Vec<egui::Pos2> = Vec::new();
    for (c, a0, a1) in corners {
        for i in 0..=steps {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32;
            pts.push(c + egui::vec2(a.cos(), a.sin()) * r);
        }
    }
    let uv_at = |pos: egui::Pos2| {
        let fx = ((pos.x - map.left()) / map.width().max(1.0)).clamp(0.0, 1.0);
        let fy = ((pos.y - map.top()) / map.height().max(1.0)).clamp(0.0, 1.0);
        egui::pos2(uv[0] + (uv[2] - uv[0]) * fx, uv[1] + (uv[3] - uv[1]) * fy)
    };
    let mut mesh = egui::Mesh::with_texture(tex);
    let center = rect.center();
    mesh.vertices.push(egui::epaint::Vertex { pos: center, uv: uv_at(center), color: tint });
    for pt in &pts {
        mesh.vertices.push(egui::epaint::Vertex { pos: *pt, uv: uv_at(*pt), color: tint });
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    egui::Shape::mesh(mesh)
}
