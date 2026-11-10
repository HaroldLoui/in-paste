//! 列表绘制与点击命中。

use eframe::egui::{Rect, Sense, TextureHandle, Ui};

use crate::model::ITEMS;
use crate::ui::row;

/// 画出所有行，返回本帧被点击的行号。
pub fn draw(
    ui: &mut Ui,
    card: Rect,
    thumb: Option<&TextureHandle>,
    selected: Option<usize>,
) -> Option<usize> {
    let p = ui.painter().clone();
    let mut clicked = None;

    for (i, item) in ITEMS.iter().enumerate() {
        row::draw(&p, card, i, item, thumb, selected == Some(i));

        let resp = ui.interact(
            row::rect(card, i),
            ui.scope_id().with(("clip-row", i)),
            Sense::click(),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            clicked = Some(i);
        }
    }

    clicked
}
