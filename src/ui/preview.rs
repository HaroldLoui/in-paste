//! 右侧预览卡：箭头、头部图标与标题、正文、底部操作栏。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
    TextureHandle, pos2, vec2,
};

use crate::icons;
use crate::model::{Item, Kind};
use crate::theme::{
    BODY, BUTTON, DIVIDER, INK, NAVY, PREVIEW_BG, PREVIEW_R, PREVIEW_W, RED, TILE_HEADER,
};
use crate::ui::{draw_card, rr};

pub fn draw(p: &Painter, card: Rect, item: &Item, arrow_y: f32, thumb: Option<&TextureHandle>) {
    // 指向选中行的箭头
    let tip_y = arrow_y.clamp(card.top() + 12.0, card.bottom() - 12.0);
    p.add(Shape::convex_polygon(
        vec![
            pos2(card.left() - 18.0, tip_y),
            pos2(card.left() + 1.0, tip_y - 11.0),
            pos2(card.left() + 1.0, tip_y + 11.0),
        ],
        PREVIEW_BG,
        Stroke::NONE,
    ));

    draw_card(p, card, PREVIEW_R, PREVIEW_BG);

    let header_tile = rr(card, 28.0, 26.0, 76.0, 80.0);
    p.rect_filled(header_tile, CornerRadius::same(18), TILE_HEADER);
    match item.kind {
        Kind::Image => {
            if let Some(tex) = thumb {
                p.image(
                    tex.id(),
                    header_tile,
                    Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                icons::doc(
                    p,
                    header_tile.center(),
                    35.0,
                    40.0,
                    3.0,
                    Color32::from_rgb(0x3D, 0x40, 0x67),
                );
            }
        }
        Kind::Link => icons::link(p, header_tile.center(), NAVY, TILE_HEADER),
        Kind::Pdf => icons::pdf(p, header_tile.center(), RED),
        _ => icons::doc(
            p,
            header_tile.center(),
            35.0,
            40.0,
            3.0,
            Color32::from_rgb(0x3D, 0x40, 0x67),
        ),
    }

    let title_x = 130.0;
    for (i, line) in item.preview_title().iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        p.text(
            card.min + vec2(title_x, 48.0 + i as f32 * 28.0),
            Align2::LEFT_CENTER,
            *line,
            FontId::new(21.5, FontFamily::Proportional),
            INK,
        );
    }

    p.line_segment(
        [
            card.min + vec2(14.0, 121.0),
            card.min + vec2(PREVIEW_W - 14.0, 121.0),
        ],
        Stroke::new(1.2, Color32::from_rgb(0xE8, 0xE9, 0xF4)),
    );

    for (i, line) in item.preview_body().iter().enumerate() {
        p.text(
            card.min + vec2(30.0, 157.0 + i as f32 * 28.0),
            Align2::LEFT_CENTER,
            *line,
            FontId::new(21.0, FontFamily::Proportional),
            BODY,
        );
    }

    p.line_segment(
        [
            card.min + vec2(0.0, 246.0),
            card.min + vec2(PREVIEW_W, 246.0),
        ],
        Stroke::new(1.2, DIVIDER),
    );

    // 底部操作栏
    let button = rr(card, 29.0, 268.0, 111.0, 50.0);
    p.rect_filled(button, CornerRadius::same(12), BUTTON);
    icons::copy(
        p,
        pos2(button.left() + 28.5, button.center().y),
        1.0,
        Color32::WHITE,
        BUTTON,
    );
    p.text(
        pos2(button.left() + 54.0, button.center().y),
        Align2::LEFT_CENTER,
        "复制",
        FontId::new(18.0, FontFamily::Proportional),
        Color32::WHITE,
    );

    for x in [162.0, 288.0] {
        p.line_segment(
            [card.min + vec2(x, 272.0), card.min + vec2(x, 314.0)],
            Stroke::new(1.2, DIVIDER),
        );
    }

    icons::icon_bold(
        p,
        card.min + vec2(200.0, 293.0),
        26.0,
        '\u{E840}',
        Color32::from_rgb(0x2C, 0x32, 0x52),
    );
    p.text(
        card.min + vec2(225.0, 293.0),
        Align2::LEFT_CENTER,
        "置顶",
        FontId::new(18.5, FontFamily::Proportional),
        Color32::from_rgb(0x1C, 0x21, 0x38),
    );

    icons::icon_bold(
        p,
        card.min + vec2(327.0, 293.0),
        26.0,
        '\u{E74D}',
        Color32::from_rgb(0x2C, 0x32, 0x52),
    );
    p.text(
        card.min + vec2(352.0, 293.0),
        Align2::LEFT_CENTER,
        "删除",
        FontId::new(18.5, FontFamily::Proportional),
        Color32::from_rgb(0x1C, 0x21, 0x38),
    );
}
