//! 右侧预览卡：头部图标与标题、正文。

use gpui_kit::assets::IconName;
use gpui_kit::*;

use crate::core::{Item, Kind};
use crate::theme::*;
use crate::ui::icons;

pub fn render(item: Item) -> impl IntoElement {
    let kind = item.kind;

    div()
        .id("preview")
        .w(px(PREVIEW_W))
        .h(px(PREVIEW_H))
        .flex()
        .flex_col()
        .rounded(px(PREVIEW_R))
        .bg(c(PREVIEW_BG))
        .shadow_md()
        .px(px(28.))
        .pt(px(26.))
        .gap(px(16.))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(16.))
                .child(
                    div()
                        .size(px(76.))
                        .rounded(px(18.))
                        .bg(c(TILE_HEADER))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(match kind {
                            Kind::Link => {
                                icons::icon_colored(IconName::Link, 34., NAVY).into_any_element()
                            }
                            Kind::Pdf => {
                                icons::icon_colored(IconName::File, 34., RED).into_any_element()
                            }
                            Kind::Image => {
                                icons::icon_colored(IconName::Image, 34., NAVY).into_any_element()
                            }
                            _ => icons::icon_colored(IconName::FileText, 34., NAVY)
                                .into_any_element(),
                        }),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .children(
                            item.preview_title()
                                .iter()
                                .filter(|l| !l.is_empty())
                                .map(|l| {
                                    div()
                                        .w_full()
                                        .truncate()
                                        .text_size(px(21.))
                                        .text_color(c(INK))
                                        .child(*l)
                                        .into_any_element()
                                }),
                        ),
                ),
        )
        .child(div().w_full().h(px(1.)).bg(c(DIVIDER)))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .children(item.preview_body().iter().map(|l| {
                    div()
                        .text_size(px(20.))
                        .text_color(c(BODY))
                        .child(*l)
                        .into_any_element()
                })),
        )
}
