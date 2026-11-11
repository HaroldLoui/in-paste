//! 列表行：图标块、标题、相对时间、快捷键胶囊。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, TextureHandle, vec2,
};

use crate::icons;
use crate::model::{Item, Kind};
use crate::theme::{
    INK, LIST_TOP, MUTED, NAVY, PILL, PILL_SELECTED, RED, ROW_H, ROW_INSET, ROW_W, ROW0_H, SELECT,
    TILE, TILE_NORMAL, TILE_PDF, TILE_SELECTED,
};
use crate::ui::rr;

/// 第 `index` 行的顶部（卡片坐标系）。
fn top_of(index: usize) -> f32 {
    if index == 0 {
        LIST_TOP
    } else {
        LIST_TOP + ROW0_H + (index as f32 - 1.0) * ROW_H
    }
}

/// 第 `index` 行的矩形（卡片坐标系）。
pub fn rect(card: Rect, index: usize) -> Rect {
    let top = top_of(index);
    let height = if index == 0 { ROW0_H } else { ROW_H };
    rr(card, ROW_INSET, top, ROW_W, height)
}

pub fn draw(
    p: &Painter,
    card: Rect,
    index: usize,
    item: &Item,
    thumb: Option<&TextureHandle>,
    selected: bool,
) {
    let top = top_of(index);
    let height = if index == 0 { ROW0_H } else { ROW_H };

    if selected {
        p.rect_filled(
            rr(card, ROW_INSET, top, ROW_W, height),
            CornerRadius::same(14),
            SELECT,
        );
    }

    let tile_y = top + 15.0;
    let tile = rr(card, 30.0, tile_y, TILE, TILE);
    let tile_center = tile.center();

    match item.kind {
        Kind::Image => {
            if let Some(tex) = thumb {
                p.image(
                    tex.id(),
                    tile,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                p.rect_filled(tile, CornerRadius::same(15), TILE_NORMAL);
            }
        }
        kind => {
            let bg = match kind {
                Kind::Pdf => TILE_PDF,
                _ if selected => TILE_SELECTED,
                _ => TILE_NORMAL,
            };
            p.rect_filled(tile, CornerRadius::same(15), bg);

            match kind {
                Kind::Text | Kind::Rich => icons::doc(p, tile_center, 33.0, 36.0, 3.0, NAVY),
                Kind::Link => icons::link(p, tile_center, NAVY, bg),
                Kind::Pdf => icons::pdf(p, tile_center, RED),
                Kind::Image => {}
            }
        }
    }

    let title_top = top + if selected { 23.0 } else { 22.0 };
    // 标题单行裁剪，避免长文本压到右侧快捷键胶囊上。
    let title_clip = Rect::from_min_max(
        card.min + vec2(110.0, title_top),
        card.min + vec2(card.width() - 150.0, title_top + 34.0),
    );
    p.with_clip_rect(title_clip).text(
        card.min + vec2(110.0, title_top + 10.0),
        Align2::LEFT_CENTER,
        item.title,
        FontId::new(20.6, FontFamily::Proportional),
        INK,
    );

    let meta_top = top + if selected { 54.0 } else { 53.0 };
    p.text(
        card.min + vec2(110.0, meta_top + 8.0),
        Align2::LEFT_CENTER,
        item.meta,
        FontId::new(17.5, FontFamily::Proportional),
        MUTED,
    );

    // 快捷键胶囊
    let center_y = top + if selected { 47.0 } else { 45.0 };
    let pill = Rect::from_min_size(
        card.min + vec2(card.width() - 140.0, center_y - 21.0),
        vec2(110.0, 42.0),
    );
    p.rect_filled(
        pill,
        CornerRadius::same(11),
        if selected { PILL_SELECTED } else { PILL },
    );
    p.text(
        pill.center(),
        Align2::CENTER_CENTER,
        item.key,
        FontId::new(20.5, FontFamily::Proportional),
        INK,
    );
}
