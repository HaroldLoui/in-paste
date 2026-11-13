//! in-paste —— Windows 呼出式剪贴板管理器（gpui-kit 重写版）。
//!
//! 架构：
//! - `core`     纯逻辑（记录模型 / 过滤）
//! - `platform` 平台壳（Win32 / 热键 / 托盘）
//! - `theme`    设计 token（调样式只动这里）
//! - `ui`       gpui-kit 视图
#![windows_subsystem = "windows"]

mod core;
mod platform;
mod theme;
mod ui;

use std::borrow::Cow;

use gpui_kit::component::theme::Theme;
use gpui_kit::*;

use crate::ui::icons::AppAssets;

fn main() {
    application().with_assets(AppAssets).run(|cx| {
        init(cx);
        ui::app::bind_keys(cx);

        register_fonts(cx);
        Theme::update(cx, |theme| {
            theme.font_family = "Microsoft YaHei".into();
            theme.mono_font_family = "Consolas".into();
            theme.font_size = px(16.);
            // gpui-component RootPlugin 会给根表面刷不透明主题底色，
            // 盖住透明窗口留白 → 置为透明。
            let clear = theme.transparent;
            theme.background = clear;
            theme.tokens.background = clear.into();
        });

        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(720.), px(920.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("in-paste".into()),
                appears_transparent: true,
                traffic_light_position: None,
            }),
            window_background: WindowBackgroundAppearance::Transparent,
            focus: true,
            show: true,
            is_movable: true,
            is_resizable: false,
            is_minimizable: false,
            window_min_size: Some(size(px(508.), px(484.))),
            ..Default::default()
        };

        open_window(opts, cx, |window, cx| {
            window.set_window_title("in-paste");
            let view = cx.new(|cx| ui::app::UiApp::new(window, cx));
            ui::app::wire_actions(&view, cx);
            ui::app::spawn_poll(&view, cx);
            view
        })
        .expect("failed to open window");

        eprintln!("[in-paste] gpui rewrite started");
    });
}

/// 加载系统 CJK 字体；失败不致命（DirectWrite 仍可解析已装字体）。
fn register_fonts(cx: &mut App) {
    let candidates: [&[u8]; 2] = [
        include_bytes!("C:/Windows/Fonts/msyh.ttc"),
        include_bytes!("C:/Windows/Fonts/segoeui.ttf"),
    ];
    for bytes in candidates {
        let _ = cx.text_system().add_fonts(vec![Cow::Borrowed(bytes)]);
    }
    let families = cx.text_system().all_font_names();
    let has_yahei = families
        .iter()
        .any(|f| f.contains("YaHei") || f.contains("微软雅黑"));
    eprintln!("[in-paste] YaHei visible={has_yahei}");
}
