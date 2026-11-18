use crate::config::{
    load_settings, save_settings, Mode, Profile, RuleSet, Settings, DELAY_MAX, DELAY_MIN,
    REPEAT_MAX, REPEAT_MIN,
};
use crate::engine::{
    self, request_toggle, Captured, ACTION_IS_DOWN, ACTION_VK, ACTIVE_PROFILE, APP_ALLOWED,
    ENABLED, REPEAT_MS, TOGGLE_COUNT, TOGGLE_VK, TRIGGER_DOWN, TRIGGER_VK,
};
use crate::bg::GifBackground;
use crate::fx;
use crate::icon::make_icon_rgba;
use crate::keys::{
    is_mouse_vk, key_to_vk, key_width, vk_to_config, vk_to_name, KEY_ROWS, MOUSE_ROW,
    TOGGLE_ROWS, VK_LBUTTON,
};
use crate::system::{is_elevated, list_open_apps, relaunch_elevated};
use crate::theme::*;
use crate::tray::{quit_app, SHOW_REQUEST};
use eframe::egui;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum Picker {
    Trigger,
    Action,
    Toggle,
}

fn key_button(ui: &mut egui::Ui, name: &str, selected: bool, disabled: bool) -> bool {
    let sense = if disabled { egui::Sense::hover() } else { egui::Sense::click() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(key_width(name), 28.0), sense);
    let hov = resp.hovered() && !disabled;
    if hov {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let rgb = egui::Color32::from_rgb;
    let (top, bot) = if selected {
        (rgb(238, 241, 248), rgb(156, 164, 180))
    } else if disabled {
        (rgb(30, 31, 34), rgb(25, 26, 29))
    } else if hov {
        (rgb(86, 88, 96), rgb(46, 48, 54))
    } else {
        (rgb(64, 66, 72), rgb(36, 37, 42))
    };
    let txt = if selected {
        rgb(20, 22, 28)
    } else if disabled {
        BORDER_STRONG
    } else {
        TEXT
    };
    let p = ui.painter();
    rounded_gradient(p, rect, 7.0, top, bot);
    p.rect_stroke(
        rect,
        egui::Rounding::same(7.0),
        egui::Stroke::new(
            1.0f32,
            if selected {
                ACCENT_ALT
            } else if hov {
                ACCENT
            } else {
                BORDER_STRONG
            },
        ),
    );
    if !disabled {
        p.rect_stroke(
            rect.shrink(1.5),
            egui::Rounding::same(5.5),
            egui::Stroke::new(1.0f32, egui::Color32::from_white_alpha(12)),
        );
    }
    p.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        name,
        if name.len() > 2 { bold(11.0) } else { bold(13.0) },
        txt,
    );
    resp.clicked()
}

pub struct BetterFnApp {
    settings: Settings,
    logo: egui::TextureHandle,
    picker: Option<Picker>,
    settings_open: bool,
    apps: Vec<(String, String)>,
    hidden: bool,
    editing: usize,
    elevated: bool,
    cur_h: f32,
    gif_bg: Option<GifBackground>,
    t0: Option<f64>,
    ripple_start: f64,
    last_toggle: u32,
}

impl BetterFnApp {
    pub fn new(ctx: &egui::Context, start_hidden: bool) -> Self {
        let settings = load_settings();
        if settings.enable_on_launch {
            ENABLED.store(true, Ordering::SeqCst);
        }
        engine::start();
        engine::apply_settings(&settings, true);
        crate::tray::start_tray();

        let rgba = make_icon_rgba(64, true);
        let image = egui::ColorImage::from_rgba_unmultiplied([64, 64], &rgba);
        let logo = ctx.load_texture("logo", image, egui::TextureOptions::LINEAR);

        Self {
            settings,
            logo,
            picker: None,
            settings_open: false,
            apps: Vec::new(),
            hidden: start_hidden,
            editing: 0,
            elevated: is_elevated(),
            cur_h: WIN_H,
            gif_bg: GifBackground::load(ctx),
            t0: None,
            ripple_start: -100.0,
            last_toggle: TOGGLE_COUNT.load(Ordering::SeqCst),
        }
    }

    fn rules(&self) -> &RuleSet {
        let n = self.settings.profiles.len();
        if self.editing >= 1 && self.editing <= n {
            &self.settings.profiles[self.editing - 1].rules
        } else {
            &self.settings.rules
        }
    }

    fn rules_mut(&mut self) -> &mut RuleSet {
        let n = self.settings.profiles.len();
        if self.editing >= 1 && self.editing <= n {
            &mut self.settings.profiles[self.editing - 1].rules
        } else {
            &mut self.settings.rules
        }
    }

    fn apply(&self, reset: bool) {
        engine::apply_settings(&self.settings, reset);
    }

    fn desired_height(&self) -> f32 {
        let mut h = WIN_H;
        if self.rules().mode == Mode::Repeat {
            h += 56.0;
        }
        if !self.settings.profiles.is_empty() {
            h += 34.0;
        }
        if self.picker.is_some() || self.settings_open {
            h = h.max(480.0);
        }
        h
    }

    fn save(&self) {
        save_settings(&self.settings);
    }

    fn all_rule_keys(&self) -> Vec<String> {
        let mut keys = vec![
            self.settings.rules.trigger.clone(),
            self.settings.rules.action.clone(),
        ];
        for p in &self.settings.profiles {
            keys.push(p.rules.trigger.clone());
            keys.push(p.rules.action.clone());
        }
        keys
    }

    fn update_key(&mut self, which: Picker, value: String) -> bool {
        let vk = key_to_vk(&value);
        if vk == 0 {
            return false;
        }
        if which == Picker::Toggle && is_mouse_vk(vk) {
            return false;
        }
        if which == Picker::Trigger && vk == VK_LBUTTON {
            return false;
        }
        let others: Vec<String> = match which {
            Picker::Trigger => vec![self.rules().action.clone(), self.settings.toggle_key.clone()],
            Picker::Action => vec![self.rules().trigger.clone(), self.settings.toggle_key.clone()],
            Picker::Toggle => self.all_rule_keys(),
        };
        if others.iter().any(|o| key_to_vk(o) == vk) {
            return false;
        }
        match which {
            Picker::Trigger => self.rules_mut().trigger = value,
            Picker::Action => self.rules_mut().action = value,
            Picker::Toggle => self.settings.toggle_key = value,
        }
        self.apply(true);
        self.save();
        true
    }

    fn ui_titlebar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("titlebar")
            .exact_height(TITLEBAR_H)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let painter = ui.painter().clone();

                let drag = ui.interact(
                    rect,
                    egui::Id::new("tb_drag"),
                    egui::Sense::click_and_drag(),
                );
                if drag.drag_started_by(egui::PointerButton::Primary) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                let logo_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(14.0, (TITLEBAR_H - 20.0) / 2.0),
                    egui::vec2(20.0, 20.0),
                );
                painter.image(
                    self.logo.id(),
                    logo_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                let n1 = painter.text(
                    egui::pos2(logo_rect.right() + 9.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Better",
                    bold(14.0),
                    TEXT,
                );
                let name_rect = painter.text(
                    egui::pos2(n1.right() + 4.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "FN",
                    bold(14.0),
                    ACCENT_LIGHT,
                );

                {
                    let tt = ctx.input(|i| i.time) as f32;
                    let on = ENABLED.load(Ordering::SeqCst);
                    let allowed = APP_ALLOWED.load(Ordering::SeqCst);
                    let led_c = if !on {
                        BORDER_STRONG
                    } else if allowed {
                        GREEN
                    } else {
                        AMBER
                    };
                    let lp = egui::pos2(name_rect.right() + 11.0, rect.center().y + 0.5);
                    if on {
                        let pulse = 0.5 + 0.5 * (tt * 3.0).sin();
                        painter.circle_filled(lp, 5.0 + 4.0 * pulse, led_c.gamma_multiply(0.26));
                    }
                    painter.circle_filled(lp, 3.6, led_c);
                    if on {
                        painter.circle_filled(
                            lp + egui::vec2(-0.9, -0.9),
                            1.1,
                            egui::Color32::from_white_alpha(110),
                        );
                    }
                }

                let btn = |offset: f32| {
                    egui::Rect::from_min_size(
                        egui::pos2(rect.right() - offset, rect.top()),
                        egui::vec2(46.0, TITLEBAR_H),
                    )
                };
                let close_rect = btn(46.0);
                let min_rect = btn(92.0);
                let gear_rect = btn(138.0);

                let close = ui.interact(close_rect, egui::Id::new("tb_close"), egui::Sense::click());
                let minimize = ui.interact(min_rect, egui::Id::new("tb_min"), egui::Sense::click());
                let gear = ui.interact(gear_rect, egui::Id::new("tb_gear"), egui::Sense::click());

                let hover_fill = egui::Color32::from_white_alpha(14);
                if gear.hovered() || self.settings_open {
                    painter.rect_filled(gear_rect, egui::Rounding::ZERO, hover_fill);
                }
                if minimize.hovered() {
                    painter.rect_filled(min_rect, egui::Rounding::ZERO, hover_fill);
                }
                if close.hovered() {
                    painter.rect_filled(
                        close_rect,
                        egui::Rounding {
                            nw: 0.0,
                            ne: WIN_RADIUS,
                            sw: 0.0,
                            se: 0.0,
                        },
                        egui::Color32::from_rgb(92, 96, 108),
                    );
                }

                let gear_k = ctx.animate_bool_with_time(
                    egui::Id::new("gear_rot"),
                    self.settings_open,
                    0.35,
                );
                let gear_k = 1.0 - (1.0 - gear_k).powi(3);
                draw_gear(
                    &painter,
                    gear_rect.center(),
                    if self.settings_open { ACCENT_LIGHT } else { MUTED_LIGHT },
                    gear_k * std::f32::consts::FRAC_PI_4 * 2.0 + if gear.hovered() { 0.12 } else { 0.0 },
                );

                let icon_stroke = egui::Stroke::new(1.3, MUTED_LIGHT);
                let mc = min_rect.center();
                painter.line_segment([mc + egui::vec2(-5.0, 0.0), mc + egui::vec2(5.0, 0.0)], icon_stroke);

                let cc = close_rect.center();
                let close_stroke = egui::Stroke::new(
                    1.3,
                    if close.hovered() { egui::Color32::WHITE } else { MUTED_LIGHT },
                );
                painter.line_segment([cc + egui::vec2(-5.0, -5.0), cc + egui::vec2(5.0, 5.0)], close_stroke);
                painter.line_segment([cc + egui::vec2(-5.0, 5.0), cc + egui::vec2(5.0, -5.0)], close_stroke);

                {
                    let on = ENABLED.load(Ordering::SeqCst);
                    let mid = if on { ACCENT } else { BORDER_STRONG };
                    let top = rect.bottom() - 1.0;
                    let mx = rect.center().x;
                    let left = egui::Rect::from_min_max(
                        egui::pos2(rect.left(), top),
                        egui::pos2(mx, rect.bottom()),
                    );
                    let right = egui::Rect::from_min_max(
                        egui::pos2(mx, top),
                        egui::pos2(rect.right(), rect.bottom()),
                    );
                    gradient_quad(&painter, left, BORDER, mid, mid, BORDER);
                    gradient_quad(&painter, right, mid, BORDER, BORDER, mid);

                    let glow = ctx.animate_bool_with_time(egui::Id::new("sweep_on"), on, 0.5);
                    if glow > 0.01 {
                        let tt = ctx.input(|i| i.time) as f32;
                        let x0 = rect.left() + (tt * 0.32).fract() * (rect.width() + 140.0) - 140.0;
                        for j in 0..24 {
                            let x = x0 + j as f32 * 5.0;
                            if x < rect.left() || x + 5.0 > rect.right() {
                                continue;
                            }
                            let a = (j as f32 / 23.0 * std::f32::consts::PI).sin();
                            painter.rect_filled(
                                egui::Rect::from_min_max(
                                    egui::pos2(x, rect.bottom() - 1.5),
                                    egui::pos2(x + 5.0, rect.bottom()),
                                ),
                                egui::Rounding::ZERO,
                                ACCENT_ALT.gamma_multiply(0.85 * a * glow),
                            );
                        }
                    }
                }

                if gear.clicked() {
                    self.settings_open = !self.settings_open;
                    self.picker = None;
                    if self.settings_open {
                        self.apps = list_open_apps();
                    }
                }
                if minimize.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                if close.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    self.hidden = true;
                    self.picker = None;
                    self.settings_open = false;
                }
            });
    }

    fn ui_hero(&mut self, ui: &mut egui::Ui) {
        let enabled = ENABLED.load(Ordering::SeqCst);
        let waiting = enabled && !APP_ALLOWED.load(Ordering::SeqCst);
        let t = ui.input(|i| i.time) as f32;
        let k = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new("power_on"), enabled, 0.35);
        let trigger = vk_to_name(TRIGGER_VK.load(Ordering::SeqCst));
        let action = vk_to_name(ACTION_VK.load(Ordering::SeqCst));
        let toggle = vk_to_name(TOGGLE_VK.load(Ordering::SeqCst));
        let target = self.settings.target_app.clone();
        let mode = engine::current_mode();
        let repeat_ms = REPEAT_MS.load(Ordering::SeqCst);
        let slot = ACTIVE_PROFILE.load(Ordering::SeqCst);
        let profile_name = if slot >= 1 && slot <= self.settings.profiles.len() {
            Some(self.settings.profiles[slot - 1].app.clone())
        } else {
            None
        };
        let ripple_age = (ui.input(|i| i.time) - self.ripple_start) as f32;

        ui.vertical_centered(|ui| {
            let (rect, resp) =
                ui.allocate_exact_size(egui::vec2(96.0, 96.0), egui::Sense::click());
            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            let hov = ui
                .ctx()
                .animate_bool_with_time(egui::Id::new("power_hover"), resp.hovered(), 0.12);
            let press = ui.ctx().animate_bool_with_time(
                egui::Id::new("power_press"),
                resp.is_pointer_button_down_on(),
                0.08,
            );

            let p = ui.painter().clone();
            let c = rect.center();
            let r = 28.0 + 1.5 * hov - 2.5 * press;

            let live = ACTION_IS_DOWN.load(Ordering::SeqCst) as u8 as f32;
            fx::draw_reactor(&p, c, r, t, k, hov, live, ripple_age, enabled);

            if resp.clicked() {
                request_toggle();
            }

            ui.add_space(4.0);
            let (status, sub, color) = if !enabled {
                (
                    "Stopped".to_string(),
                    format!("Press {} or tap the button", toggle),
                    MUTED_LIGHT.lerp_to_gamma(GREEN, k),
                )
            } else if waiting {
                (
                    "Waiting".to_string(),
                    format!("Active only in {}", target),
                    AMBER,
                )
            } else {
                let sub = match mode {
                    Mode::Hold => format!("Holding {} presses {}", trigger, action),
                    Mode::Repeat => {
                        format!("Holding {} taps {} every {} ms", trigger, action, repeat_ms)
                    }
                };
                let status = match &profile_name {
                    Some(name) => format!("Running · {}", name),
                    None => "Running".to_string(),
                };
                (status, sub, MUTED_LIGHT.lerp_to_gamma(GREEN, k))
            };

            let galley = ui
                .painter()
                .layout_no_wrap(status.to_string(), bold(13.0), color);
            let chip_w = galley.size().x + 36.0;
            let (chip, _) = ui.allocate_exact_size(egui::vec2(chip_w, 26.0), egui::Sense::hover());
            let p = ui.painter();
            p.rect_filled(chip, egui::Rounding::same(13.0), color.gamma_multiply(0.14));
            p.rect_stroke(
                chip,
                egui::Rounding::same(13.0),
                egui::Stroke::new(1.0, color.gamma_multiply(0.45)),
            );
            let dot = egui::pos2(chip.left() + 14.0, chip.center().y);
            if enabled && !waiting {
                let pulse = 0.5 + 0.5 * (t * 3.0).sin();
                p.circle_filled(dot, 3.5 + 3.0 * pulse, color.gamma_multiply(0.25));
            }
            p.circle_filled(dot, 3.5, color);
            let tp = egui::pos2(chip.left() + 24.0, chip.center().y - galley.size().y / 2.0);
            p.galley(tp, galley, color);

            ui.add_space(5.0);
            ui.label(egui::RichText::new(sub).font(reg(12.0)).color(MUTED));
        });
    }

    fn key_cap(&mut self, ui: &mut egui::Ui, which: Picker) {
        let (key_name, caption) = match which {
            Picker::Trigger => (self.rules().trigger.clone(), "Trigger"),
            _ => (self.rules().action.clone(), "Action"),
        };
        let label = vk_to_name(key_to_vk(&key_name));
        let label_font = if label.chars().count() > 5 { bold(13.0) } else { bold(19.0) };

        let width = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 54.0), egui::Sense::click());
        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        let open = self.picker == Some(which);
        let hovered = resp.hovered();

        let live = match which {
            Picker::Trigger => TRIGGER_DOWN.load(Ordering::SeqCst),
            _ => ACTION_IS_DOWN.load(Ordering::SeqCst),
        };
        let key_id = match which {
            Picker::Trigger => "cap_t",
            _ => "cap_a",
        };
        let lk = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new((key_id, "live")), live, 0.07);
        let hk = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new((key_id, "hov")), hovered, 0.12);
        let dy = (if resp.is_pointer_button_down_on() { 2.0 } else { 0.0 }) + 2.5 * lk - 1.5 * hk;

        let cw = width * 0.80;
        let cap = egui::Rect::from_min_size(
            rect.min + egui::vec2((width - cw) * 0.5, 1.5),
            egui::vec2(cw, 40.0),
        );
        let round = egui::Rounding::same(10.0);
        let on = ENABLED.load(Ordering::SeqCst);
        let ek = ui
            .ctx()
            .animate_bool_with_time(egui::Id::new((key_id, "on")), on, 0.25);
        let rgb = egui::Color32::from_rgb;
        let p = ui.painter().clone();
        let face = cap.translate(egui::vec2(0.0, dy));

        let base = cap.translate(egui::vec2(0.0, 4.0));
        p.rect_filled(base, egui::Rounding::same(12.0), rgb(11, 11, 13));
        p.rect_stroke(base, egui::Rounding::same(12.0), egui::Stroke::new(1.0f32, BORDER));

        if lk > 0.01 {
            for i in 1..6 {
                let e = i as f32 * 3.0;
                p.rect_filled(
                    face.expand(e),
                    egui::Rounding::same(11.0 + e),
                    FX.gamma_multiply(0.14 * lk * (1.0 - i as f32 / 6.0)),
                );
            }
        }

        let top = rgb(66, 68, 74)
            .lerp_to_gamma(rgb(86, 88, 96), hk)
            .lerp_to_gamma(rgb(72, 76, 90), ek * (1.0 - hk))
            .lerp_to_gamma(rgb(150, 158, 178), lk);
        let bot = rgb(36, 37, 42)
            .lerp_to_gamma(rgb(46, 48, 54), hk)
            .lerp_to_gamma(rgb(40, 43, 54), ek * (1.0 - hk))
            .lerp_to_gamma(rgb(84, 90, 108), lk);
        rounded_gradient(&p, face, 10.0, top, bot);

        p.rect_stroke(
            face.shrink(1.5),
            egui::Rounding::same(9.5),
            egui::Stroke::new(
                1.0f32,
                egui::Color32::from_white_alpha((10.0 + 14.0 * hk + 24.0 * lk) as u8),
            ),
        );
        let border = if open {
            FX
        } else {
            BORDER_STRONG
                .lerp_to_gamma(FX_SOFT.gamma_multiply(0.75), ek * 0.75)
                .lerp_to_gamma(FX_SOFT, hk)
                .lerp_to_gamma(FX, lk)
        };
        p.rect_stroke(face, round, egui::Stroke::new(1.2 + 0.6 * lk, border));

        p.text(
            face.center() + egui::vec2(0.0, 1.5),
            egui::Align2::CENTER_CENTER,
            label.clone(),
            label_font.clone(),
            egui::Color32::from_black_alpha(150),
        );
        p.text(
            face.center(),
            egui::Align2::CENTER_CENTER,
            label,
            label_font,
            TEXT.lerp_to_gamma(egui::Color32::WHITE, lk),
        );
        p.text(
            egui::pos2(cap.center().x, rect.top() + 50.0),
            egui::Align2::CENTER_CENTER,
            caption.to_uppercase(),
            bold(10.0),
            MUTED,
        );

        if resp.clicked() {
            self.picker = if open { None } else { Some(which) };
        }
    }

    fn ui_panel(&mut self, ui: &mut egui::Ui) {
        let total_w = ui.available_width();
        let enabled = ENABLED.load(Ordering::SeqCst);

        if !self.settings.profiles.is_empty() {
            let mut pick: Option<usize> = None;
            egui::ScrollArea::horizontal()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        if pill_tab(ui, "Default", self.editing == 0).clicked() {
                            pick = Some(0);
                        }
                        for (i, p) in self.settings.profiles.iter().enumerate() {
                            if pill_tab(ui, &p.app, self.editing == i + 1).clicked() {
                                pick = Some(i + 1);
                            }
                        }
                    });
                });
            if let Some(i) = pick {
                self.editing = i;
                self.picker = None;
            }
            ui.add_space(8.0);
        }

        let rules = self.rules().clone();

        egui::Frame::none()
            .fill(PANEL.gamma_multiply(GLASS_MAIN)) 
            .rounding(egui::Rounding::same(16.0))
            .stroke(egui::Stroke::new(1.0, BORDER))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(total_w - 26.0);
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                ui.columns(3, |cols| {
                    self.key_cap(&mut cols[0], Picker::Trigger);
                    arrow_cell(&mut cols[1], enabled);
                    self.key_cap(&mut cols[2], Picker::Action);
                });

                divider(ui);

                let labels: Vec<&str> = Mode::ALL.iter().map(|m| m.label()).collect();
                let selected = Mode::ALL.iter().position(|m| *m == rules.mode).unwrap_or(0);
                if let Some(i) = segmented(ui, "mode", &labels, selected) {
                    self.rules_mut().mode = Mode::ALL[i];
                    self.apply(true);
                    self.save();
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Delay").font(bold(13.0)).color(TEXT));
                    ui.label(
                        egui::RichText::new("Lower is faster")
                            .font(reg(11.0))
                            .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        value_chip(ui, &format!("{} ms", rules.delay_ms));
                    });
                });
                ui.add_space(8.0);
                let slider_w = ui.available_width();
                let mut delay = rules.delay_ms;
                let out = fancy_slider(ui, &mut delay, DELAY_MIN, DELAY_MAX, slider_w, true);
                if out.changed {
                    self.rules_mut().delay_ms = delay;
                    self.apply(false);
                }
                if out.released {
                    self.save();
                }

                if rules.mode == Mode::Repeat {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Interval").font(bold(13.0)).color(TEXT));
                        ui.label(
                            egui::RichText::new("Time between taps")
                                .font(reg(11.0))
                                .color(MUTED),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            value_chip(ui, &format!("{} ms", rules.repeat_ms));
                        });
                    });
                    ui.add_space(8.0);
                    let slider_w = ui.available_width();
                    let mut interval = rules.repeat_ms;
                    let out =
                        fancy_slider(ui, &mut interval, REPEAT_MIN, REPEAT_MAX, slider_w, true);
                    if out.changed {
                        self.rules_mut().repeat_ms = interval;
                        self.apply(false);
                    }
                    if out.released {
                        self.save();
                    }
                }
            });
    }

    fn ui_footer(&mut self, ctx: &egui::Context) {
        let toggle = vk_to_name(TOGGLE_VK.load(Ordering::SeqCst));
        egui::TopBottomPanel::bottom("footer")
            .exact_height(30.0)
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(18.0, 0.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} toggles. Closing hides to tray.", toggle))
                            .font(reg(11.0))
                            .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let quit = ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Quit").font(reg(12.0)).color(RED_SOFT),
                                )
                                .frame(false),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if quit.clicked() {
                            quit_app();
                        }
                    });
                });
            });
    }

    fn ui_overlay(&mut self, ctx: &egui::Context) {
        let open = self.picker.is_some() || self.settings_open;
        let kraw = ctx.animate_bool_with_time(egui::Id::new("overlay_anim"), open, 0.22);
        if !open && kraw <= 0.001 {
            return;
        }
        let k = 1.0 - (1.0 - kraw).powi(3);
        let screen = ctx.screen_rect();
        let mut close = false;

        egui::Area::new(egui::Id::new("overlay_dim"))
            .order(egui::Order::Middle)
            .fixed_pos(egui::pos2(0.0, TITLEBAR_H))
            .movable(false)
            .show(ctx, |ui| {
                let size = egui::vec2(screen.width(), screen.height() - TITLEBAR_H);
                let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                ui.painter().rect_filled(
                    rect,
                    egui::Rounding {
                        nw: 0.0,
                        ne: 0.0,
                        sw: WIN_RADIUS,
                        se: WIN_RADIUS,
                    },
                    egui::Color32::from_black_alpha((95.0 * kraw) as u8),
                );
                if resp.clicked() && open {
                    close = true;
                }
            });

        if let Some(target) = self.picker {
            self.ui_picker_sheet(ctx, target, k);
        } else if self.settings_open {
            self.ui_settings_sheet(ctx, k);
        }

        if open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close = true;
        }
        if close {
            if self.picker.is_some() {
                self.picker = None;
            } else {
                self.settings_open = false;
            }
        }
    }

    fn ui_picker_sheet(&mut self, ctx: &egui::Context, target: Picker, k: f32) {
        let mut chosen: Option<String> = None;
        let mut capture_click = false;
        let rules = self.rules().clone();

        let (current, others, title, subtitle) = match target {
            Picker::Trigger => (
                rules.trigger.clone(),
                vec![rules.action.clone(), self.settings.toggle_key.clone()],
                "Choose the trigger",
                "The key or mouse button you hold down",
            ),
            Picker::Action => (
                rules.action.clone(),
                vec![rules.trigger.clone(), self.settings.toggle_key.clone()],
                "Choose the action",
                "The key or mouse button pressed for you",
            ),
            Picker::Toggle => (
                self.settings.toggle_key.clone(),
                self.all_rule_keys(),
                "Choose the toggle key",
                "Turns the app on and off",
            ),
        };
        let current_vk = key_to_vk(&current);
        let other_vks: Vec<u32> = others.iter().map(|o| key_to_vk(o)).collect();
        let rows: &[&[&str]] = if target == Picker::Toggle { TOGGLE_ROWS } else { KEY_ROWS };
        let qwerty = target != Picker::Toggle;
        let armed = engine::capture_armed();

        egui::Area::new(egui::Id::new("picker_sheet"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(16.0, TITLEBAR_H + 12.0 + (1.0 - k) * 22.0))
            .movable(false)
            .show(ctx, |ui| {
                ui.set_opacity(k);
                let blur_slot = ui.painter().add(egui::Shape::Noop);
                let sheet = egui::Frame::none()
                    .fill(PANEL.gamma_multiply(GLASS_SHEET))
                    .rounding(egui::Rounding::same(16.0))
                    .stroke(egui::Stroke::new(1.0, BORDER_STRONG))
                    .inner_margin(egui::Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.set_width(WIN_W - 62.0);
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        ui.label(egui::RichText::new(title).font(bold(15.0)).color(TEXT));
                        ui.label(egui::RichText::new(subtitle).font(reg(12.0)).color(MUTED));
                        ui.add_space(8.0);

                        for (ri, row) in rows.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let indent = if qwerty {
                                    match ri {
                                        1 => 14.0,
                                        2 => 30.0,
                                        _ => 0.0,
                                    }
                                } else {
                                    0.0
                                };
                                if indent > 0.0 {
                                    ui.add_space(indent);
                                }
                                for name in row.iter() {
                                    let vk = key_to_vk(name);
                                    let disabled = other_vks.contains(&vk);
                                    if key_button(ui, name, vk == current_vk, disabled) {
                                        chosen = Some(name.to_string());
                                    }
                                }
                            });
                        }

                        if qwerty {
                            ui.horizontal(|ui| {
                                for name in MOUSE_ROW.iter() {
                                    let vk = key_to_vk(name);
                                    let disabled = other_vks.contains(&vk)
                                        || (target == Picker::Trigger && vk == VK_LBUTTON);
                                    if key_button(ui, name, vk == current_vk, disabled) {
                                        chosen = Some(name.to_string());
                                    }
                                }
                            });
                        }

                        ui.add_space(6.0);
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 30.0),
                            egui::Sense::click(),
                        );
                        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                        let t = ui.input(|i| i.time) as f32;
                        let pulse = if armed { 0.5 + 0.5 * (t * 4.0).sin() } else { 0.0 };
                        let edge = if armed {
                            0.5 + 0.5 * pulse
                        } else if resp.hovered() {
                            1.0
                        } else {
                            0.0
                        };
                        let text = if !armed {
                            "Press any key…"
                        } else if target == Picker::Toggle {
                            "Press a key…  (Esc cancels)"
                        } else {
                            "Press a key or mouse button…  (Esc cancels)"
                        };
                        let p = ui.painter();
                        p.rect_filled(
                            rect,
                            egui::Rounding::same(8.0),
                            if armed {
                                ACCENT_DIM
                            } else if resp.hovered() {
                                KEYCAP_HOVER
                            } else {
                                KEYCAP
                            },
                        );
                        p.rect_stroke(
                            rect,
                            egui::Rounding::same(8.0),
                            egui::Stroke::new(1.0, BORDER_STRONG.lerp_to_gamma(ACCENT, edge)),
                        );
                        p.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            bold(12.0),
                            if armed { egui::Color32::WHITE } else { MUTED_LIGHT },
                        );
                        if resp.clicked() {
                            capture_click = true;
                        }
                        if armed {
                            ui.ctx().request_repaint();
                        }
                    });

                if let Some(g) = &self.gif_bg {
                    let full = ui.ctx().screen_rect();
                    let now = ui.ctx().input(|i| i.time);
                    ui.painter().set(
                        blur_slot,
                        g.blurred_shape(full, sheet.response.rect, 16.0, now, 0.94 * k),
                    );
                }
            });

        if capture_click {
            if armed {
                engine::stop_capture();
            } else {
                engine::start_capture();
            }
        }
        if let Some(name) = chosen {
            engine::stop_capture();
            if self.update_key(target, name) {
                self.picker = None;
            }
        }
    }

    fn ui_settings_sheet(&mut self, ctx: &egui::Context, k: f32) {
        egui::Area::new(egui::Id::new("settings_sheet"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(16.0, TITLEBAR_H + 12.0 + (1.0 - k) * 22.0))
            .movable(false)
            .show(ctx, |ui| {
                ui.set_opacity(k);
                let blur_slot = ui.painter().add(egui::Shape::Noop);
                let sheet = egui::Frame::none()
                    .fill(PANEL.gamma_multiply(GLASS_SHEET))
                    .rounding(egui::Rounding::same(16.0))
                    .stroke(egui::Stroke::new(1.0, BORDER_STRONG))
                    .inner_margin(egui::Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.set_width(WIN_W - 62.0);
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);

                        ui.label(egui::RichText::new("Settings").font(bold(15.0)).color(TEXT));
                        ui.add_space(6.0);

                        egui::ScrollArea::vertical()
                            .max_height(ctx.screen_rect().height() - TITLEBAR_H - 24.0 - 28.0 - 34.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                self.settings_rows(ui);
                            });
                    });

                if let Some(g) = &self.gif_bg {
                    let full = ui.ctx().screen_rect();
                    let now = ui.ctx().input(|i| i.time);
                    ui.painter().set(
                        blur_slot,
                        g.blurred_shape(full, sheet.response.rect, 16.0, now, 0.94 * k),
                    );
                }
            });
    }

    fn settings_rows(&mut self, ui: &mut egui::Ui) {
        setting_row(ui, "Turn on at launch", "Starts already active", |ui| {
            let mut v = self.settings.enable_on_launch;
            if switch(ui, &mut v) {
                self.settings.enable_on_launch = v;
                self.save();
            }
        });

        let admin_desc = if self.elevated {
            "Running as administrator"
        } else {
            "Needed for games started as admin"
        };
        setting_row(ui, "Run as administrator", admin_desc, |ui| {
            let mut v = self.settings.run_as_admin;
            if switch(ui, &mut v) {
                if v && !self.elevated {
                    self.settings.run_as_admin = true;
                    self.save();
                    if relaunch_elevated(false) {
                        quit_app();
                    }
                    self.settings.run_as_admin = false;
                    self.save();
                } else {
                    self.settings.run_as_admin = v;
                    self.save();
                }
            }
        });

        setting_row(ui, "Sound", "Plays a chime when switched on or off", |ui| {
            let mut v = self.settings.sound;
            if switch(ui, &mut v) {
                self.settings.sound = v;
                self.apply(false);
                self.save();
            }
        });

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let on = self.settings.sound;
            let (ir, _) = ui.allocate_exact_size(egui::vec2(22.0, 24.0), egui::Sense::hover());
            draw_speaker(
                ui.painter(),
                ir.center(),
                self.settings.volume as f32 / 100.0,
                if on { ACCENT_LIGHT } else { MUTED },
                !on || self.settings.volume == 0,
            );
            let slider_w = (ui.available_width() - 48.0).max(60.0);
            let mut vol = self.settings.volume;
            let out = fancy_slider(ui, &mut vol, 0, 100, slider_w, on);
            if out.changed {
                self.settings.volume = vol;
                self.apply(false);
            }
            if out.released {
                self.save();
                if on {
                    crate::sound::play_toggle_sound(true); 
                }
            }
            let (br, _) = ui.allocate_exact_size(egui::vec2(40.0, 24.0), egui::Sense::hover());
            ui.painter().text(
                br.right_center(),
                egui::Align2::RIGHT_CENTER,
                format!("{}%", self.settings.volume),
                bold(12.0),
                if on { ACCENT_LIGHT } else { MUTED },
            );
        });
        ui.add_space(4.0);

        setting_row(ui, "Corner overlay", "Shows ON / OFF on screen after a switch", |ui| {
            let mut v = self.settings.overlay;
            if switch(ui, &mut v) {
                self.settings.overlay = v;
                self.apply(false);
                self.save();
            }
        });

        setting_row(ui, "Toggle key", "Turns the app on and off", |ui| {
            let name = vk_to_name(TOGGLE_VK.load(Ordering::SeqCst));
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(74.0, 28.0), egui::Sense::click());
            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            let hov = resp.hovered();
            let p = ui.painter();
            p.rect_filled(
                rect,
                egui::Rounding::same(8.0),
                if hov { KEYCAP_HOVER } else { KEYCAP },
            );
            p.rect_stroke(
                rect,
                egui::Rounding::same(8.0),
                egui::Stroke::new(1.0, if hov { ACCENT } else { BORDER_STRONG }),
            );
            p.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                name,
                bold(12.0),
                TEXT,
            );
            if resp.clicked() {
                self.picker = Some(Picker::Toggle);
            }
        });

        setting_row(ui, "Only in one app", "Works when that app is in front", |ui| {
            let mut v = self.settings.restrict_app;
            if switch(ui, &mut v) {
                self.settings.restrict_app = v;
                if v {
                    self.apps = list_open_apps();
                }
                self.apply(false);
                self.save();
            }
        });

        if self.settings.restrict_app {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let combo_w = (ui.available_width() - 78.0).max(120.0);
                let current = if self.settings.target_app.is_empty() {
                    "Choose an app".to_string()
                } else {
                    self.settings.target_app.clone()
                };
                let mut picked: Option<String> = None;
                egui::ComboBox::from_id_salt("app_pick")
                    .selected_text(egui::RichText::new(current).font(reg(13.0)).color(TEXT))
                    .width(combo_w)
                    .show_ui(ui, |ui| {
                        for (exe, title) in self.apps.iter() {
                            let sel = exe.eq_ignore_ascii_case(&self.settings.target_app);
                            let r = ui
                                .selectable_label(sel, exe.as_str())
                                .on_hover_text(title.as_str());
                            if r.clicked() {
                                picked = Some(exe.clone());
                            }
                        }
                    });
                let refresh = ui
                    .add(egui::Button::new(
                        egui::RichText::new("Refresh").font(reg(12.0)).color(MUTED_LIGHT),
                    ))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if refresh.clicked() {
                    self.apps = list_open_apps();
                }
                if let Some(exe) = picked {
                    self.settings.target_app = exe;
                    self.apply(false);
                    self.save();
                }
            });
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("App profiles").font(bold(13.0)).color(TEXT));
        ui.label(
            egui::RichText::new("Own keys, mode and delay for a program, switched automatically")
                .font(reg(11.0))
                .color(MUTED),
        );
        ui.add_space(2.0);

        let mut remove: Option<usize> = None;
        let mut edit: Option<usize> = None;
        for (i, p) in self.settings.profiles.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(p.app.as_str()).font(reg(13.0)).color(TEXT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let rm = ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Remove").font(reg(12.0)).color(RED_SOFT),
                            )
                            .frame(false),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if rm.clicked() {
                        remove = Some(i);
                    }
                    let ed = ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Edit").font(reg(12.0)).color(ACCENT_LIGHT),
                            )
                            .frame(false),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if ed.clicked() {
                        edit = Some(i);
                    }
                });
            });
        }

        let mut add: Option<String> = None;
        ui.horizontal(|ui| {
            let combo_w = (ui.available_width() - 78.0).max(120.0);
            egui::ComboBox::from_id_salt("profile_add")
                .selected_text(egui::RichText::new("Add profile…").font(reg(13.0)).color(TEXT))
                .width(combo_w)
                .show_ui(ui, |ui| {
                    for (exe, title) in self.apps.iter() {
                        if self
                            .settings
                            .profiles
                            .iter()
                            .any(|p| p.app.eq_ignore_ascii_case(exe))
                        {
                            continue;
                        }
                        if ui
                            .selectable_label(false, exe.as_str())
                            .on_hover_text(title.as_str())
                            .clicked()
                        {
                            add = Some(exe.clone());
                        }
                    }
                });
            let refresh = ui
                .add(egui::Button::new(
                    egui::RichText::new("Refresh").font(reg(12.0)).color(MUTED_LIGHT),
                ))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if refresh.clicked() {
                self.apps = list_open_apps();
            }
        });

        if let Some(i) = remove {
            self.settings.profiles.remove(i);
            if self.editing == i + 1 {
                self.editing = 0;
            } else if self.editing > i + 1 {
                self.editing -= 1;
            }
            self.apply(true);
            self.save();
        }
        if let Some(i) = edit {
            self.editing = i + 1;
            self.settings_open = false;
        }
        if let Some(exe) = add {
            let rules = self.settings.rules.clone();
            self.settings.profiles.push(Profile { app: exe, rules });
            self.editing = self.settings.profiles.len();
            self.apply(true);
            self.save();
            self.settings_open = false;
        }
    }
}

impl Drop for BetterFnApp {
    fn drop(&mut self) {
        engine::shutdown();
    }
}
impl eframe::App for BetterFnApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let full = ctx.screen_rect();
        let now = ctx.input(|i| i.time);
        let tf = now as f32;
        let t0 = *self.t0.get_or_insert(now);

        let tc = TOGGLE_COUNT.load(Ordering::SeqCst);
        if tc != self.last_toggle {
            self.last_toggle = tc;
            self.ripple_start = now;
        }

        {
            let bg = ctx.layer_painter(egui::LayerId::background());
            bg.rect_filled(full, egui::Rounding::same(WIN_RADIUS), BG);
            if let Some(g) = &self.gif_bg {
                g.draw(&bg, full, WIN_RADIUS, now);
            }
            let glow_k = ctx.animate_bool_with_time(
                egui::Id::new("aurora"),
                ENABLED.load(Ordering::SeqCst),
                0.6,
            );
            let aurora = egui::Rect::from_min_size(
                full.min + egui::vec2(0.0, TITLEBAR_H),
                egui::vec2(full.width(), 240.0),
            );
            let top = FX_SOFT.gamma_multiply(0.020 + 0.050 * glow_k);
            gradient_quad(&bg, aurora, top, top, egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT);

            let pointer = ctx.input(|i| i.pointer.hover_pos());
            fx::draw_backdrop(&bg, full, tf, glow_k, pointer, (now - self.ripple_start) as f32);

            bg.rect_filled(
                egui::Rect::from_min_size(full.min, egui::vec2(full.width(), TITLEBAR_H)),
                egui::Rounding {
                    nw: WIN_RADIUS,
                    ne: WIN_RADIUS,
                    sw: 0.0,
                    se: 0.0,
                },
                BG_DEEP,
            );
        }

        if SHOW_REQUEST.swap(false, Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            self.hidden = false;
            self.t0 = Some(now); 
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.hidden = true;
            self.picker = None;
            self.settings_open = false;
        }
        if ctx.input(|i| i.viewport().focused) == Some(true) {
            self.hidden = false;
        }

        if let Some(Captured::Key(vk)) = engine::poll_capture() {
            if let Some(target) = self.picker {
                if self.update_key(target, vk_to_config(vk)) {
                    self.picker = None;
                }
            }
        }
        if self.picker.is_none() && engine::capture_armed() {
            engine::stop_capture();
        }
        if self.editing > self.settings.profiles.len() {
            self.editing = 0;
        }

        let want = self.desired_height();
        if (want - self.cur_h).abs() > 0.5 {
            self.cur_h = want;
            let size = egui::vec2(WIN_W, want);
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(egui::vec2(WIN_W, 2000.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(WIN_W, 100.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(size));
        }

        self.ui_titlebar(ctx);
        self.ui_footer(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(16.0, 10.0)))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                self.ui_hero(ui);
                ui.add_space(10.0);
                self.ui_panel(ui);
            });

        self.ui_overlay(ctx);

        let intro = ((now - t0) / 0.6).clamp(0.0, 1.0) as f32;
        if intro < 1.0 {
            let e = 1.0 - (1.0 - intro).powi(3);
            ctx.layer_painter(egui::LayerId::new(
                egui::Order::Tooltip,
                egui::Id::new("intro_fade"),
            ))
            .rect_filled(full, egui::Rounding::same(WIN_RADIUS), BG_DEEP.gamma_multiply(1.0 - e));
            ctx.request_repaint();
        }

        ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("win_border"),
        ))
        .rect_stroke(
            full.shrink(0.5),
            egui::Rounding::same(WIN_RADIUS),
            egui::Stroke::new(1.0, BORDER_STRONG),
        );

        let ms = if self.hidden {
            500
        } else if ENABLED.load(Ordering::SeqCst) || now - self.ripple_start < 1.7 {
            16
        } else {
            50
        };
        let ms = if !self.hidden {
            match &self.gif_bg {
                Some(g) => ms.min(g.frame_ms().max(16.0) as u64),
                None => ms,
            }
        } else {
            ms
        };
        ctx.request_repaint_after(Duration::from_millis(ms));
    }
}

fn value_chip(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(ACCENT_DIM)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(9.0, 3.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .font(bold(12.0))
                    .color(ACCENT_LIGHT),
            );
        });
}

