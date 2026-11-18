use eframe::egui;

pub const WIN_W: f32 = 360.0;
pub const WIN_H: f32 = 424.0; 
pub const TITLEBAR_H: f32 = 38.0;
pub const WIN_RADIUS: f32 = 14.0;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub const BG: egui::Color32 = egui::Color32::from_rgb(11, 11, 13);
pub const BG_DEEP: egui::Color32 = egui::Color32::from_rgb(8, 8, 10);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(34, 36, 42);
pub const MAGENTA: egui::Color32 = egui::Color32::from_rgb(226, 231, 240);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(60, 62, 70);
pub const BORDER_STRONG: egui::Color32 = egui::Color32::from_rgb(98, 101, 112);
pub const KEYCAP: egui::Color32 = egui::Color32::from_rgb(30, 31, 35);
pub const KEYCAP_HOVER: egui::Color32 = egui::Color32::from_rgb(42, 43, 49);
pub const KEY_LIP: egui::Color32 = egui::Color32::from_rgb(7, 7, 8);
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(206, 212, 224);
pub const ACCENT_LIGHT: egui::Color32 = egui::Color32::from_rgb(246, 248, 252);
pub const ACCENT_DIM: egui::Color32 = egui::Color32::from_rgb(58, 61, 70);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(240, 242, 246);
pub const GLASS_MAIN: f32 = 0.45;
pub const GLASS_SHEET: f32 = 0.62;
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(122, 126, 136);
pub const MUTED_LIGHT: egui::Color32 = egui::Color32::from_rgb(190, 194, 204);
pub const GREEN: egui::Color32 = egui::Color32::from_rgb(52, 211, 120);
pub const AMBER: egui::Color32 = egui::Color32::from_rgb(255, 176, 66);
pub const RED_SOFT: egui::Color32 = egui::Color32::from_rgb(160, 166, 178);
pub const DISC_OFF: egui::Color32 = egui::Color32::from_rgb(23, 24, 27);
pub const DISC_ON: egui::Color32 = egui::Color32::from_rgb(34, 37, 46);
pub const FX: egui::Color32 = egui::Color32::from_rgb(238, 242, 250);
pub const FX_SOFT: egui::Color32 = egui::Color32::from_rgb(168, 180, 202);
pub const RING_OFF: egui::Color32 = egui::Color32::from_rgb(104, 107, 118);
pub const ACCENT_ALT: egui::Color32 = egui::Color32::from_rgb(196, 204, 220);

pub fn reg(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Proportional)
}

pub fn bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("bold".into()))
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let win = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let load = |file: &str| std::fs::read(format!("{}\\Fonts\\{}", win, file)).ok();

    if let Some(data) = load("segoeui.ttf") {
        fonts
            .font_data
            .insert("segoe".to_owned(), egui::FontData::from_owned(data).into());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe".to_owned());
    }

    let mut bold_chain: Vec<String> = Vec::new();
    if let Some(data) = load("seguisb.ttf").or_else(|| load("segoeuib.ttf")) {
        fonts.font_data.insert(
            "segoe-semibold".to_owned(),
            egui::FontData::from_owned(data).into(),
        );
        bold_chain.push("segoe-semibold".to_owned());
    }
    let proportional = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    bold_chain.extend(proportional);
    fonts
        .families
        .insert(egui::FontFamily::Name("bold".into()), bold_chain);

    ctx.set_fonts(fonts);
}

pub fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();

    v.window_fill = PANEL;
    v.panel_fill = BG;
    v.extreme_bg_color = KEYCAP;
    v.faint_bg_color = PANEL;
    v.window_rounding = egui::Rounding::same(14.0);
    v.window_stroke = egui::Stroke::new(1.0, BORDER);
    v.selection.bg_fill = egui::Color32::from_rgb(84, 88, 100);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT_LIGHT);

    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.weak_bg_fill = PANEL;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT);

    v.widgets.inactive.bg_fill = KEYCAP;
    v.widgets.inactive.weak_bg_fill = KEYCAP;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_STRONG);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT);

    v.widgets.hovered.bg_fill = KEYCAP_HOVER;
    v.widgets.hovered.weak_bg_fill = KEYCAP_HOVER;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    v.widgets.active.bg_fill = ACCENT_DIM;
    v.widgets.active.weak_bg_fill = ACCENT_DIM;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT);

    v.widgets.open.bg_fill = KEYCAP;
    v.widgets.open.weak_bg_fill = KEYCAP;

    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.rounding = egui::Rounding::same(8.0);
    }

    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.interaction.selectable_labels = false;

    use egui::{FontFamily, FontId, TextStyle};
    style.text_styles.insert(TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small, FontId::new(11.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace));

    ctx.set_style(style);
}

pub fn draw_power_icon(p: &egui::Painter, c: egui::Pos2, r: f32, w: f32, color: egui::Color32) {
    let stroke = egui::Stroke::new(w, color);
    let start = (-90.0f32 + 38.0).to_radians();
    let end = (270.0f32 - 38.0).to_radians();
    let steps = 28;
    let mut pts: Vec<egui::Pos2> = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let a = start + (end - start) * (i as f32 / steps as f32);
        pts.push(c + egui::vec2(a.cos(), a.sin()) * r);
    }
    let first = pts[0];
    let last = pts[pts.len() - 1];
    p.add(egui::Shape::line(pts, stroke));

    let top = c + egui::vec2(0.0, -r - 2.0);
    let mid = c + egui::vec2(0.0, -2.0);
    p.line_segment([top, mid], stroke);
    for cap in [first, last, top, mid] {
        p.circle_filled(cap, w / 2.0, color);
    }
}

pub fn draw_gear(p: &egui::Painter, c: egui::Pos2, color: egui::Color32, rot: f32) {
    let teeth = egui::Stroke::new(2.6, color);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::FRAC_PI_4 + rot;
        let d = egui::vec2(a.cos(), a.sin());
        p.line_segment([c + d * 5.2, c + d * 8.2], teeth);
    }
    p.circle_stroke(c, 4.4, egui::Stroke::new(2.6, color));
}

pub fn gradient_quad(
    p: &egui::Painter,
    rect: egui::Rect,
    tl: egui::Color32,
    tr: egui::Color32,
    br: egui::Color32,
    bl: egui::Color32,
) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), tl);
    mesh.colored_vertex(rect.right_top(), tr);
    mesh.colored_vertex(rect.right_bottom(), br);
    mesh.colored_vertex(rect.left_bottom(), bl);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(egui::Shape::mesh(mesh));
}

pub fn draw_speaker(p: &egui::Painter, c: egui::Pos2, level: f32, color: egui::Color32, muted: bool) {
    let body = egui::Rect::from_min_max(c + egui::vec2(-8.0, -3.0), c + egui::vec2(-4.0, 3.0));
    p.rect_filled(body, egui::Rounding::same(1.0), color);
    p.add(egui::Shape::convex_polygon(
        vec![
            c + egui::vec2(-4.0, -3.0),
            c + egui::vec2(0.5, -6.5),
            c + egui::vec2(0.5, 6.5),
            c + egui::vec2(-4.0, 3.0),
        ],
        color,
        egui::Stroke::NONE,
    ));
    if muted {
        let o = c + egui::vec2(7.0, 0.0);
        let st = egui::Stroke::new(1.8, RED_SOFT);
        p.line_segment([o + egui::vec2(-3.5, -3.5), o + egui::vec2(3.5, 3.5)], st);
        p.line_segment([o + egui::vec2(-3.5, 3.5), o + egui::vec2(3.5, -3.5)], st);
        return;
    }
    let center = c + egui::vec2(1.5, 0.0);
    let stroke = egui::Stroke::new(1.6, color);
    for (i, rad) in [4.5f32, 8.0f32].iter().enumerate() {
        let need = if i == 0 { 0.0 } else { 0.5 };
        if level < need {
            break;
        }
        let pts: Vec<egui::Pos2> = (0..=8)
            .map(|st| {
                let a = (-48.0f32 + 96.0 * st as f32 / 8.0).to_radians();
                center + egui::vec2(a.cos(), a.sin()) * *rad
            })
            .collect();
        p.add(egui::Shape::line(pts, stroke));
    }
}

pub struct SliderOut {
    pub changed: bool,
    pub released: bool,
}

pub fn fancy_slider(
    ui: &mut egui::Ui,
    value: &mut u32,
    min: u32,
    max: u32,
    width: f32,
    active: bool,
) -> SliderOut {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 24.0), egui::Sense::click_and_drag());
    let resp = if active {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    };
    let pad = 10.0;
    let l = rect.left() + pad;
    let r = rect.right() - pad;
    let span = (max - min).max(1) as f32;
    let mut out = SliderOut { changed: false, released: false };

    if active && (resp.dragged() || resp.clicked()) {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t = ((pos.x - l) / (r - l)).clamp(0.0, 1.0);
            let v = (min as f32 + t * span).round() as u32;
            if v != *value {
                *value = v;
                out.changed = true;
            }
        }
    }
    if active && (resp.drag_stopped() || resp.clicked()) {
        out.released = true;
    }

    let t = ((*value as f32 - min as f32) / span).clamp(0.0, 1.0);
    let kx = lerp(l, r, t);
    let cy = rect.center().y;
    let hot = active && (resp.hovered() || resp.dragged());
    let k = ui.ctx().animate_bool_with_time(resp.id, hot, 0.12);
    let p = ui.painter();

    let track = egui::Rect::from_min_max(egui::pos2(l - 4.0, cy - 3.0), egui::pos2(r + 4.0, cy + 3.0));
    p.rect_filled(track, egui::Rounding::same(3.0), BG_DEEP);
    p.rect_stroke(track, egui::Rounding::same(3.0), egui::Stroke::new(1.0, BORDER));
    let fill = egui::Rect::from_min_max(track.min, egui::pos2(kx.max(track.min.x + 6.0), track.max.y));
    p.rect_filled(
        fill,
        egui::Rounding::same(3.0),
        if active { ACCENT } else { RING_OFF },
    );

    let knob = egui::pos2(kx, cy);
    if active {
        for i in 0..3 {
            p.circle_filled(
                knob,
                9.0 + i as f32 * 3.0,
                ACCENT.gamma_multiply((0.10 + 0.10 * k) - i as f32 * 0.03),
            );
        }
    }
    p.circle_filled(knob, 7.5 + k, if active { egui::Color32::WHITE } else { MUTED_LIGHT });
    p.circle_stroke(
        knob,
        7.5 + k,
        egui::Stroke::new(2.0, if active { ACCENT } else { RING_OFF }),
    );
    out
}

pub fn divider(ui: &mut egui::Ui) {
    ui.add_space(7.0);
    let width = ui.available_width();
    let (r, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(r, egui::Rounding::ZERO, BORDER);
    ui.add_space(7.0);
}

pub fn hash01(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B1);
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^= x >> 16;
    (x & 0xFFFF) as f32 / 65535.0
}

pub fn draw_arc(p: &egui::Painter, c: egui::Pos2, r: f32, a0: f32, a1: f32, stroke: egui::Stroke) {
    let steps = 22;
    let pts: Vec<egui::Pos2> = (0..=steps)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32;
            c + egui::vec2(a.cos(), a.sin()) * r
        })
        .collect();
    p.add(egui::Shape::line(pts, stroke));
}

pub fn draw_particles(p: &egui::Painter, area: egui::Rect, t: f32, k: f32) {
    for i in 0..34u32 {
        let h1 = hash01(i * 3 + 1);
        let h2 = hash01(i * 3 + 2);
        let h3 = hash01(i * 3 + 3);
        let speed = 0.010 + 0.022 * h3 + 0.018 * k;
        let y = 1.0 - (h2 + t * speed).fract();
        let x = (h1 + (t * 0.35 + h3 * 6.28).sin() * 0.025).clamp(0.02, 0.98);
        let fade = (y * std::f32::consts::PI).sin();
        let tw = 0.65 + 0.35 * (t * (1.2 + h1 * 2.0) + h2 * 9.0).sin();
        let a = fade * tw * (0.16 + 0.34 * k);
        let pos = egui::pos2(
            area.left() + x * area.width(),
            area.top() + y * area.height(),
        );
        let r = 0.8 + 1.6 * h3;
        p.circle_filled(pos, r * 3.2, ACCENT.gamma_multiply(a * 0.18));
        let col = if h1 > 0.7 { ACCENT_ALT } else { ACCENT_LIGHT };
        p.circle_filled(pos, r, col.gamma_multiply(a));
    }
}

pub fn arrow_cell(ui: &mut egui::Ui, enabled: bool) {
    use std::f32::consts::PI;
    let t = ui.input(|i| i.time) as f32;
    let ctx = ui.ctx().clone();
    let ek = ctx.animate_bool_with_time(egui::Id::new("arrow_on"), enabled, 0.4);

    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 54.0), egui::Sense::hover());
    let c = egui::pos2(rect.center().x, rect.top() + 21.5);
    let half = (width * 0.5 - 2.0).clamp(20.0, 44.0);
    let x0 = -half + 7.0; 
    let x1 = half - 5.0; 
    let p = ui.painter();
    let at = |x: f32, y: f32| c + egui::vec2(x, y);
    let gauss = |d: f32, sigma: f32| (-(d * d) / (2.0 * sigma * sigma)).exp();

    const SPEED: f32 = 0.50;
    const SPAN: f32 = 1.6;
    let pulses = [
        (t * SPEED).rem_euclid(1.0) * SPAN - 0.3,
        (t * SPEED + 0.5).rem_euclid(1.0) * SPAN - 0.3,
    ];

    let samples = 64;
    let base_col = RING_OFF.lerp_to_gamma(FX_SOFT, 0.55 * ek);
    for i in 0..samples {
        let u = i as f32 / (samples - 1) as f32;
        let edge = (u * PI).sin().powf(0.6);
        let glow = pulses.iter().map(|&pp| gauss(u - pp, 0.085)).fold(0.0f32, f32::max) * ek;
        let col = base_col.lerp_to_gamma(egui::Color32::WHITE, glow * 0.85);
        let a = (0.40 + 0.25 * ek + 0.75 * glow).min(1.0) * edge;
        let x = lerp(x0, x1, u);
        if glow > 0.05 {
            p.circle_filled(at(x, 0.0), 3.6 + 2.2 * glow, FX_SOFT.gamma_multiply(0.07 * glow));
        }
        p.circle_filled(at(x, 0.0), 0.75 + 0.9 * glow, col.gamma_multiply(a));
    }

    let breath = 0.5 + 0.5 * (t * 1.2).sin();
    let node = RING_OFF.lerp_to_gamma(FX, ek);
    if ek > 0.01 {
        p.circle_filled(at(-half, 0.0), 6.5 + 1.5 * breath, FX.gamma_multiply(0.06 * ek));
    }
    p.circle_stroke(at(-half, 0.0), 3.2, egui::Stroke::new(1.4f32, node));
    p.circle_filled(at(-half, 0.0), 1.1, node.gamma_multiply(0.8));

    let arrival = pulses.iter().map(|&pp| gauss(1.0 - pp, 0.10)).fold(0.0f32, f32::max) * ek;
    let head_col = RING_OFF
        .lerp_to_gamma(FX, ek * 0.85)
        .lerp_to_gamma(egui::Color32::WHITE, arrival * 0.7);
    if ek > 0.01 {
        p.circle_filled(
            at(half - 2.0, 0.0),
            9.0 + 4.0 * arrival,
            FX.gamma_multiply((0.05 + 0.10 * arrival) * ek),
        );
    }
    let st = egui::Stroke::new(2.2f32, head_col);
    let wing = 6.5f32;
    let back = half - 7.5;
    p.line_segment([at(back, -wing), at(half, 0.0)], st);
    p.line_segment([at(half, 0.0), at(back, wing)], st);
    for pt in [at(back, -wing), at(half, 0.0), at(back, wing)] {
        p.circle_filled(pt, 1.1, head_col);
    }
}

pub fn switch(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(40.0, 22.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let mut changed = false;
    if resp.clicked() {
        *on = !*on;
        changed = true;
    }
    let k = ui.ctx().animate_bool_with_time(resp.id, *on, 0.15);
    let hk = ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.12);
    let p = ui.painter();
    if k > 0.01 {
        p.rect_filled(
            rect.expand(2.0 + 2.0 * k),
            egui::Rounding::same(14.0),
            ACCENT.gamma_multiply(0.14 * k),
        );
    }
    p.rect_filled(rect, egui::Rounding::same(11.0), BORDER_STRONG.lerp_to_gamma(ACCENT, k));
    p.rect_stroke(
        rect,
        egui::Rounding::same(11.0),
        egui::Stroke::new(1.0, ACCENT_LIGHT.gamma_multiply(0.15 + 0.5 * hk)),
    );
    let knob_x = lerp(rect.left() + 11.0, rect.right() - 11.0, k);
    let kc = egui::pos2(knob_x, rect.center().y);
    p.circle_filled(kc + egui::vec2(0.0, 1.5), 8.0, egui::Color32::from_black_alpha(60));
    p.circle_filled(
        kc,
        8.0 + 0.8 * hk,
        egui::Color32::WHITE.lerp_to_gamma(egui::Color32::from_rgb(26, 28, 34), k),
    );
    changed
}

pub fn setting_row(ui: &mut egui::Ui, title: &str, desc: &str, right: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, 40.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(egui::RichText::new(title).font(bold(13.0)).color(TEXT));
                ui.label(egui::RichText::new(desc).font(reg(11.0)).color(MUTED));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
        },
    );
}


pub fn segmented(ui: &mut egui::Ui, salt: &str, labels: &[&str], selected: usize) -> Option<usize> {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::hover());
    let n = labels.len().max(1);
    let seg_w = rect.width() / n as f32;
    let p = ui.painter().clone();
    p.rect_filled(rect, egui::Rounding::same(9.0), BG_DEEP);
    p.rect_stroke(rect, egui::Rounding::same(9.0), egui::Stroke::new(1.0, BORDER));

    let mut clicked = None;
    for (i, label) in labels.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            egui::pos2(rect.left() + seg_w * i as f32, rect.top()),
            egui::vec2(seg_w, rect.height()),
        )
        .shrink(2.0);
        let resp = ui
            .interact(r, egui::Id::new((salt, "seg", i)), egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        let sel = i == selected;
        let hk = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new((salt, "seg_hov", i)), resp.hovered(), 0.12);
        let sk = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new((salt, "seg_sel", i)), sel, 0.15);
        p.rect_filled(
            r,
            egui::Rounding::same(7.0),
            BG_DEEP.lerp_to_gamma(KEYCAP_HOVER, hk * 0.8),
        );
        if sk > 0.01 {
            rounded_gradient(
                &p,
                r,
                7.0,
                egui::Color32::from_rgb(238, 241, 248).gamma_multiply(sk),
                egui::Color32::from_rgb(156, 164, 180).gamma_multiply(sk),
            );
        }
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            bold(12.0),
            MUTED_LIGHT
                .lerp_to_gamma(TEXT, hk)
                .lerp_to_gamma(egui::Color32::from_rgb(20, 22, 28), sk),
        );
        if resp.clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

pub fn pill_tab(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), bold(12.0), TEXT);
    let size = egui::vec2(galley.size().x + 24.0, 26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hk = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("pill_hov"), resp.hovered(), 0.12);
    let sk = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("pill_sel"), selected, 0.15);
    let p = ui.painter();
    p.rect_filled(
        rect,
        egui::Rounding::same(13.0),
        KEYCAP.lerp_to_gamma(KEYCAP_HOVER, hk).lerp_to_gamma(ACCENT_DIM, sk),
    );
    p.rect_stroke(
        rect,
        egui::Rounding::same(13.0),
        egui::Stroke::new(1.0, BORDER_STRONG.lerp_to_gamma(ACCENT, (hk * 0.6).max(sk))),
    );
    p.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        bold(12.0),
        MUTED_LIGHT.lerp_to_gamma(TEXT, hk.max(sk)),
    );
    resp
}


pub fn draw_grid(p: &egui::Painter, area: egui::Rect, t: f32, k: f32) {
    let step = 22.0;
    let rows = (area.height() / step) as i32 + 2;
    let cols = (area.width() / step) as i32 + 1;
    let scroll = (t * (3.0 + 9.0 * k)) % step;
    for r in -1..rows {
        let y = area.top() + r as f32 * step + scroll;
        if y < area.top() || y > area.bottom() {
            continue;
        }
        let v = (y - area.top()) / area.height().max(1.0);
        let a = v * v * (0.16 + 0.22 * k);
        let off = if r % 2 == 0 { 0.0 } else { step * 0.5 };
        for c in 0..cols {
            let x = area.left() + c as f32 * step + off;
            if x > area.right() {
                continue;
            }
            p.circle_filled(egui::pos2(x, y), 1.0 + 0.5 * k, ACCENT_LIGHT.gamma_multiply(a));
        }
    }
}


pub fn rounded_gradient(
    p: &egui::Painter,
    rect: egui::Rect,
    r: f32,
    top: egui::Color32,
    bottom: egui::Color32,
) {
    use std::f32::consts::{FRAC_PI_2, PI};
    let steps = 6;
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
    let h = rect.height().max(1.0);
    let col = |y: f32| top.lerp_to_gamma(bottom, ((y - rect.top()) / h).clamp(0.0, 1.0));
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.center(), col(rect.center().y));
    for pt in &pts {
        mesh.colored_vertex(*pt, col(pt.y));
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(egui::Shape::mesh(mesh));
}
