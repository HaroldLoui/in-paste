//! 右侧预览卡：左缘箭头、头部图标与标题、正文。
//!
//! 页脚操作栏已移除（复制 / 置顶 / 删除 改为列表行的悬停按钮）。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke,
    TextureHandle, pos2, vec2,
};

use crate::icons;
use crate::model::{Item, Kind};
use crate::theme::{BODY, INK, NAVY, PREVIEW_BG, PREVIEW_R, PREVIEW_W, RED, TILE_HEADER};
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

    let title_clip = Rect::from_min_max(
        card.min + vec2(120.0, 26.0),
        card.min + vec2(PREVIEW_W - 16.0, 110.0),
    );
    let tp = p.with_clip_rect(title_clip);
    for (i, line) in item.preview_title().iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        tp.text(
            card.min + vec2(130.0, 48.0 + i as f32 * 28.0),
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

    let body_clip = Rect::from_min_max(
        card.min + vec2(20.0, 128.0),
        card.min + vec2(PREVIEW_W - 16.0, card.height() - 10.0),
    );
    let bp = p.with_clip_rect(body_clip);
    for (i, line) in item.preview_body().iter().enumerate() {
        bp.text(
            card.min + vec2(30.0, 156.0 + i as f32 * 28.0),
            Align2::LEFT_CENTER,
            *line,
            FontId::new(21.0, FontFamily::Proportional),
            BODY,
        );
    }
}
