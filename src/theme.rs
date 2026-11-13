//! 颜色、尺寸、圆角等设计 token。
//! 数值来自设计稿的逐像素测量，改这里就能整体校准观感。
//!
//! 布局模型：窗口 = 卡片（卡片撑满窗口）；展开时窗口右侧再多一块预览卡。
//! 卡片尺寸是状态，窗口尺寸由它推导。

#![allow(dead_code)]

use gpui_kit::Rgba;

// ---------------------------------------------------------------- 窗口 / 卡片

/// 卡片顶部拖拽带高度（同时是工具栏顶部留白）。
pub const CARD_TOP_PAD: f32 = 18.0;
/// 主卡与预览卡之间的间距。
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

// ---------------------------------------------------------------- 工具栏

pub const TABS_CY: f32 = 112.0;
pub const TOOLBAR_PAD_X: f32 = 30.0;
/// 搜索行与 tabs 行之间的留白（也是拖拽带）。
pub const TOOLBAR_GAP: f32 = 12.0;
pub const TAB_FIRST: f32 = 44.0;
pub const TAB_GAP: f32 = 72.0;

// ---------------------------------------------------------------- 颜色助手

/// 避免与 gpui 的 `rgb(hex: u32)` 冲突，命名为 `rgb8`/`rgba8`。
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
pub const SWITCH_ON: Tup = (0x2E, 0x7B, 0xFD);
pub const SWITCH_OFF: Tup = (0xD3, 0xD6, 0xE6);

/// `(r,g,b)` → `Rgba`
pub fn c(t: Tup) -> Rgba {
    rgb8(t.0, t.1, t.2)
}

/// `(r,g,b,a)` → `Rgba`
pub fn ca(t: Tup, a: u8) -> Rgba {
    rgba8(t.0, t.1, t.2, a)
}

/// 由卡片尺寸推导窗口尺寸。
pub fn window_size(card_w: f32, card_h: f32, expanded: bool) -> (f32, f32) {
    let extra_w = if expanded {
        PREVIEW_GAP + PREVIEW_W
    } else {
        0.0
    };
    (card_w + extra_w, card_h)
}
