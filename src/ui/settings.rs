//! 设置页：快捷键说明、行为开关、退出入口。
//!
//! 页面画在主卡内部（与列表共用同一张卡片），交互命中矩形返回给 `app`。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, pos2, vec2,
};

use crate::fonts::bold_family;
use crate::icons;
use crate::theme::{BODY, DIVIDER, INK, MUTED, PILL, RED, SWITCH_OFF, SWITCH_ON, TILE_NORMAL};
use crate::ui::rr;

/// 设置页可点击区域。
pub struct Hits {
    pub back: Rect,
    /// 「失焦时自动隐藏」整行。
    pub auto_hide: Rect,
    /// 「窗口置顶」整行。
    pub pinned: Rect,
    pub exit: Rect,
}

fn row_band(card: Rect, y: f32) -> Rect {
    rr(card, 30.0, y - 24.0, card.width() - 60.0, 48.0)
}

fn toggle(p: &Painter, card: Rect, y: f32, on: bool) {
    let track = Rect::from_center_size(pos2(card.max.x - 56.0, y), vec2(52.0, 30.0));
    p.rect_filled(
        track,
        CornerRadius::same(15),
        if on { SWITCH_ON } else { SWITCH_OFF },
    );
    let knob_x = if on {
        track.right() - 15.0
    } else {
        track.left() + 15.0
    };
    p.circle_filled(pos2(knob_x, track.center().y), 11.0, Color32::WHITE);
}

pub fn draw(p: &Painter, card: Rect, auto_hide: bool, pinned: bool, pointer: Option<Pos2>) -> Hits {
    let highlighted = |r: Rect| pointer.is_some_and(|pos| r.contains(pos));

    p.text(
        card.min + vec2(30.0, 48.0),
        Align2::LEFT_CENTER,
        "设置",
        FontId::new(27.0, bold_family()),
        INK,
    );
    p.text(
        card.min + vec2(30.0, 82.0),
        Align2::LEFT_CENTER,
        "in-paste · UI 原型 v0.1",
        FontId::new(15.0, FontFamily::Proportional),
        MUTED,
    );

    let back = rr(card, card.width() - 108.0, 32.0, 84.0, 34.0);
    p.rect_filled(
        back,
        CornerRadius::same(9),
        if highlighted(back) { PILL } else { TILE_NORMAL },
    );
    icons::icon_bold(
        p,
        pos2(back.left() + 22.0, back.center().y),
        15.0,
        '\u{E72B}',
        BODY,
    );
    p.text(
        pos2(back.left() + 34.0, back.center().y),
        Align2::LEFT_CENTER,
        "返回",
        FontId::new(15.5, FontFamily::Proportional),
        BODY,
    );

    p.line_segment(
        [
            card.min + vec2(30.0, 104.0),
            card.min + vec2(card.width() - 30.0, 104.0),
        ],
        eframe::egui::Stroke::new(1.2, DIVIDER),
    );

    // 快捷键说明
    p.text(
        card.min + vec2(30.0, 138.0),
        Align2::LEFT_CENTER,
        "呼出面板",
        FontId::new(18.0, FontFamily::Proportional),
        BODY,
    );
    let pill = Rect::from_min_size(
        card.min + vec2(card.width() - 30.0 - 150.0, 138.0 - 20.0),
        vec2(150.0, 40.0),
    );
    p.rect_filled(pill, CornerRadius::same(10), crate::theme::SEARCH_BG);
    p.text(
        pill.center(),
        Align2::CENTER_CENTER,
        "Ctrl + Shift + V",
        FontId::new(16.0, FontFamily::Proportional),
        INK,
    );

    // 开关行
    let auto_row = row_band(card, 200.0);
    let pin_row = row_band(card, 260.0);
    for (row, label) in [(auto_row, "失焦时自动隐藏"), (pin_row, "窗口置顶（钉住）")]
    {
        if highlighted(row) {
            p.rect_filled(row, CornerRadius::same(10), TILE_NORMAL);
        }
        p.text(
            pos2(card.min.x + 36.0, row.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::new(18.0, FontFamily::Proportional),
            BODY,
        );
    }
    toggle(p, card, auto_row.center().y, auto_hide);
    toggle(p, card, pin_row.center().y, pinned);

    p.line_segment(
        [
            card.min + vec2(30.0, 306.0),
            card.min + vec2(card.width() - 30.0, 306.0),
        ],
        eframe::egui::Stroke::new(1.2, DIVIDER),
    );

    // 退出
    let exit = rr(card, 30.0, 330.0, card.width() - 60.0, 52.0);
    if highlighted(exit) {
        p.rect_filled(exit, CornerRadius::same(12), TILE_NORMAL);
    }
    p.text(
        pos2(exit.left() + 16.0, exit.center().y),
        Align2::LEFT_CENTER,
        "退出 in-paste",
        FontId::new(18.0, FontFamily::Proportional),
        RED,
    );

    p.text(
        card.min + vec2(30.0, card.height() - 40.0),
        Align2::LEFT_CENTER,
        "按 Esc 返回列表 · 点击卡片外会隐藏面板",
        FontId::new(14.0, FontFamily::Proportional),
        MUTED,
    );

    Hits {
        back,
        auto_hide: auto_row,
        pinned: pin_row,
        exit,
    }
}
