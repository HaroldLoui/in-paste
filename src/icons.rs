//! 图标绘制。
//!
//! 设计稿里的图标混用两种来源：图钉 / 复制 / 垃圾桶 / 关闭 / 放大镜 / 图片
//! 用 Segoe Fluent Icons 字形；网格、T、链条、文档、PDF 则用 painter 手绘，
//! 这样在任意 DPI 下都保持同一套线宽。

use eframe::egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
    pos2, vec2,
};

use crate::fonts;

/// 图标字体比设计稿的线细，叠四次制造加粗效果。
pub fn icon_bold(p: &Painter, center: Pos2, size: f32, ch: char, color: Color32) {
    for (dx, dy) in [(-0.55, 0.0), (0.55, 0.0), (0.0, -0.55), (0.0, 0.55)] {
        p.text(
            center + vec2(dx, dy + 1.0),
            Align2::CENTER_CENTER,
            ch,
            FontId::new(size, fonts::icons_family()),
            color,
        );
    }
}

/// 工具栏第一个「全部」分类：2 × 2 圆角方块。
pub fn grid(p: &Painter, center: Pos2, color: Color32) {
    let s = 13.0;
    let gap = 4.0;
    let off = (s + gap) * 0.5;
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        p.rect_filled(
            Rect::from_center_size(center + vec2(dx * off, dy * off), Vec2::splat(s)),
            CornerRadius::same(4),
            color,
        );
    }
}

/// 衬线体「T」。
pub fn letter_t(p: &Painter, c: Pos2, color: Color32) {
    let w = 28.0;
    let bar_h = 4.0;
    let top = c.y - 15.0;
    p.rect_filled(
        Rect::from_min_size(pos2(c.x - w * 0.5, top), vec2(w, bar_h)),
        CornerRadius::same(1),
        color,
    );
    p.rect_filled(
        Rect::from_min_size(pos2(c.x - 2.0, top), vec2(4.0, 28.0)),
        CornerRadius::same(1),
        color,
    );
    p.rect_filled(
        Rect::from_min_size(pos2(c.x - 7.0, top + 26.0), vec2(14.0, 4.0)),
        CornerRadius::same(1),
        color,
    );
    for x in [c.x - w * 0.5, c.x + w * 0.5 - 2.5] {
        p.rect_filled(
            Rect::from_min_size(pos2(x, top), vec2(2.5, 5.0)),
            CornerRadius::same(1),
            color,
        );
    }
}

/// 文档 + 三条横线。
pub fn doc(p: &Painter, c: Pos2, w: f32, h: f32, stroke: f32, color: Color32) {
    let rect = Rect::from_center_size(c, vec2(w, h));
    p.rect_stroke(
        rect,
        CornerRadius::same((w * 0.18) as u8),
        Stroke::new(stroke, color),
        StrokeKind::Inside,
    );
    let lx = rect.left() + w * 0.26;
    let full = w * 0.56;
    for (i, frac) in [0.32, 0.52, 0.72].iter().enumerate() {
        let lw = if i == 2 { full * 0.62 } else { full };
        p.rect_filled(
            Rect::from_min_size(pos2(lx, rect.top() + h * frac - 1.3), vec2(lw, 2.6)),
            CornerRadius::same(1),
            color,
        );
    }
}

fn capsule_points(center: Pos2, half_len: f32, half_w: f32, angle: f32) -> Vec<Pos2> {
    let (sin, cos) = angle.sin_cos();
    let axis = vec2(cos, sin);
    let normal = vec2(-sin, cos);
    let end = half_len - half_w;
    let mut pts = Vec::with_capacity(28);
    for i in 0..=12 {
        let a = -std::f32::consts::FRAC_PI_2 + i as f32 / 12.0 * std::f32::consts::PI;
        pts.push(center + axis * (end + half_w * a.cos()) + normal * (half_w * a.sin()));
    }
    for i in 0..=12 {
        let a = std::f32::consts::FRAC_PI_2 + i as f32 / 12.0 * std::f32::consts::PI;
        pts.push(center + axis * (-end + half_w * a.cos()) + normal * (half_w * a.sin()));
    }
    pts
}

/// 斜向链条：两只胶囊互相穿插，`bg` 用来遮住下层。
pub fn link(p: &Painter, c: Pos2, color: Color32, bg: Color32) {
    let stroke = Stroke::new(2.4, color);
    let angle = -45f32.to_radians();
    let axis = vec2(angle.cos(), angle.sin());
    let normal = vec2(-axis.y, axis.x);
    let lower = capsule_points(c - axis * 4.0 - normal * 4.0, 9.5, 3.4, angle);
    let upper = capsule_points(c + axis * 4.0 + normal * 4.0, 9.5, 3.4, angle);
    p.add(Shape::closed_line(lower, stroke));
    p.add(Shape::convex_polygon(upper.clone(), bg, Stroke::NONE));
    p.add(Shape::closed_line(upper, stroke));
}

fn folded_doc_points(r: Rect, fold: f32, rad: f32) -> Vec<Pos2> {
    vec![
        pos2(r.left() + rad, r.top()),
        pos2(r.right() - fold, r.top()),
        pos2(r.right(), r.top() + fold),
        pos2(r.right(), r.bottom() - rad),
        pos2(r.right() - rad, r.bottom()),
        pos2(r.left() + rad, r.bottom()),
        pos2(r.left(), r.bottom() - rad),
        pos2(r.left(), r.top() + rad),
    ]
}

/// 折角文档（文件分类）。
pub fn file(p: &Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, vec2(25.0, 30.0));
    let fold = 8.5;
    p.add(Shape::closed_line(
        folded_doc_points(r, fold, 3.5),
        Stroke::new(2.6, color),
    ));
    p.line_segment(
        [
            pos2(r.right() - fold, r.top()),
            pos2(r.right() - fold, r.top() + fold),
        ],
        Stroke::new(2.2, color),
    );
    p.line_segment(
        [
            pos2(r.right() - fold, r.top() + fold),
            pos2(r.right(), r.top() + fold),
        ],
        Stroke::new(2.2, color),
    );
}

/// 图片分类：圆角相框 + 太阳 + 山。
pub fn image(p: &Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, vec2(28.0, 28.0));
    p.rect_stroke(
        r,
        CornerRadius::same(7),
        Stroke::new(2.6, color),
        StrokeKind::Inside,
    );
    p.circle_stroke(
        pos2(r.right() - 8.0, r.top() + 8.0),
        2.1,
        Stroke::new(2.0, color),
    );
    let mountain = vec![
        pos2(r.left() + 4.5, r.bottom() - 6.5),
        pos2(r.left() + 10.5, r.top() + 12.5),
        pos2(r.left() + 15.5, r.bottom() - 8.5),
        pos2(r.left() + 18.5, r.bottom() - 11.5),
        pos2(r.right() - 4.5, r.bottom() - 6.5),
    ];
    p.add(Shape::line(mountain, Stroke::new(2.2, color)));
}

/// 复制按钮里的两张纸。
pub fn copy(p: &Painter, c: Pos2, scale: f32, color: Color32, bg: Color32) {
    let size = vec2(14.5, 17.5) * scale;
    let back = Rect::from_center_size(c + vec2(3.0, -2.6) * scale, size);
    let front = Rect::from_center_size(c + vec2(-2.0, 2.0) * scale, size);
    p.rect_stroke(
        back,
        CornerRadius::same(3),
        Stroke::new(2.0 * scale, color),
        StrokeKind::Inside,
    );
    p.rect_filled(front, CornerRadius::same(3), bg);
    p.rect_stroke(
        front,
        CornerRadius::same(3),
        Stroke::new(2.0 * scale, color),
        StrokeKind::Inside,
    );
}

/// PDF 文件图标：红色折角文档 + 「PDF」字样。
pub fn pdf(p: &Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, vec2(30.0, 34.0));
    let fold = 9.0;
    let rad = 3.0;
    let pts = vec![
        pos2(r.left() + rad, r.top()),
        pos2(r.right() - fold, r.top()),
        pos2(r.right(), r.top() + fold),
        pos2(r.right(), r.bottom() - rad),
        pos2(r.right() - rad, r.bottom()),
        pos2(r.left() + rad, r.bottom()),
        pos2(r.left(), r.bottom() - rad),
        pos2(r.left(), r.top() + rad),
    ];
    p.add(Shape::closed_line(pts, Stroke::new(2.4, color)));
    p.line_segment(
        [
            pos2(r.right() - fold, r.top()),
            pos2(r.right() - fold, r.top() + fold),
        ],
        Stroke::new(2.0, color),
    );
    p.line_segment(
        [
            pos2(r.right() - fold, r.top() + fold),
            pos2(r.right(), r.top() + fold),
        ],
        Stroke::new(2.0, color),
    );
    p.text(
        r.center() + vec2(0.0, 1.0),
        Align2::CENTER_CENTER,
        "PDF",
        FontId::new(8.0, fonts::bold_family()),
        color,
    );
}
