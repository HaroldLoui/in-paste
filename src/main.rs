// 剪贴板管理器 UI 原型（egui 0.37）
// 初始只显示主卡；点击某行后窗口向右展开，并在右侧显示该行预览。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod fonts;
mod hotkey;
mod icons;
mod model;
mod theme;
mod ui;
mod window;

fn main() -> eframe::Result {
    eframe::run_native(
        "in-paste",
        window::native_options(),
        Box::new(|cc| Ok(Box::new(app::UiApp::new(cc)))),
    )
}
