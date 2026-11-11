//! 窗口创建与 Windows 无边框处理。
//!
//! 窗口用无装饰（`with_decorations(false)`）创建。winit 0.30 的「无边框」其实是
//! **保留 `WS_CAPTION|WS_BORDER|...` 样式、靠 `WM_NCCALCSIZE` 把非客户区吃掉**，
//! 并且会在窗口属性 diff 时用 `to_window_styles()` 反复重写整套装饰样式。
//!
//! 因此这里首帧做一次窗口子类化，在系统应用样式之前就把装饰位过滤掉
//! （`WM_STYLECHANGING`），并在 `WM_NCCALCSIZE` 里让客户区铺满整窗
//! （顺带消掉 winit 给阴影预留的 1px 亮线）。winit 之后再怎么重写样式，
//! 窗口都不会真正进入「有 caption」状态。
//!
//! 另：用户看到的「标题栏一闪而过」主因是 debug 构建的控制台窗口从透明留白里
//! 透出来（见 `main.rs` 的 GUI 子系统说明），不完全是窗口样式问题；这里的过滤
//! 属于必要的加固，但偶发闪烁仍在（见 docs/REVIEW.md §7.5）。
//!
//! 隐藏面板走 DWM cloak（见 `set_cloaked`），不走 `ShowWindow`——后者重新显示后
//! 顶部会留下 1px 白色条带。置顶/取消置顶走 `set_topmost` 直接改 z-order，绕开
//! winit 的 `set_window_level`（它同样会触发整写样式）。
//!
//! 窗口尺寸一律以「设计点」表达，原生尺寸乘 `theme::UI_SCALE`；
//! 拖拽热区与自定义缩放都在 `app` 里用 `ViewportCommand` 完成。

use eframe::egui::ViewportBuilder;

use crate::theme;

/// 原生逻辑尺寸（winit 用的点）= 设计点 × UI_SCALE。
fn native_size(design: eframe::egui::Vec2) -> [f32; 2] {
    [design.x * theme::UI_SCALE, design.y * theme::UI_SCALE]
}

pub fn native_options() -> eframe::NativeOptions {
    let default_win = theme::window_size(
        eframe::egui::vec2(theme::DEFAULT_CARD_W, theme::DEFAULT_CARD_H),
        false,
    );
    let min_win = theme::window_size(
        eframe::egui::vec2(theme::MIN_CARD_W, theme::MIN_CARD_H),
        false,
    );

    eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("in-paste")
            .with_inner_size(native_size(default_win))
            .with_min_inner_size(native_size(min_win))
            .with_decorations(false)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_resizable(false)
            .with_taskbar(false)
            .with_active(true),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    }
}

#[cfg(windows)]
mod platform {
    use std::sync::atomic::{AtomicBool, Ordering};

    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const GWL_EXSTYLE: i32 = -20;

    const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
    const WS_EX_APPWINDOW: isize = 0x0004_0000;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        fn GetCursorPos(point: *mut CursorPoint) -> i32;
        fn GetForegroundWindow() -> isize;
        fn SetForegroundWindow(hwnd: isize) -> i32;
        fn IsWindow(hwnd: isize) -> i32;
        fn GetShellWindow() -> isize;
        fn RedrawWindow(
            hwnd: isize,
            update_rect: *const core::ffi::c_void,
            update_region: isize,
            flags: u32,
        ) -> i32;
        fn SetWindowPos(
            hwnd: isize,
            after: isize,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
        fn SetWindowTextW(hwnd: isize, text: *const u16) -> i32;
    }

    #[repr(C)]
    struct CursorPoint {
        x: i32,
        y: i32,
    }

    type SubclassProc = unsafe extern "system" fn(usize, u32, usize, isize, usize, usize) -> isize;

    #[link(name = "comctl32")]
    unsafe extern "system" {
        fn SetWindowSubclass(hwnd: isize, proc: SubclassProc, id: usize, ref_data: usize) -> i32;
        fn DefSubclassProc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    }

    const WM_NCCALCSIZE: u32 = 0x0083;
    /// 窗口样式即将改变（`SetWindowLong(GWL_STYLE/GWL_EXSTYLE)` 时同步发出）。
    const WM_STYLECHANGING: u32 = 0x007C;
    const GWL_STYLE: i32 = -16;

    const WS_CAPTION: isize = 0x00C0_0000;
    const WS_THICKFRAME: isize = 0x0004_0000;
    const WS_MINIMIZEBOX: isize = 0x0002_0000;
    const WS_MAXIMIZEBOX: isize = 0x0001_0000;
    const WS_SYSMENU: isize = 0x0008_0000;

    /// `WM_STYLECHANGING` 的 lParam 指向的结构（`STYLESTRUCT`）。
    ///
    /// 注意字段是 32 位 `LONG`，不是指针宽度的 `isize`：写成 isize 会把相邻内存
    /// 当成 styleNew 改写，既过滤不掉样式还会写坏栈。
    #[repr(C)]
    struct StyleStruct {
        old: i32,
        new: i32,
    }

    /// 窗口子类化：在系统应用样式之前把装饰位去掉，并接管非客户区计算。
    ///
    /// winit 0.30 的无边框实现是「保留 `WS_CAPTION` 等装饰样式，靠 `WM_NCCALCSIZE`
    /// 把非客户区吃掉」。它每次做窗口属性 diff 都会用 `to_window_styles()` 重写整套
    /// 装饰样式；如果只在外面每帧剥样式，就会和 winit 的样式写入来回打架（实测把
    /// 客户区撑大 22×56）。所以改成：
    ///
    /// - `WM_STYLECHANGING`：改写即将写入的样式，永远维持「无 caption」，
    ///   winit 后续再重写也只会被这里过滤掉，不再有样式来回；
    /// - `WM_NCCALCSIZE`：客户区 = 整窗（顺带消掉 winit 给阴影预留的 1px 亮线）。
    unsafe extern "system" fn wnd_subclass_proc(
        hwnd: usize,
        msg: u32,
        wparam: usize,
        lparam: isize,
        _id: usize,
        _data: usize,
    ) -> isize {
        match msg {
            WM_NCCALCSIZE if wparam != 0 => {
                // lParam 是 NCCALCSIZE_PARAMS，rgrc[0] 进来时就是整窗矩形；
                // 原样返回即「客户区 = 整窗」。
                let _ = lparam;
                0
            }
            WM_STYLECHANGING => {
                let styles = unsafe { &mut *(lparam as *mut StyleStruct) };
                let decor =
                    (WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU)
                        as i32;
                if wparam as i32 == GWL_STYLE {
                    styles.new &= !decor;
                } else if wparam as i32 == GWL_EXSTYLE {
                    styles.new = (styles.new | WS_EX_TOOLWINDOW as i32) & !(WS_EX_APPWINDOW as i32);
                }
                unsafe { DefSubclassProc(hwnd as isize, msg, wparam, lparam) }
            }
            _ => unsafe { DefSubclassProc(hwnd as isize, msg, wparam, lparam) },
        }
    }

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, value: *const u32, size: u32) -> i32;
    }

    /// DWM 隐藏窗口（从合成器里摘掉，不改变 `IsWindowVisible` 状态）。
    const DWMWA_CLOAK: u32 = 13;

    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;

    static FRAME_INIT: AtomicBool = AtomicBool::new(false);

    /// 当前鼠标位置（**物理像素**，虚拟屏幕坐标）。
    ///
    /// 自定义缩放用它算绝对位移：窗口在缩放过程中会移动，如果用窗口内相对坐标，
    /// 「窗口移动」会被重复计入位移，形成正反馈越拉越大。
    pub fn cursor_screen() -> Option<(f32, f32)> {
        let mut point = CursorPoint { x: 0, y: 0 };
        let ok = unsafe { GetCursorPos(&mut point) };
        (ok != 0).then_some((point.x as f32, point.y as f32))
    }

    /// 首帧初始化：装窗口子类化（接管非客户区），并把窗口设成工具窗。
    /// 首次调用返回 true，调用方应请求重绘。
    ///
    /// 不再每帧剥 `WS_CAPTION` 等样式：winit 本来就保留这些样式、靠
    /// `WM_NCCALCSIZE` 藏边框；剥样式只会和 winit 的样式重写互相打架，
    /// 在两者之间露出真标题栏。
    pub fn init_window(frame: &eframe::Frame) -> bool {
        let Some(hwnd) = frame_hwnd(frame) else {
            return false;
        };
        if FRAME_INIT.swap(true, Ordering::Relaxed) {
            return false;
        }
        unsafe {
            let _ = SetWindowSubclass(hwnd, wnd_subclass_proc, 1, 0);

            // 首次把已存在的装饰样式清掉；之后 winit 再怎么写都会被
            // WM_STYLECHANGING 过滤器改回「无 caption」。
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let bare = style
                & !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU);
            if bare != style {
                SetWindowLongPtrW(hwnd, GWL_STYLE, bare);
            }

            // cloak 隐藏时窗口仍是 WS_VISIBLE，设成工具窗避免出现在 Alt+Tab / 任务栏。
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let tool_ex = (ex | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
            if tool_ex != ex {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, tool_ex);
            }

            // 清掉窗口标题：DWM 在激活 / 改 z-order 时偶尔会从透明留白里
            // 把窗口标题文字画出来（看起来就是标题栏闪一下）。窗口是工具窗，
            // 不需要标题，直接置空后这条残影就不会再出现。
            let empty = [0u16];
            SetWindowTextW(hwnd, empty.as_ptr());

            // 让新的非客户区计算立刻生效，并清掉 DWM 在创建阶段缓存的装饰帧。
            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
            disable_nc_rendering(hwnd);
        }
        true
    }

    /// 关闭 DWM 的非客户区渲染并强制重建窗口框架。
    ///
    /// 窗口创建时 winit 是按「有装饰」写样式的，DWM 可能缓存了那一帧装饰外观；
    /// 之后即使样式被我们过滤成无 caption，激活/失焦时 DWM 仍可能把缓存帧短暂
    /// 合成出来（标题栏闪一下）。关闭 NC 渲染 + `RDW_FRAME` 立即重建可以清掉缓存。
    fn disable_nc_rendering(hwnd: isize) {
        const RDW_INVALIDATE: u32 = 0x0001;
        const RDW_UPDATENOW: u32 = 0x0100;
        const RDW_FRAME: u32 = 0x0400;
        let policy: u32 = 1; // DWMNCRP_DISABLED
        unsafe {
            DwmSetWindowAttribute(hwnd, 2, &policy, size_of::<u32>() as u32);
            RedrawWindow(
                hwnd,
                core::ptr::null(),
                0,
                RDW_INVALIDATE | RDW_FRAME | RDW_UPDATENOW,
            );
        }
    }

    /// 置顶 / 取消置顶，绕开 winit 的 `set_window_level`。
    ///
    /// `ViewportCommand::WindowLevel` 会走 winit 的 window-flags diff，进而用
    /// `to_window_styles()` 重写整套装饰样式并 `SWP_FRAMECHANGED`，是标题栏闪烁的
    /// 直接触发点。直接 `SetWindowPos` 改 z-order 不碰样式。
    pub fn set_topmost(frame: &eframe::Frame, on: bool) {
        const HWND_TOPMOST: isize = -1;
        const HWND_NOTOPMOST: isize = -2;
        if let Some(hwnd) = frame_hwnd(frame) {
            unsafe {
                SetWindowPos(
                    hwnd,
                    if on { HWND_TOPMOST } else { HWND_NOTOPMOST },
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }

    pub fn frame_hwnd(frame: &eframe::Frame) -> Option<isize> {
        if let Ok(handle) = frame.window_handle()
            && let RawWindowHandle::Win32(w) = handle.as_raw()
        {
            Some(w.hwnd.get())
        } else {
            None
        }
    }

    /// 用 DWM cloak 隐藏/显示窗口。
    ///
    /// 不用 `ViewportCommand::Visible`：它走 `ShowWindow`，重新显示后窗口顶部会留下
    /// 一条 1px 的白色非客户区条带（透明窗口下非常显眼）。cloak 只是把窗口从合成器
    /// 里摘掉，重新放回来后没有这条线。
    pub fn set_cloaked(frame: &eframe::Frame, on: bool) {
        if let Some(hwnd) = frame_hwnd(frame) {
            let value: u32 = u32::from(on);
            unsafe {
                DwmSetWindowAttribute(hwnd, DWMWA_CLOAK, &value, size_of::<u32>() as u32);
            }
        }
    }

    /// 当前前台窗口（可能属于别的进程）。
    pub fn foreground() -> Option<isize> {
        let hwnd = unsafe { GetForegroundWindow() };
        (hwnd != 0).then_some(hwnd)
    }

    pub fn is_window(hwnd: isize) -> bool {
        unsafe { IsWindow(hwnd) != 0 }
    }

    /// 桌面外壳窗口：没有别的可归还对象时用它兜底，避免把焦点留在隐藏窗口上。
    pub fn shell_window() -> Option<isize> {
        let hwnd = unsafe { GetShellWindow() };
        (hwnd != 0).then_some(hwnd)
    }

    /// 把前台还给指定窗口（cloak 隐藏后系统不会自动归还）。
    pub fn set_foreground(hwnd: isize) {
        unsafe {
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn init_window(_frame: &eframe::Frame) -> bool {
        false
    }

    pub fn set_topmost(_frame: &eframe::Frame, _on: bool) {}

    pub fn frame_hwnd(_frame: &eframe::Frame) -> Option<isize> {
        None
    }

    pub fn cursor_screen() -> Option<(f32, f32)> {
        None
    }

    pub fn set_cloaked(_frame: &eframe::Frame, _on: bool) {}

    pub fn foreground() -> Option<isize> {
        None
    }

    pub fn is_window(_hwnd: isize) -> bool {
        false
    }

    pub fn set_foreground(_hwnd: isize) {}

    pub fn shell_window() -> Option<isize> {
        None
    }
}

pub use platform::{
    cursor_screen, foreground, frame_hwnd, init_window, is_window, set_cloaked, set_foreground,
    set_topmost, shell_window,
};
