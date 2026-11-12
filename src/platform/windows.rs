//! Win32 窗口层：hwnd / cloak / 置顶 / 工具窗加固 / 拖动。
//! 与 spike 相同，另加窗口拖动（WM_NCLBUTTONDOWN HTCAPTION）。

#![allow(dead_code)]

#[cfg(windows)]
pub mod win {
    use gpui_kit::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const GWL_STYLE: i32 = -16;
    const GWL_EXSTYLE: i32 = -20;

    const WS_CAPTION: isize = 0x00C0_0000;
    const WS_THICKFRAME: isize = 0x0004_0000;
    const WS_MINIMIZEBOX: isize = 0x0002_0000;
    const WS_MAXIMIZEBOX: isize = 0x0001_0000;
    const WS_SYSMENU: isize = 0x0008_0000;

    const WS_EX_TOOLWINDOW: isize = 0x0000_0080;
    const WS_EX_APPWINDOW: isize = 0x0004_0000;

    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;

    const HWND_TOPMOST: isize = -1;
    const HWND_NOTOPMOST: isize = -2;

    const DWMWA_CLOAK: u32 = 13;

    const WM_NCLBUTTONDOWN: u32 = 0x00A1;
    const HTCAPTION: usize = 0x2;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
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
        fn GetForegroundWindow() -> isize;
        fn SetForegroundWindow(hwnd: isize) -> i32;
        fn IsWindow(hwnd: isize) -> i32;
        fn GetShellWindow() -> isize;
        fn ReleaseCapture() -> i32;
        fn SendMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    }

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, value: *const u32, size: u32) -> i32;
    }

    pub fn hwnd_of(window: &Window) -> Option<isize> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::Win32(w) => Some(w.hwnd.get()),
            _ => None,
        }
    }

    /// 首帧加固：剥装饰样式、固定工具窗、清空标题、关 DWM 非客户区渲染。
    pub fn harden(window: &Window) -> Option<isize> {
        let hwnd = hwnd_of(window)?;
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let bare = style
                & !(WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU);
            if bare != style {
                SetWindowLongPtrW(hwnd, GWL_STYLE, bare);
            }

            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let tool_ex = (ex | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
            if tool_ex != ex {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, tool_ex);
            }

            let empty = [0u16];
            SetWindowTextW(hwnd, empty.as_ptr());

            SetWindowPos(
                hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );

            let policy: u32 = 1; // DWMNCRP_DISABLED
            DwmSetWindowAttribute(hwnd, 2, &policy, size_of::<u32>() as u32);
        }
        Some(hwnd)
    }

    pub fn set_cloaked(hwnd: isize, on: bool) {
        let value: u32 = u32::from(on);
        unsafe {
            DwmSetWindowAttribute(hwnd, DWMWA_CLOAK, &value, size_of::<u32>() as u32);
        }
    }

    pub fn set_topmost(hwnd: isize, on: bool) {
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

    pub fn foreground() -> isize {
        unsafe { GetForegroundWindow() }
    }

    pub fn set_foreground(hwnd: isize) {
        unsafe {
            SetForegroundWindow(hwnd);
        }
    }

    pub fn is_window(hwnd: isize) -> bool {
        unsafe { IsWindow(hwnd) != 0 }
    }

    pub fn shell_window() -> isize {
        unsafe { GetShellWindow() }
    }

    /// 模拟标题栏拖动（工具栏留白抓取区）。
    pub fn begin_drag(hwnd: isize) {
        unsafe {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0);
        }
    }
}

#[cfg(not(windows))]
pub mod win {
    use gpui_kit::Window;

    pub fn hwnd_of(_: &Window) -> Option<isize> {
        None
    }
    pub fn harden(_: &Window) -> Option<isize> {
        None
    }
    pub fn set_cloaked(_: isize, _: bool) {}
    pub fn set_topmost(_: isize, _: bool) {}
    pub fn foreground() -> isize {
        0
    }
    pub fn set_foreground(_: isize) {}
    pub fn is_window(_: isize) -> bool {
        false
    }
    pub fn shell_window() -> isize {
        0
    }
    pub fn begin_drag(_: isize) {}
}
