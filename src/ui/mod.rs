//! 界面绘制层。

pub mod app;
pub mod icons;
pub mod preview;
pub mod row;
pub mod settings;
pub mod toolbar;

use gpui_kit::{Div, InteractiveElement as _, Stateful, WindowControlArea, div};

/// 空白拖拽热区：透明、无其它交互，只把这块区域声明成窗口拖动区。
///
/// 只能放在纯留白上；一旦覆盖到可交互控件，NC 命中会抢走那些控件的鼠标事件。
pub fn drag_zone(id: &'static str) -> Stateful<Div> {
    div().id(id).window_control_area(WindowControlArea::Drag)
}
