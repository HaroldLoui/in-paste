//! 列表行：图标块、标题、相对时间、快捷键胶囊。
//!
//! 鼠标悬停时胶囊替换成三个小图标按钮（复制 / 置顶 / 删除），
//! 选中态只负责高亮行背景。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Rect, TextureHandle, vec2,
};

use crate::icons;
use crate::model::{Item, Kind};
use crate::theme::{
    ACCENT, CARD_BG, HOVER_BTN, INK, LIST_TOP, MUTED, NAVY, PILL, RED, ROW_H, ROW_INSET, ROW0_H,
    SELECT, TILE, TILE_NORMAL, TILE_PDF, TILE_SELECTED,
};
use crate::ui::rr;

/// 第 `index` 行的顶部（卡片坐标系）。
pub fn top_of(index: usize) -> f32 {
    if index == 0 {
        LIST_TOP
    } else {
        LIST_TOP + ROW0_H + (index as f32 - 1.0) * ROW_H
    }
}

pub fn height(index: usize) -> f32 {
    if index == 0 { ROW0_H } else { ROW_H }
}

/// 第 `index` 行的矩形（卡片坐标系）。
pub fn rect(card: Rect, index: usize) -> Rect {
    let width = (card.width() - 2.0 * ROW_INSET).max(120.0);
    rr(card, ROW_INSET, top_of(index), width, height(index))
}

/// 行内三个 hover 按钮（复制 / 置顶 / 删除）。
pub fn actions(card: Rect, index: usize) -> [Rect; 3] {
    let row = rect(card, index);
    let right = card.left() + card.width() - 30.0;
    let y = row.center().y;
    [90.0, 55.0, 20.0].map(|inset| {
        Rect::from_center_size(
            eframe::egui::pos2(right - inset, y),
            vec2(HOVER_BTN, HOVER_BTN),
        )
    })
}

/// 绘制一行。
///
/// 参数偏多是因为这是纯绘制函数：行状态由调用方（app）持有，
/// 这里不引入额外结构体，保持 list/row 两层都可以直接照着状态画。
#[allow(clippy::too_many_arguments)]
pub fn draw(
    p: &Painter,
    card: Rect,
    index: usize,
    item: &Item,
    thumb: Option<&TextureHandle>,
    selected: bool,
    hovered: bool,
    hover_btn: Option<usize>,
) {
    let top = top_of(index);
    let row = rect(card, index);

    if selected {
        p.rect_filled(row, CornerRadius::same(14), SELECT);
    }

    let tile = rr(card, 30.0, top + 15.0, TILE, TILE);
    let tile_center = tile.center();

    match item.kind {
        Kind::Image => {
            if let Some(tex) = thumb {
                p.image(
                    tex.id(),
                    tile,
                    Rect::from_min_max(eframe::egui::Pos2::ZERO, eframe::egui::pos2(1.0, 1.0)),
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

    // 标题单行裁剪，避免长文本压到右侧胶囊/按钮上。
    let title_clip = Rect::from_min_max(
        card.min + vec2(110.0, top + 14.0),
        card.min + vec2(card.width() - 150.0, top + 52.0),
    );
    p.with_clip_rect(title_clip).text(
        card.min + vec2(110.0, top + 32.0),
        Align2::LEFT_CENTER,
        item.title,
        FontId::new(20.6, FontFamily::Proportional),
        INK,
    );

    p.text(
        card.min + vec2(110.0, top + 61.0),
        Align2::LEFT_CENTER,
        item.meta,
        FontId::new(17.5, FontFamily::Proportional),
        MUTED,
    );

    if item.pinned {
        icons::icon_bold(
            p,
            card.min + vec2(card.width() - 158.0, top + 61.0),
            14.0,
            '\u{E840}',
            ACCENT,
        );
    }

    // 选中或悬停时显示三个小按钮（不显示 Ctrl+N 快捷键）。
    if selected || hovered {
        let row_bg = if selected { SELECT } else { CARD_BG };
        for (k, btn) in actions(card, index).iter().enumerate() {
            if hover_btn == Some(k) {
                p.rect_filled(*btn, CornerRadius::same(9), PILL);
            }
        }
        let [copy_btn, pin_btn, del_btn] = actions(card, index);
        icons::copy(p, copy_btn.center(), 0.8, NAVY, row_bg);
        icons::icon_bold(
            p,
            pin_btn.center(),
            15.0,
            '\u{E840}',
            if item.pinned { ACCENT } else { NAVY },
        );
        icons::icon_bold(p, del_btn.center(), 15.0, '\u{E74D}', RED);
    }
}
