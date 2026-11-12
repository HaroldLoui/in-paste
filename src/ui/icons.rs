//! 图标：gpui-kit 自带 Lucide Icon + 简单几何自绘。

use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::{Tup, c};

/// 分类「全部」：2×2 圆角方块。
pub fn grid_icon(color: Tup, size: f32) -> impl IntoElement {
    let s = size * 0.38;
    let gap = size * 0.12;
    let off = (s + gap) * 0.5;
    let half = size * 0.5;
    div().relative().size(px(size)).children(
        [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)].map(|(dx, dy)| {
            div()
                .absolute()
                .size(px(s))
                .rounded(px(3.))
                .bg(c(color))
                .left(px(half + dx * off - s * 0.5))
                .top(px(half + dy * off - s * 0.5))
        }),
    )
}

/// 分类「文本」：衬线 T。
pub fn text_icon(color: Tup, size: f32) -> impl IntoElement {
    let w = size * 0.72;
    let bar = size * 0.12;
    div().relative().size(px(size)).children([
        div()
            .absolute()
            .w(px(w))
            .h(px(bar))
            .top(px(size * 0.2))
            .left(px(0.))
            .rounded(px(1.))
            .bg(c(color)),
        div()
            .absolute()
            .w(px(bar))
            .h(px(size * 0.48))
            .top(px(size * 0.2))
            .left(px(w * 0.5 - bar * 0.5))
            .rounded(px(1.))
            .bg(c(color)),
        div()
            .absolute()
            .w(px(w * 0.45))
            .h(px(bar))
            .top(px(size * 0.68))
            .left(px(w * 0.5 - w * 0.225))
            .rounded(px(1.))
            .bg(c(color)),
    ])
}

#[allow(dead_code)]
pub fn icon(name: IconName, size: f32) -> impl IntoElement {
    Icon::new(name).size(px(size))
}

pub fn icon_colored(name: IconName, size: f32, color: Tup) -> impl IntoElement {
    Icon::new(name).size(px(size)).text_color(c(color))
}
