//! 颜色、尺寸、圆角等设计 token。
//! 数值来自设计稿的逐像素测量，改这里就能整体校准观感。

use eframe::egui::Color32;

// ---------------------------------------------------------------- 窗口 / 卡片

/// 界面整体缩放系数（相对设计稿点数）。
///
/// 0.667 ≈ 缩小 1/3：窗口的原生逻辑尺寸、以及 egui 的绘制尺寸都按这个系数收，
/// 但内部布局仍用设计稿坐标（720×920 点），改这里即可整体放大/缩小。
pub const UI_SCALE: f32 = 2.0 / 3.0;

/// 仅显示主卡时的窗口宽度。
pub const BASE_W: f32 = 720.0;
/// 窗口高度。
pub const BASE_H: f32 = 920.0;
/// 展开预览后的窗口宽度。
pub const FULL_W: f32 = 1171.0;

pub const CARD_X: f32 = 24.0;
pub const CARD_Y: f32 = 40.0;
pub const CARD_W: f32 = 672.0;
/// 卡片高度按 8 行内容取整：列表起点 146 + 首行 90 + 7×81 + 底部留白 13。
pub const CARD_H: f32 = 816.0;
pub const CARD_R: u8 = 20;

pub const PREVIEW_X: f32 = 716.0;
pub const PREVIEW_W: f32 = 431.0;
pub const PREVIEW_H: f32 = 338.0;
pub const PREVIEW_R: u8 = 18;

// ---------------------------------------------------------------- 列表行

/// 列表第一行的顶部（工具栏两行：搜索栏 + 分类图标行）。
pub const LIST_TOP: f32 = 146.0;
pub const ROW_INSET: f32 = 10.0;
pub const ROW_W: f32 = 652.0;
pub const ROW_H: f32 = 81.0;
pub const ROW0_H: f32 = 90.0;
pub const TILE: f32 = 64.0;

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
pub const BUTTON: Color32 = Color32::from_rgb(0x11, 0x67, 0xFD);
pub const TILE_SELECTED: Color32 = Color32::from_rgb(0xD9, 0xE3, 0xFE);
pub const TILE_NORMAL: Color32 = Color32::from_rgb(0xED, 0xEE, 0xF9);
pub const TILE_PDF: Color32 = Color32::from_rgb(0xF6, 0xE3, 0xEA);
pub const TILE_HEADER: Color32 = Color32::from_rgb(0xD2, 0xE0, 0xFD);
pub const SEARCH_BG: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xF5);
pub const DIVIDER: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xF2);
pub const PILL: Color32 = Color32::from_rgb(0xE5, 0xE7, 0xF4);
pub const PILL_SELECTED: Color32 = Color32::from_rgb(0xB7, 0xCB, 0xF5);
pub const RED: Color32 = Color32::from_rgb(0xE2, 0x3B, 0x34);
