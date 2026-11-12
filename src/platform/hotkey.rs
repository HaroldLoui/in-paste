//! 全局快捷键：Ctrl + Shift + V 呼出 / 收起面板。
//!
//! `GlobalHotKeyManager` 必须在有 win32 事件循环的线程上创建（主线程）。
//! 事件接收放在独立线程，只写原子标记；主循环用定时任务轮询后处理。

use std::sync::atomic::{AtomicBool, Ordering};

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

/// 快捷键被按下的标记，轮询任务 swap 后处理。
pub static TOGGLE: AtomicBool = AtomicBool::new(false);

/// 持有 manager，`Drop` 时注销热键。
pub struct Hotkey {
    _manager: GlobalHotKeyManager,
}

impl Hotkey {
    pub fn install() -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
        manager.register(hotkey).ok()?;
        let id = hotkey.id();

        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            while let Ok(event) = receiver.recv() {
                if event.id == id && event.state == HotKeyState::Pressed {
                    TOGGLE.store(true, Ordering::SeqCst);
                }
            }
        });

        Some(Self { _manager: manager })
    }
}

/// 轮询热键/托盘标记（由 main 的后台任务调用）。
pub fn take_toggle() -> bool {
    TOGGLE.swap(false, Ordering::SeqCst)
}
