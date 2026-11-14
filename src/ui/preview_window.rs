//! 独立预览窗口：主窗口之外的第二个原生窗口。
//!
//! 这个窗口只负责显示当前选中的记录，不接收交互；定位、owner、cloak 由
//! `platform::windows` 负责。窗口四周留一圈透明边给 `preview` 的阴影和箭头。

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::core::Item;
use crate::theme::*;
use crate::ui::preview;

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
            .when_some(item, |this, item| this.child(preview::render(item)))
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
            let half = 18.0;

            let (tip_x, base_x) = match side {
                PreviewSide::Right => (card_left - 20.0, card_left + 1.0),
                PreviewSide::Left => (card_right + 18.0, card_right - 1.0),
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
