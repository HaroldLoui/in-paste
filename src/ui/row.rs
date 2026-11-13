//! 列表行：图标块、标题、meta、选中或悬停时的三个操作按钮。

use gpui_kit::assets::IconName;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::core::{Item, Kind};
use crate::theme::*;
use crate::ui::{icons, no_drag_div};

pub fn height(index: usize) -> f32 {
    if index == 0 { ROW0_H } else { ROW_H }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RowAction {
    Copy,
    Pin,
    Delete,
}

/// `view` 用于回写选中/悬停/操作；`pos` 是可见列表位置，`real` 是 items 真实下标。
pub fn render(
    view: &Entity<crate::ui::app::UiApp>,
    pos: usize,
    real: usize,
    item: &Item,
    selected: bool,
    hovered: bool,
) -> impl IntoElement {
    let kind = item.kind;
    let tile_bg = match kind {
        Kind::Pdf => TILE_PDF,
        _ if selected => TILE_SELECTED,
        _ => TILE_NORMAL,
    };

    let v_select = view.clone();
    let v_hover = view.clone();
    let v_copy = view.clone();
    let v_pin = view.clone();
    let v_del = view.clone();
    let pinned = item.pinned;
    let show_actions = selected || hovered;

    no_drag_div(SharedString::from(format!("row-{pos}")))
        .w_full()
        .h(px(height(pos) - 1.))
        .flex()
        .items_center()
        .gap(px(14.))
        .px(px(12.))
        .rounded(px(14.))
        .when(selected, |d| d.bg(c(SELECT)))
        .hover(|s| s.bg(c(SELECT)))
        .cursor_pointer()
        .on_hover(move |is_hovered, _, cx| {
            v_hover.update(cx, |this, cx| {
                if *is_hovered {
                    this.hovered = Some(pos);
                } else if this.hovered == Some(pos) {
                    this.hovered = None;
                }
                cx.notify();
            });
        })
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            v_select.update(cx, |this, cx| {
                this.on_row_select(pos, cx);
            });
        })
        .child(
            div()
                .id(SharedString::from(format!("tile-{pos}")))
                .size(px(TILE))
                .rounded(px(15.))
                .bg(c(tile_bg))
                .flex()
                .items_center()
                .justify_center()
                .child(tile_content(kind)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .w_full()
                        .truncate()
                        .text_size(px(20.))
                        .text_color(c(INK))
                        .child(item.title),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .text_size(px(17.))
                        .text_color(c(MUTED))
                        .child(item.meta),
                ),
        )
        .when(pinned, |d| {
            d.child(icons::icon_colored(IconName::Pin, 16., ACCENT))
        })
        .when(show_actions, |d| {
            d.child(
                div()
                    .flex()
                    .gap(px(6.))
                    .child(action_btn("copy", IconName::Copy, NAVY, move |cx| {
                        v_copy.update(cx, |this, cx| {
                            this.on_row_action(real, RowAction::Copy, cx);
                        });
                    }))
                    .child(action_btn(
                        "pin",
                        IconName::Pin,
                        if pinned { ACCENT } else { NAVY },
                        move |cx| {
                            v_pin.update(cx, |this, cx| {
                                this.on_row_action(real, RowAction::Pin, cx);
                            });
                        },
                    ))
                    .child(action_btn("del", IconName::Trash, RED, move |cx| {
                        v_del.update(cx, |this, cx| {
                            this.on_row_action(real, RowAction::Delete, cx);
                        });
                    })),
            )
        })
}

fn tile_content(kind: Kind) -> impl IntoElement {
    match kind {
        Kind::Link => icons::icon_colored(IconName::Link, 28., NAVY).into_any_element(),
        Kind::Pdf => icons::icon_colored(IconName::File, 28., RED).into_any_element(),
        Kind::Image => icons::icon_colored(IconName::Image, 28., NAVY).into_any_element(),
        Kind::Text | Kind::Rich => {
            icons::icon_colored(IconName::FileText, 28., NAVY).into_any_element()
        }
    }
}

fn action_btn(
    id: &str,
    name: IconName,
    color: Tup,
    on_click: impl Fn(&mut App) + 'static,
) -> impl IntoElement {
    no_drag_div(SharedString::from(format!("rowbtn-{id}")))
        .size(px(HOVER_BTN))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.))
        .bg(ca((255, 255, 255), 160))
        .cursor_pointer()
        .hover(|s| s.bg(rgb8(255, 255, 255)))
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            on_click(cx);
        })
        .child(icons::icon_colored(name, 16., color))
}
