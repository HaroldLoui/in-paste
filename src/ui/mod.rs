//! 界面绘制层。
//!
//! 所有尺寸都以卡片左上角为原点，由 `rr` 换算成窗口坐标。

pub mod list;
pub mod preview;
pub mod row;
pub mod search;
pub mod settings;
pub mod toolbar;

use eframe::egui::{
    Color32, CornerRadius, Painter, Rect, Stroke, StrokeKind, epaint::Shadow, vec2,
};

/// 圆角卡片：两层外阴影 + 填充 + 1px 内高光。
pub fn draw_card(p: &Painter, rect: Rect, radius: u8, fill: Color32) {
    p.add(
        Shadow {
            offset: [0, 10],
            blur: 34,
            spread: 0,
            color: Color32::from_black_alpha(26),
        }
        .as_shape(rect, CornerRadius::same(radius)),
    );
    p.add(
        Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: Color32::from_black_alpha(16),
        }
        .as_shape(rect, CornerRadius::same(radius)),
    );
    p.rect_filled(rect, CornerRadius::same(radius), fill);
    p.rect_stroke(
        rect.shrink(0.5),
        CornerRadius::same(radius),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 110)),
        StrokeKind::Inside,
    );
}

/// 卡片内相对坐标 → 绝对矩形。
pub fn rr(card: Rect, x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(card.min + vec2(x, y), vec2(w, h))
}
