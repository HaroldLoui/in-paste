//! 全局快捷键：Ctrl + Shift + V 呼出 / 收起面板。
//!
//! `GlobalHotKeyManager` 必须在有 win32 事件循环的线程上创建（这里是主线程，
//! 由 eframe 创建时的 `UiApp::new` 调用）。事件接收放在独立线程里，
//! 只往原子量里写标记并 `request_repaint`，由 `App::logic` 统一处理。

use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::Context;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

/// 快捷键被按下的标记，`App::logic` 里 swap 后处理。
pub static TOGGLE: AtomicBool = AtomicBool::new(false);

/// 持有 manager，`Drop` 时注销热键。
pub struct Hotkey {
    _manager: GlobalHotKeyManager,
}

impl Hotkey {
    pub fn install(ctx: &Context) -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
        manager.register(hotkey).ok()?;
        let id = hotkey.id();

        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            while let Ok(event) = receiver.recv() {
                if event.id == id && event.state == HotKeyState::Pressed {
                    TOGGLE.store(true, Ordering::SeqCst);
                    ctx.request_repaint();
                }
            }
        });

        Some(Self { _manager: manager })
    }
}
