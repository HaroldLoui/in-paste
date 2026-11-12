//! 设计 token —— 从 in-paste/src/theme.rs 原样迁移。
//! 后期调样式只动这个文件。
//! 注意：颜色助手命名为 `rgb8`/`rgba8`，避免与 gpui 的 `rgb(hex: u32)` 冲突。
#![allow(dead_code)]

use gpui_kit::Rgba;

// ---------------------------------------------------------------- 窗口 / 卡片

pub const MARGIN_X: f32 = 24.0;
pub const MARGIN_TOP: f32 = 40.0;
pub const MARGIN_BOTTOM: f32 = 64.0;
pub const PREVIEW_GAP: f32 = 20.0;

pub const DEFAULT_CARD_W: f32 = 672.0;
pub const DEFAULT_CARD_H: f32 = 816.0;
pub const MIN_CARD_W: f32 = 460.0;
pub const MIN_CARD_H: f32 = 420.0;

pub const CARD_R: f32 = 20.0;
pub const PREVIEW_W: f32 = 431.0;
pub const PREVIEW_H: f32 = 240.0;
pub const PREVIEW_R: f32 = 18.0;

// ---------------------------------------------------------------- 列表行

pub const LIST_TOP: f32 = 146.0;
pub const LIST_BOTTOM_PAD: f32 = 13.0;
pub const ROW_INSET: f32 = 10.0;
pub const ROW_H: f32 = 81.0;
pub const ROW0_H: f32 = 90.0;
pub const TILE: f32 = 64.0;
pub const HOVER_BTN: f32 = 34.0;

// ---------------------------------------------------------------- 颜色

pub fn rgb8(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

pub type Tup = (u8, u8, u8);

pub const CARD_BG: Tup = (0xF6, 0xF6, 0xFD);
pub const PREVIEW_BG: Tup = (0xF5, 0xF5, 0xFD);
pub const SELECT: Tup = (0xC6, 0xD9, 0xFE);
pub const INK: Tup = (0x0B, 0x0D, 0x1A);
pub const BODY: Tup = (0x1F, 0x23, 0x44);
pub const MUTED: Tup = (0x6A, 0x75, 0x97);
pub const PLACEHOLDER: Tup = (0x6B, 0x71, 0x90);
pub const NAVY: Tup = (0x3F, 0x45, 0x68);
pub const ACCENT: Tup = (0x0D, 0x6B, 0xFC);
pub const TILE_SELECTED: Tup = (0xD9, 0xE3, 0xFE);
pub const TILE_NORMAL: Tup = (0xED, 0xEE, 0xF9);
pub const TILE_PDF: Tup = (0xF6, 0xE3, 0xEA);
pub const TILE_HEADER: Tup = (0xD2, 0xE0, 0xFD);
pub const SEARCH_BG: Tup = (0xE6, 0xE7, 0xF5);
pub const DIVIDER: Tup = (0xE6, 0xE7, 0xF2);
pub const PILL: Tup = (0xE5, 0xE7, 0xF4);
pub const RED: Tup = (0xE2, 0x3B, 0x34);

pub const WINDOW_DEBUG_BG: Tup = (0xF3, 0xF2, 0xFD);

/// `(r,g,b)` → `Rgba`
pub fn c(t: Tup) -> Rgba {
    rgb8(t.0, t.1, t.2)
}

/// `(r,g,b,a)` → `Rgba`
pub fn ca(t: Tup, a: u8) -> Rgba {
    rgba8(t.0, t.1, t.2, a)
}
