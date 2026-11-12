//! in-paste gpui-kit spike —— 阶段 0 验收工程。
//!
//! 验收清单见 spike/README.md。不接入真实剪贴板，只验证窗口层 + 样式层 + IME。

mod model;
mod platform;
mod theme;
mod ui;

use std::borrow::Cow;

use gpui_kit::component::theme::Theme;
use gpui_kit::*;

use ui::SpikeApp;

actions!(spike, [MoveUp, MoveDown]);

fn main() {
    application().run(|cx| {
        init(cx);

        cx.bind_keys([
            KeyBinding::new("up", MoveUp, None),
            KeyBinding::new("down", MoveDown, None),
        ]);

        register_cjk_fonts(cx);

        Theme::update(cx, |theme| {
            theme.font_family = "Microsoft YaHei".into();
            theme.mono_font_family = "Consolas".into();
            theme.font_size = px(15.);
            // gpui-component 的 WindowState RootPlugin 会把根表面刷成
            // theme.tokens.background（不透明主题底色），盖住透明窗口的
            // 留白区 → 卡片外圈出现白边。置成透明，让留白透出桌面。
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
            let view = cx.new(|cx| SpikeApp::new(window, cx));

            let v = view.clone();
            cx.on_action(move |_: &MoveUp, cx| {
                v.update(cx, |this, cx| this.on_move(-1, cx));
            });
            let v = view.clone();
            cx.on_action(move |_: &MoveDown, cx| {
                v.update(cx, |this, cx| this.on_move(1, cx));
            });

            view
        })
        .expect("failed to open spike window");

        eprintln!("[spike] window opened — see README.md for acceptance checklist");
    });
}

fn register_cjk_fonts(cx: &mut App) {
    let candidates: [&[u8]; 2] = [
        include_bytes!("C:/Windows/Fonts/msyh.ttc"),
        include_bytes!("C:/Windows/Fonts/segoeui.ttf"),
    ];
    let mut ok = 0;
    for bytes in candidates {
        if cx
            .text_system()
            .add_fonts(vec![Cow::Borrowed(bytes)])
            .is_ok()
        {
            ok += 1;
        }
    }
    let families = cx.text_system().all_font_names();
    let has_yahei = families
        .iter()
        .any(|f| f.contains("YaHei") || f.contains("微软雅黑"));
    eprintln!("[spike] fonts registered={ok}, YaHei visible={has_yahei}");
    eprintln!(
        "[spike] font families sample: {:?}",
        &families[..families.len().min(12)]
    );
}
