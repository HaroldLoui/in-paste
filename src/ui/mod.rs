//! 界面绘制层。

pub mod app;
pub mod icons;
pub mod preview;
pub mod row;
pub mod settings;
pub mod toolbar;

use gpui_kit::{Div, ElementId, InteractiveElement as _, Stateful, WindowControlArea, div};

/// 窗口拖动热区：透明、无其它交互，只声明这块区域可以拖动窗口。
///
/// 整卡铺一层后，所有可交互组件必须在 mouse down 时 `stop_propagation`，
/// 否则那块区域的点击会被 WM_NCHITTEST 当成拖标题栏（click 永远完不成）。
pub fn drag_zone(id: &'static str) -> Stateful<Div> {
    div().id(id).window_control_area(WindowControlArea::Drag)
}

/// 因为整个窗口实际都是一个可拖动的`drag_zone`，所以可交互的组件都需要加`stop_propagation`避免被`drag`事件覆盖原事件行为
pub fn no_drag_div(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
}
