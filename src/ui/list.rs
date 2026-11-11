//! 列表绘制、悬停按钮与点击命中。
//!
//! 行内容超出可视区时按 `scroll` 偏移绘制，滚轮由 `app` 换算后传入。

use eframe::egui::{Align2, FontFamily, FontId, Painter, Pos2, Rect, TextureHandle, vec2};

use crate::model::Item;
use crate::theme::{LIST_BOTTOM_PAD, LIST_TOP, MUTED};
use crate::ui::row;

/// 行内 hover 按钮的动作。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Copy,
    Pin,
    Delete,
}

pub struct Input<'a> {
    pub items: &'a [Item],
    /// 当前分类过滤 + 置顶排序后的真实下标。
    pub visible: &'a [usize],
    pub selected: Option<usize>,
    pub scroll: f32,
    pub pointer: Option<Pos2>,
    pub pressed: bool,
    pub thumb: Option<&'a TextureHandle>,
}

#[derive(Default)]
pub struct Output {
    /// 本帧被点击的行（悬停按钮之外的区域）。
    pub pressed_row: Option<usize>,
    /// 本帧被点击的悬停按钮。
    pub action: Option<(usize, Action)>,
    pub hovered: Option<usize>,
}

/// 列表可视区域（卡片坐标系）。
pub fn list_rect(card: Rect) -> Rect {
    Rect::from_min_max(
        card.min + vec2(0.0, LIST_TOP),
        card.max - vec2(0.0, LIST_BOTTOM_PAD),
    )
}

/// 列表内容高度（不含顶部工具栏）。
pub fn content_height(n: usize) -> f32 {
    if n == 0 {
        0.0
    } else {
        row::height(0) + (n as f32 - 1.0) * row::height(1)
    }
}

/// 竖向可滚动距离。
pub fn max_scroll(card: Rect, n: usize) -> f32 {
    (content_height(n) - list_rect(card).height()).max(0.0)
}

pub fn draw(p: &Painter, card: Rect, input: &Input) -> Output {
    let list = list_rect(card);
    let n = input.visible.len();

    let scard = card.translate(vec2(0.0, -input.scroll));
    let row_at = |pos: Pos2| (0..n).find(|&i| row::rect(scard, i).contains(pos));
    let hovered = input
        .pointer
        .filter(|pos| list.contains(*pos))
        .and_then(row_at);
    let hover_action = hovered.and_then(|i| {
        input.pointer.and_then(|pos| {
            row::actions(scard, i)
                .iter()
                .position(|r| r.contains(pos))
                .map(|k| (i, k))
        })
    });

    let paint = p.with_clip_rect(list);
    if n == 0 {
        paint.text(
            list.center(),
            Align2::CENTER_CENTER,
            "没有匹配的剪贴板记录",
            FontId::new(17.0, FontFamily::Proportional),
            MUTED,
        );
    }

    for (pos_idx, &real) in input.visible.iter().enumerate() {
        let r = row::rect(scard, pos_idx);
        if r.bottom() < list.top() || r.top() > list.bottom() {
            continue;
        }
        let hover_btn = hover_action.filter(|(i, _)| *i == pos_idx).map(|(_, k)| k);
        row::draw(
            &paint,
            scard,
            pos_idx,
            &input.items[real],
            input.thumb,
            input.selected == Some(pos_idx),
            hovered == Some(pos_idx),
            hover_btn,
        );
    }

    let mut out = Output {
        hovered,
        ..Default::default()
    };
    if input.pressed {
        if let Some((i, k)) = hover_action {
            let action = match k {
                0 => Action::Copy,
                1 => Action::Pin,
                _ => Action::Delete,
            };
            out.action = Some((i, action));
        } else {
            out.pressed_row = hovered;
        }
    }
    out
}
