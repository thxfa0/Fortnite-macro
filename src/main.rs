#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bg;
mod config;
mod engine;
mod fx;
mod icon;
mod keys;
mod overlay;
mod sound;
mod system;
mod theme;
mod tray;
mod ui;

use eframe::egui;
use theme::{WIN_H, WIN_W};

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let start_hidden = args.iter().any(|a| a == "--tray");
    let restarted = args.iter().any(|a| a == "--restart");

    if !system::ensure_single_instance(start_hidden, restarted) {
        return Ok(());
    }

    if config::load_settings().run_as_admin
        && !system::is_elevated()
        && system::relaunch_elevated(start_hidden)
    {
        return Ok(());
    }

    let icon = egui::IconData {
        rgba: icon::make_icon_rgba(64, true),
        width: 64,
        height: 64,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Better FN")
            .with_inner_size([WIN_W, WIN_H])
            .with_min_inner_size([WIN_W, WIN_H])
            .with_max_inner_size([WIN_W, WIN_H])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_visible(!start_hidden)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Better FN",
        options,
        Box::new(move |cc| {
            engine::set_ctx(cc.egui_ctx.clone());
            theme::setup_fonts(&cc.egui_ctx);
            theme::setup_style(&cc.egui_ctx);
            Ok(Box::new(ui::BetterFnApp::new(&cc.egui_ctx, start_hidden)))
        }),
    )
}
