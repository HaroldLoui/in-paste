//! 颜色、尺寸、圆角等设计 token。
//! 数值来自设计稿的逐像素测量，改这里就能整体校准观感。
//!
//! 布局模型：窗口 = 卡片 + 四周留白；展开时窗口右侧再多一块预览卡
//! （留白间距 + 预览卡宽度）。卡片尺寸是状态，窗口尺寸由它推导，
//! 这样窗口被用户拖大时卡片会跟着变宽。

use eframe::egui::{Color32, Vec2, vec2};

// ---------------------------------------------------------------- 窗口 / 卡片

/// 界面整体缩放系数（相对设计稿点数）。
///
/// 0.667 ≈ 缩小 1/3：窗口的原生逻辑尺寸、以及 egui 的绘制尺寸都按这个系数收，
/// 但内部布局仍用设计稿坐标（如 720×920 点），改这里即可整体放大/缩小。
pub const UI_SCALE: f32 = 2.0 / 3.0;

/// 窗口左右留白（承载卡片阴影，也是自定义缩放的拖拽热区）。
pub const MARGIN_X: f32 = 24.0;
/// 窗口顶部留白。
pub const MARGIN_TOP: f32 = 40.0;
/// 窗口底部留白（比顶部大，给阴影和预览卡留空间）。
pub const MARGIN_BOTTOM: f32 = 64.0;
/// 主卡与预览卡之间的间距。
pub const PREVIEW_GAP: f32 = 20.0;

/// 默认卡片宽（设计点）。
pub const DEFAULT_CARD_W: f32 = 672.0;
/// 默认卡片高：列表起点 146 + 首行 90 + 7×81 + 底部留白 13。
pub const DEFAULT_CARD_H: f32 = 816.0;
/// 卡片最小宽度（再窄标题就没地方了）。
pub const MIN_CARD_W: f32 = 460.0;
/// 卡片最小高度。
pub const MIN_CARD_H: f32 = 420.0;

pub const CARD_R: u8 = 20;
pub const PREVIEW_W: f32 = 431.0;
pub const PREVIEW_H: f32 = 240.0;
pub const PREVIEW_R: u8 = 18;

/// 由卡片尺寸推导窗口尺寸。
pub fn window_size(card: Vec2, expanded: bool) -> Vec2 {
    let extra_w = if expanded {
        PREVIEW_GAP + PREVIEW_W
    } else {
        0.0
    };
    vec2(
        card.x + 2.0 * MARGIN_X + extra_w,
        card.y + MARGIN_TOP + MARGIN_BOTTOM,
    )
}

/// 由窗口尺寸反推卡片尺寸。
pub fn card_size(win: Vec2, expanded: bool) -> Vec2 {
    let extra_w = if expanded {
        PREVIEW_GAP + PREVIEW_W
    } else {
        0.0
    };
    let card = vec2(
        win.x - 2.0 * MARGIN_X - extra_w,
        win.y - MARGIN_TOP - MARGIN_BOTTOM,
    );
    vec2(card.x.max(MIN_CARD_W), card.y.max(MIN_CARD_H))
}

// ---------------------------------------------------------------- 列表行

/// 列表第一行的顶部（工具栏两行：搜索栏 + 分类图标行）。
pub const LIST_TOP: f32 = 146.0;
/// 列表底部留白。
pub const LIST_BOTTOM_PAD: f32 = 13.0;
pub const ROW_INSET: f32 = 10.0;
pub const ROW_H: f32 = 81.0;
pub const ROW0_H: f32 = 90.0;
pub const TILE: f32 = 64.0;
/// 行内 hover 按钮的边长。
pub const HOVER_BTN: f32 = 34.0;

// ---------------------------------------------------------------- 颜色

pub const CARD_BG: Color32 = Color32::from_rgb(0xF6, 0xF6, 0xFD);
pub const PREVIEW_BG: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xFD);
pub const SELECT: Color32 = Color32::from_rgb(0xC6, 0xD9, 0xFE);
pub const INK: Color32 = Color32::from_rgb(0x0B, 0x0D, 0x1A);
pub const BODY: Color32 = Color32::from_rgb(0x1F, 0x23, 0x44);
pub const MUTED: Color32 = Color32::from_rgb(0x6A, 0x75, 0x97);
pub const PLACEHOLDER: Color32 = Color32::from_rgb(0x6B, 0x71, 0x90);
pub const NAVY: Color32 = Color32::from_rgb(0x3F, 0x45, 0x68);
pub const ACCENT: Color32 = Color32::from_rgb(0x0D, 0x6B, 0xFC);
pub const TILE_SELECTED: Color32 = Color32::from_rgb(0xD9, 0xE3, 0xFE);
pub const TILE_NORMAL: Color32 = Color32::from_rgb(0xED, 0xEE, 0xF9);
pub const TILE_PDF: Color32 = Color32::from_rgb(0xF6, 0xE3, 0xEA);
pub const TILE_HEADER: Color32 = Color32::from_rgb(0xD2, 0xE0, 0xFD);
pub const SEARCH_BG: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xF5);
pub const DIVIDER: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xF2);
pub const PILL: Color32 = Color32::from_rgb(0xE5, 0xE7, 0xF4);
pub const RED: Color32 = Color32::from_rgb(0xE2, 0x3B, 0x34);
/// 开关打开时的轨道色。
pub const SWITCH_ON: Color32 = Color32::from_rgb(0x2E, 0x7B, 0xFD);
/// 开关关闭时的轨道色。
pub const SWITCH_OFF: Color32 = Color32::from_rgb(0xD3, 0xD6, 0xE6);
