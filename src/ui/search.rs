//! 搜索输入框。
//!
//! 胶囊背景和放大镜由 `toolbar` 绘制，这里只负责文本输入、焦点和占位文字。
//! `TextEdit` 用透明背景放在胶囊右侧，保证 IME、选区、光标都是 egui 原生行为。

use eframe::egui::{Align2, FontFamily, FontId, Frame, Margin, Response, TextEdit, Ui, pos2, vec2};

use crate::theme::{INK, PLACEHOLDER};

/// 固定 Id，便于 `Ctrl+F` 和 `Esc` 精确地请求 / 释放焦点。
pub fn id() -> eframe::egui::Id {
    eframe::egui::Id::unique("clipboard-search")
}

pub fn draw(ui: &mut Ui, rect: eframe::egui::Rect, query: &mut String) -> Response {
    let text_rect = eframe::egui::Rect::from_min_size(
        pos2(rect.left() + 53.0, rect.center().y - 15.0),
        vec2((rect.width() - 66.0).max(40.0), 30.0),
    );
    let response = ui.put(
        text_rect,
        TextEdit::singleline(query)
            .id(id())
            .font(FontId::new(17.0, FontFamily::Proportional))
            .text_color(INK)
            .frame(Frame::NONE)
            .margin(Margin::ZERO)
            .desired_width(f32::INFINITY),
    );

    if query.is_empty() && !response.has_focus() {
        ui.painter().text(
            pos2(rect.left() + 53.0, rect.center().y),
            Align2::LEFT_CENTER,
            "搜索剪贴板...",
            FontId::new(17.0, FontFamily::Proportional),
            PLACEHOLDER,
        );
    }

    response
}
