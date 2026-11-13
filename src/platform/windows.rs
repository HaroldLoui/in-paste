//! Win32 窗口层：hwnd / cloak / 置顶 / 工具窗加固 / 拖动。
//! 与 spike 相同，另加窗口拖动（WM_NCLBUTTONDOWN HTCAPTION）。

#![allow(dead_code)]

#[cfg(windows)]
pub mod win {
    use std::sync::atomic::{AtomicIsize, Ordering};

    use gpui_kit::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const GWL_STYLE: i32 = -16;
    const GWL_EXSTYLE: i32 = -20;
    const GWLP_WNDPROC: i32 = -4;

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

    const WM_NCCALCSIZE: u32 = 0x0083;
    const WM_NCHITTEST: u32 = 0x0084;

    const HTLEFT: isize = 10;
    const HTRIGHT: isize = 11;
    const HTTOP: isize = 12;
    const HTTOPLEFT: isize = 13;
    const HTTOPRIGHT: isize = 14;
    const HTBOTTOM: isize = 15;
    const HTBOTTOMLEFT: isize = 16;
    const HTBOTTOMRIGHT: isize = 17;

    /// 边缘缩放热区宽度（逻辑 px，按 DPI 缩放）。
    const RESIZE_EDGE_PX: f32 = 6.0;

    /// 子类化前 GPUI 的窗口过程。
    static OLD_WNDPROC: AtomicIsize = AtomicIsize::new(0);

    #[repr(C)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

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
        fn CallWindowProcW(
            prev: isize,
            hwnd: isize,
            msg: u32,
            wparam: usize,
            lparam: isize,
        ) -> isize;
        fn DefWindowProcW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
        fn GetWindowRect(hwnd: isize, rect: *mut WinRect) -> i32;
        fn GetDpiForWindow(hwnd: isize) -> u32;
        fn IsZoomed(hwnd: isize) -> i32;
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

    /// 子类化窗口过程：接管 NCCALCSIZE（客户区 = 整个窗口）与 NCHITTEST
    /// （边缘/四角优先返回系统缩放码），其余消息原样转发给 GPUI。
    fn install_resize_wndproc(hwnd: isize) -> bool {
        if OLD_WNDPROC.load(Ordering::Relaxed) != 0 {
            return true;
        }
        let old = unsafe {
            SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                resize_wndproc as *const () as usize as isize,
            )
        };
        if old == 0 {
            return false;
        }
        OLD_WNDPROC.store(old, Ordering::Relaxed);
        true
    }

    unsafe extern "system" fn resize_wndproc(
        hwnd: isize,
        msg: u32,
        wparam: usize,
        lparam: isize,
    ) -> isize {
        // 客户区铺满窗口，卡片才能贴边；最大化/全屏时交回默认处理。
        if msg == WM_NCCALCSIZE && wparam != 0 && unsafe { IsZoomed(hwnd) } == 0 {
            return 0;
        }

        // 边缘缩放优先于整卡 Drag；其它区域转发给 GPUI（它会返回 HTCAPTION）。
        if msg == WM_NCHITTEST
            && let Some(hit) = resize_hit_test(hwnd, lparam)
        {
            return hit;
        }

        // 缩放边缘的 NC 按下不能转给 GPUI：整卡 Drag 下的行/按钮会 stop_propagation，
        // 一旦被它们吞掉，DefWindowProc 就进不了系统 sizing loop。
        if msg == WM_NCLBUTTONDOWN && is_resize_code(wparam as isize) {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }

        forward(hwnd, msg, wparam, lparam)
    }

    fn forward(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
        let old = OLD_WNDPROC.load(Ordering::Relaxed);
        unsafe {
            if old == 0 {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            } else {
                CallWindowProcW(old, hwnd, msg, wparam, lparam)
            }
        }
    }

    fn is_resize_code(code: isize) -> bool {
        matches!(
            code,
            HTLEFT
                | HTRIGHT
                | HTTOP
                | HTTOPLEFT
                | HTTOPRIGHT
                | HTBOTTOM
                | HTBOTTOMLEFT
                | HTBOTTOMRIGHT
        )
    }

    /// 命中窗口四边/四角内侧 RESIZE_EDGE_PX 时返回对应 HT 码。
    fn resize_hit_test(hwnd: isize, lparam: isize) -> Option<isize> {
        unsafe {
            if IsZoomed(hwnd) != 0 {
                return None;
            }

            let mut rect = WinRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return None;
            }

            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width <= 0 || height <= 0 {
                return None;
            }

            // WM_NCHITTEST 的 lParam 是屏幕坐标。
            let x = (lparam & 0xFFFF) as i16 as i32 - rect.left;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32 - rect.top;

            let dpi = match GetDpiForWindow(hwnd) {
                0 => 96,
                dpi => dpi,
            } as f32;
            let edge = (RESIZE_EDGE_PX * dpi / 96.0).round().max(4.0) as i32;

            let left = x <= edge;
            let right = x >= width - edge;
            let top = y <= edge;
            let bottom = y >= height - edge;

            let hit = match (top, bottom, left, right) {
                (true, _, true, _) => HTTOPLEFT,
                (true, _, _, true) => HTTOPRIGHT,
                (_, true, true, _) => HTBOTTOMLEFT,
                (_, true, _, true) => HTBOTTOMRIGHT,
                (true, _, _, _) => HTTOP,
                (_, true, _, _) => HTBOTTOM,
                (_, _, true, _) => HTLEFT,
                (_, _, _, true) => HTRIGHT,
                _ => return None,
            };
            Some(hit)
        }
    }

    /// 首帧加固：剥装饰样式、加 thick frame、固定工具窗、清空标题、关 DWM 非客户区渲染。
    pub fn harden(window: &Window) -> Option<isize> {
        let hwnd = hwnd_of(window)?;
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            // 保留/补回 WS_THICKFRAME，缩放交给系统 sizing loop；
            // 其余 caption / min / max / sysmenu 仍然剥掉。
            let bare = (style & !(WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU))
                | WS_THICKFRAME;
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

            // 必须在 FRAMECHANGED 之前装好，否则这次 NCCALCSIZE 会走 GPUI 的默认处理。
            if !install_resize_wndproc(hwnd) {
                eprintln!("[in-paste] failed to install resize wndproc");
            }

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
