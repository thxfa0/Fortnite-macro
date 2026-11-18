use crate::theme::*;
use eframe::egui;
use std::f32::consts::{FRAC_PI_2, TAU};

pub fn neon(x: f32) -> egui::Color32 {
    FX.lerp_to_gamma(FX_SOFT, 0.5 + 0.5 * (x * TAU).sin())
}

fn hash(i: u32) -> f32 {
    hash01(i)
}

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

fn ease_in_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

pub fn draw_backdrop(
    p: &egui::Painter,
    full: egui::Rect,
    t: f32,
    k: f32,
    _pointer: Option<egui::Pos2>,
    age: f32,
) {
    let c = egui::pos2(full.center().x, full.top() + TITLEBAR_H + 58.0);

    let breathe = 0.85 + 0.15 * (t * 0.8).sin();
    for i in 0..14 {
        let f = 1.0 - i as f32 / 14.0;
        let a = f * f * (0.012 + 0.030 * k * breathe);
        p.circle_filled(c, 24.0 + i as f32 * 11.0, FX.gamma_multiply(a));
    }

    if k > 0.01 {
        for i in 0..22u32 {
            let h1 = hash(i * 3 + 301);
            let h2 = hash(i * 3 + 302);
            let h3 = hash(i * 3 + 303);
            let ph = (h2 + t * (0.035 + 0.045 * h3)).fract();
            let x = full.left() + 12.0 + h1 * (full.width() - 24.0) + (t * 0.7 + h3 * 6.0).sin() * 5.0;
            let y = full.bottom() - 10.0 - ph * (full.height() - 70.0);
            let a = (ph * std::f32::consts::PI).sin() * 0.65 * k;
            let col = FX.lerp_to_gamma(FX_SOFT, h3);
            p.circle_filled(egui::pos2(x, y), 3.2 + 2.0 * h3, col.gamma_multiply(a * 0.16));
            p.circle_filled(egui::pos2(x, y), 0.8 + 0.9 * h3, col.gamma_multiply(a));
        }
    }

    if k > 0.4 && age >= 0.45 && age < 1.6 {
        let q = ((age - 0.45) / 1.15).clamp(0.0, 1.0);
        let e = ease_out(q);
        for i in 0..7 {
            let f = 1.0 - i as f32 / 7.0;
            p.circle_filled(
                c,
                30.0 + e * 220.0 + i as f32 * 26.0,
                FX.gamma_multiply((1.0 - q).powi(2) * 0.030 * f),
            );
        }
    }
}

fn ignition(p: &egui::Painter, c: egui::Pos2, r: f32, a: f32) {
    if a < 0.62 {
        let q = (a / 0.55).clamp(0.0, 1.0);
        let e = ease_in_out(q);
        let fade = if a > 0.55 { 1.0 - (a - 0.55) / 0.07 } else { 1.0 };

        p.circle_filled(c, r * (0.35 + 0.65 * e), FX.gamma_multiply(0.10 * q * fade));

        let head = -FRAC_PI_2 + e * TAU;
        let trail = 2.6f32;
        let steps = 36;
        let mut prev: Option<egui::Pos2> = None;
        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            let ang = head - u * trail;
            let pt = c + egui::vec2(ang.cos(), ang.sin()) * r;
            if let Some(pp) = prev {
                let col = egui::Color32::WHITE.lerp_to_gamma(FX_SOFT, u);
                p.line_segment(
                    [pp, pt],
                    egui::Stroke::new(3.6f32 - 1.6 * u, col.gamma_multiply((1.0 - u).powi(2) * fade)),
                );
            }
            prev = Some(pt);
        }

    }

    if a >= 0.50 {
        let q = ((a - 0.50) / 1.0).clamp(0.0, 1.0);
        let e = ease_out(q);

        p.circle_filled(c, r + 2.0, egui::Color32::WHITE.gamma_multiply((1.0 - q).powi(3) * 0.30));

        for j in 0..2 {
            let qq = (e - j as f32 * 0.16).clamp(0.0, 1.0);
            if qq <= 0.0 {
                continue;
            }
            let alpha = (1.0 - qq).powi(2) * 0.85;
            p.circle_stroke(
                c,
                r + 6.0 + qq * 58.0,
                egui::Stroke::new(0.8 + 2.8 * (1.0 - qq), FX_SOFT.gamma_multiply(alpha)),
            );
        }

    }
}

fn shutdown(p: &egui::Painter, c: egui::Pos2, r: f32, a: f32) {
    if a >= 0.7 {
        return;
    }
    let q = a / 0.7;
    let e = ease_out(q);
    p.circle_stroke(
        c,
        r + 4.0 + (1.0 - e) * 26.0,
        egui::Stroke::new(
            0.8 + 1.6 * (1.0 - q),
            MUTED_LIGHT.gamma_multiply((1.0 - q).powi(2) * 0.6),
        ),
    );
    p.circle_filled(c, r, FX.gamma_multiply((1.0 - q).powi(2) * 0.10));
}

#[allow(clippy::too_many_arguments)]
pub fn draw_reactor(
    p: &egui::Painter,
    c: egui::Pos2,
    r: f32,
    t: f32,
    k: f32,
    hov: f32,
    live: f32,
    ripple_age: f32,
    enabled: bool,
) {
    if ripple_age >= 0.0 && ripple_age < 1.6 {
        if enabled {
            ignition(p, c, r, ripple_age);
        } else {
            shutdown(p, c, r, ripple_age);
        }
    }

    if hov > 0.01 && k < 0.99 {
        p.circle_filled(c, r + 8.0, FX.gamma_multiply(0.05 * hov * (1.0 - k)));
    }
    if k > 0.01 {
        let breathe = 0.85 + 0.15 * (t * 1.6).sin();
        for i in (0..8).rev() {
            let rad = r + 4.0 + i as f32 * 4.0;
            let a = (0.10 - i as f32 * 0.012).max(0.0) * k * breathe;
            p.circle_filled(c, rad, FX.gamma_multiply(a));
        }
    }

    let hair = BORDER_STRONG.lerp_to_gamma(FX.gamma_multiply(0.5), k);
    p.circle_stroke(c, r + 8.0, egui::Stroke::new(1.0f32, hair));

    if k > 0.01 {
        let head = t * 2.0 - FRAC_PI_2;
        let arc = 1.7f32;
        let steps = 28;
        let mut prev: Option<egui::Pos2> = None;
        for i in 0..=steps {
            let u = i as f32 / steps as f32;
            let ang = head - u * arc;
            let pt = c + egui::vec2(ang.cos(), ang.sin()) * (r + 8.0);
            if let Some(pp) = prev {
                p.line_segment(
                    [pp, pt],
                    egui::Stroke::new(2.2f32, FX_SOFT.gamma_multiply((1.0 - u).powi(2) * k)),
                );
            }
            prev = Some(pt);
        }
    }

    p.circle_stroke(
        c,
        r,
        egui::Stroke::new(3.0 + hov, RING_OFF.lerp_to_gamma(FX, k)),
    );

    p.circle_filled(c, r - 2.0, DISC_OFF.lerp_to_gamma(DISC_ON, k));
    p.circle_stroke(
        c,
        r - 7.0,
        egui::Stroke::new(1.0f32, egui::Color32::from_white_alpha((7.0 + 12.0 * k) as u8)),
    );

    if live > 0.01 {
        p.circle_stroke(
            c,
            r + 15.0,
            egui::Stroke::new(1.2f32, FX_SOFT.gamma_multiply(0.6 * live)),
        );
    }

    draw_power_icon(p, c, r * 0.42, 2.8, MUTED.lerp_to_gamma(egui::Color32::WHITE, k));
}
