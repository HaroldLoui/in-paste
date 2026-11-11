//! 窗口创建与 Windows 无边框处理。
//!
//! 窗口直接用无装饰（`with_decorations(false)`）创建：winit 0.30 在隐藏/显示、
//! 改尺寸等 flag 变化时会按「有装饰」重设样式；如果做法是「先有装饰再每帧剥掉」，
//! 每次 winit 贴回 `WS_CAPTION` 后客户端区都会被撑大一圈（实测每次 +22×+56），
//! 于是窗口尺寸越用越大。无装饰创建从根上避开这场拉锯。
//!
//! winit 还会给无装饰窗口留 1px 非客户区（客户区原点比窗口下移 1px），透明窗口下
//! 会露成顶部一条亮线。这里用 `SetWindowSubclass` 接管 `WM_NCCALCSIZE`，让客户区
//! 铺满整窗，从根上消掉这条线（无需 DWM 的非客户区开关）。
//!
//! 隐藏面板走 DWM cloak（见 `set_cloaked`），不走 `ShowWindow`——后者重新显示后会在
//! 顶部留下 1px 白色条带。
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

    const GWL_STYLE: i32 = -16;
    const GWL_EXSTYLE: i32 = -20;

    const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
    const WS_EX_APPWINDOW: isize = 0x0004_0000;

    const WS_CAPTION: isize = 0x00C0_0000;
    const WS_THICKFRAME: isize = 0x0004_0000;
    const WS_MINIMIZEBOX: isize = 0x0002_0000;
    const WS_MAXIMIZEBOX: isize = 0x0001_0000;
    const WS_SYSMENU: isize = 0x0008_0000;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        fn GetCursorPos(point: *mut CursorPoint) -> i32;
        fn GetForegroundWindow() -> isize;
        fn SetForegroundWindow(hwnd: isize) -> i32;
        fn IsWindow(hwnd: isize) -> i32;
        fn GetShellWindow() -> isize;
        fn SetWindowPos(
            hwnd: isize,
            after: isize,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
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

    /// 接管 `WM_NCCALCSIZE`，让客户区铺满整个窗口。
    ///
    /// winit 给无装饰窗口留了 1px 的非客户区（客户区原点比窗口下移 1px），
    /// 透明窗口下这条非客户区会露成顶部一条亮线。这里不调用默认处理，
    /// 直接以传入的「整窗矩形」作为客户区返回。
    unsafe extern "system" fn nccalc_proc(
        _hwnd: usize,
        msg: u32,
        wparam: usize,
        lparam: isize,
        _id: usize,
        _data: usize,
    ) -> isize {
        if msg == WM_NCCALCSIZE && wparam != 0 {
            // lParam 是 NCCALCSIZE_PARAMS，rgrc[0] 进来时就是整窗矩形；
            // 原样返回即「客户区 = 整窗」。
            let _ = lparam;
            return 0;
        }
        unsafe { DefSubclassProc(_hwnd as isize, msg, wparam, lparam) }
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

    /// 清理系统边框样式（含 winit 可能贴回的 `WS_CAPTION`），每帧检查一次。
    /// 首帧还会装 `WM_NCCALCSIZE` 子类化，消掉顶部 1px 非客户区亮线；
    /// 首次调用返回 true，调用方应请求重绘。
    pub fn strip_system_frame(frame: &eframe::Frame) -> bool {
        let mut resized = false;
        if let Ok(handle) = frame.window_handle()
            && let RawWindowHandle::Win32(w) = handle.as_raw()
        {
            let hwnd = w.hwnd.get();
            unsafe {
                // winit 会在启动/尺寸变化后把 WS_CAPTION 重新贴回来，
                // 每帧检查一次，只在它回来时才动样式。
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                let bare = style
                    & !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU);
                let mut changed = bare != style;
                if changed {
                    SetWindowLongPtrW(hwnd, GWL_STYLE, bare);
                }

                // 隐藏面板用的是 DWM cloak，窗口仍是 WS_VISIBLE；不设 TOOLWINDOW 的话
                // 隐藏后它还会出现在 Alt+Tab / 任务栏里。这里和剥样式一样每帧兜底。
                let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let bare_ex = (ex | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
                if bare_ex != ex {
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, bare_ex);
                    changed = true;
                }

                if changed {
                    SetWindowPos(
                        hwnd,
                        0,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                }

                if !FRAME_INIT.swap(true, Ordering::Relaxed) {
                    // 首帧装一次子类化：让客户区铺满整窗，去掉顶部 1px 非客户区亮线。
                    SetWindowSubclass(hwnd, nccalc_proc, 1, 0);
                    SetWindowPos(
                        hwnd,
                        0,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                    resized = true;
                }
            }
        }
        resized
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
    pub fn strip_system_frame(_frame: &eframe::Frame) -> bool {
        false
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
    cursor_screen, foreground, frame_hwnd, is_window, set_cloaked, set_foreground, shell_window,
    strip_system_frame,
};
