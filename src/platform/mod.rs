//! 平台壳：Win32 窗口操作、全局热键、系统托盘。
//!
//! UI 阶段：托盘与置顶在用；热键暂未接（业务后接），模块保留。

#![allow(dead_code)]

pub mod hotkey;
pub mod tray;
pub mod windows;

pub use tray::{Action as TrayAction, Tray};
