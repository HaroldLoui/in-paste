//! 主卡顶部工具栏：分类图标、搜索胶囊、窗口按钮。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Rect, Stroke, pos2, vec2,
};

use crate::icons;
use crate::theme::{ACCENT, CARD_BG, NAVY, PLACEHOLDER, SEARCH_BG};
use crate::ui::rr;

pub fn draw(p: &Painter, card: Rect) {
    let cy = 43.0;

    icons::grid(p, card.min + vec2(44.0, cy), ACCENT);
    icons::letter_t(p, card.min + vec2(108.0, cy), NAVY);

    icons::image(p, card.min + vec2(177.0, cy), NAVY);
    icons::file(p, card.min + vec2(243.0, cy), NAVY);
    icons::link(p, card.min + vec2(311.0, cy), NAVY, CARD_BG);

    p.line_segment(
        [card.min + vec2(338.0, 24.0), card.min + vec2(338.0, 62.0)],
        Stroke::new(1.5, Color32::from_rgb(0xE8, 0xE8, 0xF4)),
    );

    // 搜索胶囊
    let search = rr(card, 359.0, 18.0, 265.0, 58.0);
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

    icons::icon_bold(p, card.min + vec2(681.0, cy), 26.0, '\u{E840}', NAVY);
    icons::icon(p, card.min + vec2(737.0, cy), 21.0, '\u{E711}', NAVY);

    // 选中分类下划线
    p.rect_filled(
        rr(card, 29.0, 76.0, 30.0, 3.0),
        CornerRadius::same(2),
        ACCENT,
    );
}
