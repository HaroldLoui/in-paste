//! 系统托盘：左键显示/隐藏面板，右键菜单提供设置与退出。
//!
//! 菜单和托盘事件由两个后台线程阻塞接收，只写一个原子标记并 `request_repaint`，
//! 真正的动作统一在 `App::logic` 里处理，避免跨线程碰窗口状态。

use std::sync::atomic::{AtomicU8, Ordering};

use eframe::egui::Context;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Toggle,
    Settings,
    Quit,
}

static PENDING: AtomicU8 = AtomicU8::new(0);

fn post(action: Action) {
    let value = match action {
        Action::Toggle => 1,
        Action::Settings => 2,
        Action::Quit => 3,
    };
    PENDING.store(value, Ordering::SeqCst);
}

pub fn take_action() -> Option<Action> {
    match PENDING.swap(0, Ordering::SeqCst) {
        1 => Some(Action::Toggle),
        2 => Some(Action::Settings),
        3 => Some(Action::Quit),
        _ => None,
    }
}

/// 持有 `TrayIcon`，drop 时自动移除托盘图标。
pub struct Tray {
    _icon: TrayIcon,
}

pub fn install(ctx: &Context) -> Option<Tray> {
    let menu = Menu::new();
    let toggle = MenuItem::with_id("toggle", "显示 / 隐藏面板", true, None);
    let settings = MenuItem::with_id("settings", "设置", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id("quit", "退出 in-paste", true, None);
    menu.append_items(&[&toggle, &settings, &separator, &quit])
        .ok()?;

    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("in-paste · Ctrl+Shift+V")
        .with_icon(make_icon()?)
        .with_menu_on_left_click(false)
        .build()
        .ok()?;

    let menu_ctx = ctx.clone();
    std::thread::spawn(move || {
        while let Ok(event) = MenuEvent::receiver().recv() {
            match event.id().as_ref() {
                "toggle" => post(Action::Toggle),
                "settings" => post(Action::Settings),
                "quit" => post(Action::Quit),
                _ => continue,
            }
            menu_ctx.request_repaint();
        }
    });

    let tray_ctx = ctx.clone();
    std::thread::spawn(move || {
        while let Ok(event) = TrayIconEvent::receiver().recv() {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                post(Action::Toggle);
                tray_ctx.request_repaint();
            }
        }
    });

    Some(Tray { _icon: icon })
}

/// 32×32 托盘图标：蓝色圆角底 + 白色剪贴板。
fn make_icon() -> Option<Icon> {
    const N: usize = 32;
    let mut rgba = vec![0u8; N * N * 4];

    let mut pixel = |x: usize, y: usize, color: [u8; 4]| {
        let i = (y * N + x) * 4;
        rgba[i..i + 4].copy_from_slice(&color);
    };

    let blue = [17, 103, 253, 255];
    let white = [255, 255, 255, 255];

    for y in 0..N {
        for x in 0..N {
            if in_round_rect(x as i32, y as i32, 2, 2, 29, 29, 7) {
                pixel(x, y, blue);
            }
        }
    }
    // 纸张
    for y in 9..26 {
        for x in 10..23 {
            if in_round_rect(x, y, 10, 9, 22, 25, 2) {
                pixel(x as usize, y as usize, white);
            }
        }
    }
    // 顶部夹子
    for y in 6..9 {
        for x in 13..20 {
            if in_round_rect(x, y, 13, 6, 19, 8, 1) {
                pixel(x as usize, y as usize, white);
            }
        }
    }
    // 内容线
    for (y, w) in [(13i32, 7usize), (17, 7), (21, 5)] {
        for x in 13..13 + w as i32 {
            pixel(x as usize, y as usize, blue);
            pixel(x as usize, (y + 1) as usize, blue);
        }
    }

    Icon::from_rgba(rgba, N as u32, N as u32).ok()
}

fn in_round_rect(x: i32, y: i32, left: i32, top: i32, right: i32, bottom: i32, r: i32) -> bool {
    if x < left || x > right || y < top || y > bottom {
        return false;
    }
    let cx = x.clamp(left + r, right - r);
    let cy = y.clamp(top + r, bottom - r);
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= r * r
}
