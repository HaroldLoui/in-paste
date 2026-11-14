//! 界面绘制层。

pub mod app;
pub mod icons;
pub mod preview;
pub mod preview_window;
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

/// 交互组件挡板：整卡铺了 [`drag_zone`]，可交互组件必须在下按时停冒泡，
/// 否则这次按下会被 WM_NCHITTEST 当成拖标题栏，click 永远完不成。
///
/// 只适合"点击"类控件。需要按住拖动、依赖鼠标移动的控件（文本输入/选择、
/// 滑杆、拖拽手柄、弹层菜单）请改用 `.occlude()`：整卡 Drag 下鼠标移动走
/// NC 通道，`pressed_button` 恒为 `None`，停冒泡只能保住 click，保不住拖动手势。
pub fn no_drag_div(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
}
