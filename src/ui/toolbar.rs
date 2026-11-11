//! 主卡顶部工具栏：第一行是搜索胶囊与窗口按钮，第二行是分类图标。
//!
//! 竖长版布局把分类图标从搜索栏左侧移到了搜索栏下方，
//! 搜索胶囊宽度按卡片宽度自适应。

use eframe::egui::{Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Rect, pos2, vec2};

use crate::icons;
use crate::theme::{ACCENT, NAVY, PLACEHOLDER, SEARCH_BG};
use crate::ui::rr;

/// 搜索栏所在行的垂直中心。
pub const SEARCH_CY: f32 = 47.0;
/// 分类图标行的垂直中心。
pub const TABS_CY: f32 = 112.0;

/// 右侧「置顶」按钮中心距卡片右缘的距离。
const PIN_INSET: f32 = 99.0;
/// 搜索胶囊右缘到「置顶」图标中心的间距。
const SEARCH_GAP: f32 = 57.0;
/// 卡片左内边距。
const PAD_X: f32 = 30.0;

pub fn draw(p: &Painter, card: Rect) {
    let w = card.width();
    let pin_x = w - PIN_INSET;

    // 第一行：搜索胶囊 + 置顶 / 关闭
    let search = rr(card, PAD_X, 18.0, pin_x - SEARCH_GAP - PAD_X, 58.0);
    p.rect_filled(search, CornerRadius::same(29), SEARCH_BG);
    icons::icon_bold(
        p,
        pos2(search.left() + 24.0, search.center().y),
        21.0,
        '\u{E721}',
        Color32::from_rgb(0x5E, 0x64, 0x80),
    );
    p.text(
        pos2(search.left() + 53.0, search.center().y),
        Align2::LEFT_CENTER,
        "搜索剪贴板...",
        FontId::new(17.0, FontFamily::Proportional),
        PLACEHOLDER,
    );

    icons::icon_bold(p, card.min + vec2(pin_x, SEARCH_CY), 26.0, '\u{E840}', NAVY);
    icons::icon(
        p,
        card.min + vec2(w - 43.0, SEARCH_CY),
        21.0,
        '\u{E711}',
        NAVY,
    );

    // 第二行：分类图标（左对齐），选中的第一个图标下方带指示线
    icons::grid(p, card.min + vec2(44.0, TABS_CY), ACCENT);
    icons::letter_t(p, card.min + vec2(104.0, TABS_CY), NAVY);
    icons::image(p, card.min + vec2(164.0, TABS_CY), NAVY);
    icons::file(p, card.min + vec2(224.0, TABS_CY), NAVY);
    icons::link(
        p,
        card.min + vec2(284.0, TABS_CY),
        NAVY,
        crate::theme::CARD_BG,
    );

    p.rect_filled(
        rr(card, 29.0, 132.0, 30.0, 3.0),
        CornerRadius::same(2),
        ACCENT,
    );
}
