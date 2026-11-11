// 剪贴板管理器 UI 原型（egui 0.37）
// 初始只显示主卡；点击某行后窗口向右展开，并在右侧显示该行预览。
//
// 始终用 GUI 子系统：debug 构建如果走 console 子系统，会额外出现一个控制台窗口，
// 它位于置顶面板后面；面板留白是透明的，控制台标题栏会从那条留白里透出来，
// 焦点切换时看起来就像「窗口装饰闪一下」。
#![windows_subsystem = "windows"]

mod app;
mod fonts;
mod hotkey;
mod icons;
mod model;
mod theme;
mod tray;
mod ui;
mod window;

fn main() -> eframe::Result {
    eframe::run_native(
        "in-paste",
        window::native_options(),
        Box::new(|cc| Ok(Box::new(app::UiApp::new(cc)))),
    )
}
