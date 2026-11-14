//! 右侧预览卡：头部图标与标题、正文。

use gpui_kit::assets::IconName;
use gpui_kit::prelude::FluentBuilder;
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
        .px(pxu(28.))
        .pt(pxu(26.))
        .gap(pxu(16.))
        .child(
            div()
                .flex()
                .items_start()
                .gap(pxu(16.))
                .child(
                    div()
                        .size(pxu(76.))
                        .rounded(pxu(18.))
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
                        .gap(pxu(4.))
                        .children(
                            item.preview_title()
                                .iter()
                                .filter(|l| !l.is_empty())
                                .map(|l| {
                                    div()
                                        .w_full()
                                        .truncate()
                                        .text_size(pxu(21.))
                                        .text_color(c(INK))
                                        .child(*l)
                                        .into_any_element()
                                }),
                        ),
                ),
        )
        .child(div().w_full().h(pxu(1.)).bg(c(DIVIDER)))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(pxu(8.))
                .children(item.preview_body().iter().map(|l| {
                    div()
                        .text_size(pxu(20.))
                        .text_color(c(BODY))
                        .child(*l)
                        .into_any_element()
                })),
        )
}

/// 预览卡四周的透明留白，用来容纳 `preview` 的阴影和箭头。
pub const PREVIEW_MARGIN: f32 = 24.0;
/// 预览窗口与屏幕边缘保留的距离。
pub const PREVIEW_SCREEN_MARGIN: f32 = 8.0;
/// 预览窗在左侧时，卡片在窗内额外左移一点，给右侧箭头留空间。
const PREVIEW_LEFT_CARD_X: f32 = 10.0;
pub const PREVIEW_WIN_W: f32 = PREVIEW_W + PREVIEW_MARGIN * 2.0;
pub const PREVIEW_WIN_H: f32 = PREVIEW_H + PREVIEW_MARGIN * 2.0;

/// 预览窗相对主窗的位置；同时决定三角箭头画在哪一侧。
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewSide {
    /// 预览窗在主窗右侧，箭头在预览卡左侧。
    #[default]
    Right,
    /// 预览窗在主窗左侧，箭头在预览卡右侧。
    Left,
}

/// 主窗口与预览窗口共享的状态。
#[derive(Default)]
pub struct PreviewState {
    pub item: Option<Item>,
    pub side: PreviewSide,
}

/// 预览窗口的根视图，只观察 `PreviewState` 并绘制卡片。
pub struct PreviewWindow {
    state: Entity<PreviewState>,
    _observe: Subscription,
}

impl PreviewWindow {
    pub fn new(state: Entity<PreviewState>, cx: &mut Context<Self>) -> Self {
        let _observe = cx.observe(&state, |_, _, cx| cx.notify());
        Self { state, _observe }
    }
}

impl Render for PreviewWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let item = state.item;
        let side = state.side;
        let content_h = f32::from(window.viewport_size().height);
        let card_x = match side {
            PreviewSide::Right => PREVIEW_MARGIN,
            PreviewSide::Left => PREVIEW_LEFT_CARD_X,
        };

        div()
            .size_full()
            .flex()
            .items_center()
            .pl(px(card_x))
            .when_some(item, |this, item| this.child(render(item)))
            .child(deferred(arrow(side, card_x, content_h)))
    }
}

/// 画在透明留白里的三角箭头，尖端朝主窗，底边贴着预览卡。
fn arrow(side: PreviewSide, card_x: f32, content_h: f32) -> AnyElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let window_x = f32::from(bounds.origin.x);
            let window_y = f32::from(bounds.origin.y);
            let center_y = window_y + content_h / 2.0;
            let card_left = window_x + card_x;
            let card_right = card_left + PREVIEW_W;
            let half = 18.0 * UI_SCALE;
            let right_depth = 20.0 * UI_SCALE;
            let left_depth = 18.0 * UI_SCALE;

            let (tip_x, base_x) = match side {
                PreviewSide::Right => (card_left - right_depth, card_left + 1.0),
                PreviewSide::Left => (card_right + left_depth, card_right - 1.0),
            };

            let mut path = PathBuilder::fill();
            path.move_to(point(px(tip_x), px(center_y)));
            path.line_to(point(px(base_x), px(center_y - half)));
            path.line_to(point(px(base_x), px(center_y + half)));
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, c(PREVIEW_BG));
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full()
    .into_any_element()
}
