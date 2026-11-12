//! 平台壳：Win32 窗口操作、全局热键、系统托盘。

pub mod hotkey;
pub mod tray;
pub mod windows;

pub use hotkey::Hotkey;
pub use tray::{Action as TrayAction, Tray};
