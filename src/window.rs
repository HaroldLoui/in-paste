//! 窗口创建与 Windows 无边框处理。
//!
//! eframe 在 Windows 上会忽略 `with_has_shadow(false)`，winit 还会给无边框
//! 窗口预留 1px 的非客户区（顶部出现一条线）。这里的做法是：
//! 让 winit 走「有装饰」路径以绕开那 1px，再由 `strip_system_frame`
//! 每帧剥掉标题栏样式，并用 DWM 关掉非客户区渲染。

use eframe::egui::ViewportBuilder;

use crate::theme;

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("in-paste")
            .with_inner_size([
                theme::BASE_W * theme::UI_SCALE,
                theme::BASE_H * theme::UI_SCALE,
            ])
            .with_decorations(true)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_resizable(false)
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

    use crate::theme;

    const GWL_STYLE: i32 = -16;

    const WS_CAPTION: isize = 0x00C0_0000;
    const WS_THICKFRAME: isize = 0x0004_0000;
    const WS_MINIMIZEBOX: isize = 0x0002_0000;
    const WS_MAXIMIZEBOX: isize = 0x0001_0000;
    const WS_SYSMENU: isize = 0x0008_0000;

    #[repr(C)]
    struct Margins {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        fn GetDpiForWindow(hwnd: isize) -> u32;
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

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            attr: u32,
            value: *const core::ffi::c_void,
            size: u32,
        ) -> i32;
        fn DwmExtendFrameIntoClientArea(hwnd: isize, margins: *const Margins) -> i32;
    }

    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;

    static FRAME_INIT: AtomicBool = AtomicBool::new(false);

    /// 剥离系统边框并校正窗口尺寸；本帧动过窗口时返回 true，调用方应请求重绘。
    pub fn strip_system_frame(frame: &eframe::Frame) -> bool {
        let mut resized = false;
        if let Ok(handle) = frame.window_handle() {
            if let RawWindowHandle::Win32(w) = handle.as_raw() {
                let hwnd = w.hwnd.get();
                unsafe {
                    // winit 会在启动/尺寸变化后把 WS_CAPTION 重新贴回来，
                    // 每帧检查一次，只在它回来时才动样式。
                    let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                    let bare = style
                        & !(WS_CAPTION
                            | WS_THICKFRAME
                            | WS_MINIMIZEBOX
                            | WS_MAXIMIZEBOX
                            | WS_SYSMENU);
                    if bare != style {
                        SetWindowLongPtrW(hwnd, GWL_STYLE, bare);
                        SetWindowPos(
                            hwnd,
                            0,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOSIZE
                                | SWP_NOMOVE
                                | SWP_NOZORDER
                                | SWP_NOACTIVATE
                                | SWP_FRAMECHANGED,
                        );
                    }

                    if !FRAME_INIT.swap(true, Ordering::Relaxed) {
                        let border: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE
                        let corners: u32 = 1; // DWMWCP_DONOTROUND
                        let policy: u32 = 1; // DWMNCRP_DISABLED
                        let margins = Margins {
                            left: 0,
                            right: 0,
                            top: 0,
                            bottom: 0,
                        };
                        DwmSetWindowAttribute(hwnd, 34, &border as *const u32 as *const _, 4);
                        DwmSetWindowAttribute(hwnd, 33, &corners as *const u32 as *const _, 4);
                        DwmSetWindowAttribute(hwnd, 2, &policy as *const u32 as *const _, 4);
                        DwmExtendFrameIntoClientArea(hwnd, &margins);

                        // decorations(true) 时 winit 按带标题栏的外框算了一次尺寸，
                        // 样式剥掉后把窗口收回成客户区大小。
                        //
                        // SetWindowPos 的单位是物理像素，必须按窗口当前 DPI 换算，
                        // 并叠加界面整体缩放系数 UI_SCALE。
                        // 直接传 BASE_W/BASE_H 时，150% 缩放下窗口只有设计尺寸的
                        // 2/3，egui 拿到的画布也随之变小，首帧就只画出左上角一块；
                        // 点击行触发 InnerSize（逻辑点）后才恢复正常。
                        let dpi = GetDpiForWindow(hwnd).max(96) as f32;
                        let scale = dpi / 96.0;
                        let w = (theme::BASE_W * theme::UI_SCALE * scale).round() as i32;
                        let h = (theme::BASE_H * theme::UI_SCALE * scale).round() as i32;
                        SetWindowPos(
                            hwnd,
                            0,
                            0,
                            0,
                            w,
                            h,
                            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                        );
                        resized = true;
                    }
                }
            }
        }
        resized
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn strip_system_frame(_frame: &eframe::Frame) -> bool {
        false
    }
}

pub use platform::strip_system_frame;
