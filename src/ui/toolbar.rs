//! 主卡顶部工具栏：第一行搜索胶囊，第二行分类图标 + 置顶 / 设置。

use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::*;
use crate::ui::{icons, no_drag_div};

// pub const TABS: [&str; 5] = ["全部", "文本", "图片", "PDF", "链接"];
pub const TABS: [IconName; 5] = [
    IconName::LayoutGrid,
    IconName::Type,
    IconName::Image,
    IconName::File,
    IconName::Link,
];

/// `view` 用于在 on_click 里回写状态；`search` 是输入框实体。
pub fn render(
    view: &Entity<crate::ui::app::UiApp>,
    tab: usize,
    pinned: bool,
    search: &Entity<InputState>,
) -> impl IntoElement {
    let v_tab = view.clone();
    let v_pin = view.clone();
    let v_settings = view.clone();

    div()
        .id("toolbar")
        .w_full()
        .flex()
        .flex_col()
        .pt(px(CARD_TOP_PAD))
        .px(px(TOOLBAR_PAD_X))
        .gap(px(TOOLBAR_GAP))
        .child(
            div()
                .id("search-row")
                .w_full()
                .h(px(SEARCH_ROW_H))
                .flex()
                .items_center()
                .child(
                    div()
                        .id("search-capsule")
                        .flex_1()
                        .h(px(SEARCH_ROW_H))
                        .flex()
                        .items_center()
                        .px(pxu(20.))
                        .rounded(pxu(29.))
                        .bg(c(SEARCH_BG))
                        // 输入框区域走正常客户区：保住点击聚焦与拖选文本。
                        .occlude()
                        .child(
                            Input::new(search)
                                .appearance(false)
                                .prefix(Icon::new(IconName::Search).small()),
                        ),
                ),
        )
        .child(
            div()
                .id("tabs-row")
                .w_full()
                .h(px(TABS_ROW_H))
                .flex()
                .items_center()
                .child(
                    div()
                        .id("tabs")
                        .flex()
                        .flex_1()
                        .items_center()
                        .gap(px(TAB_GAP - s(38.0)))
                        .children(TABS.iter().enumerate().map(|(i, ic)| {
                            let active = i == tab;
                            // let color = if active { ACCENT } else { NAVY };
                            let v = v_tab.clone();
                            no_drag_div(SharedString::from(format!("tab-{i}")))
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(pxu(4.))
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    cx.stop_propagation();
                                    v.update(cx, |this, cx| {
                                        this.set_tab(i, cx);
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .id(SharedString::from(format!("tab-icon-{i}")))
                                        .size(pxu(38.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(pxu(12.))
                                        .when(!active, |d| d.hover(|s| s.bg(c(TILE_NORMAL))))
                                        .child(icons::icon(*ic, 20.).into_any_element()),
                                )
                                .when(active, |d| {
                                    d.child(
                                        div().w(pxu(30.)).h(pxu(3.)).rounded(pxu(2.)).bg(c(ACCENT)),
                                    )
                                })
                                .into_any_element()
                        })),
                )
                .child(
                    div()
                        .id("win-actions")
                        .flex()
                        .gap(pxu(6.))
                        .child(icon_btn("pin", IconName::Pin, pinned, move |cx| {
                            v_pin.update(cx, |this, cx| {
                                this.toggle_pin(cx);
                                cx.notify();
                            });
                        }))
                        .child(icon_btn("settings", IconName::Settings, false, move |cx| {
                            v_settings.update(cx, |this, cx| {
                                this.open_settings(cx);
                                cx.notify();
                            });
                        })),
                ),
        )
}

fn icon_btn(
    id: &str,
    name: IconName,
    active: bool,
    on_click: impl Fn(&mut App) + 'static,
) -> impl IntoElement {
    no_drag_div(SharedString::from(format!("btn-{id}")))
        .size(pxu(38.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(pxu(10.))
        .cursor_pointer()
        .when(active, |d| d.bg(c(ACCENT)))
        .when(!active, |d| d.hover(|s| s.bg(c(PILL))))
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            on_click(cx);
        })
        .child(if active {
            icons::icon_colored(name, 20., (255, 255, 255)).into_any_element()
        } else {
            icons::icon_colored(name, 20., NAVY).into_any_element()
        })
}
