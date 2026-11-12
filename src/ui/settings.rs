//! 设置页：快捷键说明、行为开关、退出入口。

use gpui_kit::assets::IconName;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::*;
use crate::ui::icons;

#[allow(dead_code)]
fn render(view: &Entity<crate::ui::app::UiApp>) -> impl IntoElement {
    let v_back = view.clone();
    let v_auto = view.clone();
    let v_pin = view.clone();
    let v_exit = view.clone();

    // 状态从 view 读一次（render 时 view 已在手上）
    // 注意：render 里不能 read 自己（会 double-borrow），由调用方传入。
    div()
        .id("settings")
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .px(px(30.))
        .pt(px(32.))
        .pb(px(24.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_size(px(24.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(c(INK))
                                .child("设置"),
                        )
                        .child(
                            div()
                                .text_size(px(16.))
                                .text_color(c(MUTED))
                                .child("in-paste · UI 原型 v0.2"),
                        ),
                )
                .child(
                    div()
                        .id("btn-back")
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .h(px(34.))
                        .px(px(14.))
                        .rounded(px(9.))
                        .bg(c(TILE_NORMAL))
                        .cursor_pointer()
                        .hover(|s| s.bg(c(PILL)))
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            v_back.update(cx, |this, cx| {
                                this.back_to_list();
                                cx.notify();
                            });
                        })
                        .child(icons::icon_colored(IconName::ChevronLeft, 16., BODY))
                        .child(div().text_size(px(16.)).text_color(c(BODY)).child("返回")),
                ),
        )
        .child(div().mt(px(16.)).w_full().h(px(1.)).bg(c(DIVIDER)))
        .child(
            div()
                .mt(px(24.))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(20.))
                        .text_color(c(BODY))
                        .child("呼出面板"),
                )
                .child(
                    div()
                        .h(px(40.))
                        .px(px(16.))
                        .flex()
                        .items_center()
                        .rounded(px(10.))
                        .bg(c(SEARCH_BG))
                        .text_size(px(17.))
                        .text_color(c(INK))
                        .child("Ctrl + Shift + V"),
                ),
        )
        // 开关行由调用方注入状态（避免 render 内 read 自己）
        .child(
            div()
                .mt(px(24.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .id("switches"),
        )
        .child(div().mt(px(20.)).w_full().h(px(1.)).bg(c(DIVIDER)))
        .child(
            div()
                .id("btn-exit")
                .mt(px(16.))
                .w_full()
                .h(px(52.))
                .flex()
                .items_center()
                .px(px(16.))
                .rounded(px(12.))
                .cursor_pointer()
                .hover(|s| s.bg(c(TILE_NORMAL)))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    v_exit.update(cx, |this, cx| this.exit(cx));
                })
                .child(
                    div()
                        .text_size(px(20.))
                        .text_color(c(RED))
                        .child("退出 in-paste"),
                ),
        )
        .child(
            div()
                .mt_auto()
                .text_size(px(15.))
                .text_color(c(MUTED))
                .child("按 Esc 返回列表 · 点击卡片外会隐藏面板"),
        )
        // 把两个开关行插到 switches 容器：重构成带参渲染
        .map(|root| {
            // 上面 id="switches" 的空容器在 render_with_state 里填充；
            // 这里保持结构完整，实际状态由 render_with_state 提供。
            let _ = (&v_auto, &v_pin);
            root
        })
}

/// 带状态的设置页（由 app 调用）。
pub fn render_with_state(
    view: &Entity<crate::ui::app::UiApp>,
    auto_hide: bool,
    pinned: bool,
) -> impl IntoElement {
    let v_auto = view.clone();
    let v_pin = view.clone();

    div()
        .id("settings")
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .px(px(30.))
        .pt(px(32.))
        .pb(px(24.))
        .child(header(view))
        .child(div().mt(px(16.)).w_full().h(px(1.)).bg(c(DIVIDER)))
        .child(
            div()
                .mt(px(24.))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(20.))
                        .text_color(c(BODY))
                        .child("呼出面板"),
                )
                .child(
                    div()
                        .h(px(40.))
                        .px(px(16.))
                        .flex()
                        .items_center()
                        .rounded(px(10.))
                        .bg(c(SEARCH_BG))
                        .text_size(px(17.))
                        .text_color(c(INK))
                        .child("Ctrl + Shift + V"),
                ),
        )
        .child(
            div()
                .mt(px(24.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(switch_row(
                    "auto-hide",
                    "失焦时自动隐藏",
                    auto_hide,
                    move |cx| {
                        v_auto.update(cx, |this, cx| {
                            this.toggle_auto_hide();
                            cx.notify();
                        });
                    },
                ))
                .child(switch_row(
                    "pin",
                    "窗口置顶（钉住）",
                    pinned,
                    move |cx| {
                        v_pin.update(cx, |this, cx| {
                            this.toggle_pin(cx);
                            cx.notify();
                        });
                    },
                )),
        )
        .child(div().mt(px(20.)).w_full().h(px(1.)).bg(c(DIVIDER)))
        .child(exit_row(view))
        .child(
            div()
                .mt_auto()
                .text_size(px(15.))
                .text_color(c(MUTED))
                .child("按 Esc 返回列表 · 点击卡片外会隐藏面板"),
        )
}

fn header(view: &Entity<crate::ui::app::UiApp>) -> impl IntoElement {
    let v_back = view.clone();
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(24.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(c(INK))
                        .child("设置"),
                )
                .child(
                    div()
                        .text_size(px(16.))
                        .text_color(c(MUTED))
                        .child("in-paste · UI 原型 v0.2"),
                ),
        )
        .child(
            div()
                .id("btn-back")
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(34.))
                .px(px(14.))
                .rounded(px(9.))
                .bg(c(TILE_NORMAL))
                .cursor_pointer()
                .hover(|s| s.bg(c(PILL)))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    v_back.update(cx, |this, cx| {
                        this.back_to_list();
                        cx.notify();
                    });
                })
                .child(icons::icon_colored(IconName::ChevronLeft, 16., BODY))
                .child(div().text_size(px(16.)).text_color(c(BODY)).child("返回")),
        )
}

fn exit_row(view: &Entity<crate::ui::app::UiApp>) -> impl IntoElement {
    let v_exit = view.clone();
    div()
        .id("btn-exit")
        .mt(px(16.))
        .w_full()
        .h(px(52.))
        .flex()
        .items_center()
        .px(px(16.))
        .rounded(px(12.))
        .cursor_pointer()
        .hover(|s| s.bg(c(TILE_NORMAL)))
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            v_exit.update(cx, |this, cx| this.exit(cx));
        })
        .child(
            div()
                .text_size(px(20.))
                .text_color(c(RED))
                .child("退出 in-paste"),
        )
}

fn switch_row(
    id: &str,
    label: &str,
    on: bool,
    on_toggle: impl Fn(&mut App) + 'static,
) -> impl IntoElement {
    let label = label.to_owned();
    div()
        .id(SharedString::from(format!("switch-{id}")))
        .w_full()
        .h(px(52.))
        .flex()
        .items_center()
        .justify_between()
        .px(px(6.))
        .rounded(px(10.))
        .cursor_pointer()
        .hover(|s| s.bg(c(TILE_NORMAL)))
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            on_toggle(cx);
        })
        .child(div().text_size(px(20.)).text_color(c(BODY)).child(label))
        .child(switch_track(on))
}

fn switch_track(on: bool) -> impl IntoElement {
    let track = div()
        .w(px(52.))
        .h(px(30.))
        .rounded(px(15.))
        .bg(if on { c(SWITCH_ON) } else { c(SWITCH_OFF) })
        .flex()
        .items_center()
        .px(px(4.));
    let track = if on {
        track.justify_end()
    } else {
        track.justify_start()
    };
    track.child(div().size(px(22.)).rounded_full().bg(rgb8(255, 255, 255)))
}
