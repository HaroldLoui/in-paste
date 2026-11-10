// 剪贴板管理器 UI 复刻（egui 0.37）
// 初始只显示主卡；点击某行后窗口向右展开，并在右侧显示该行预览。
// 数据为设计稿示例。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Painter,
    Pos2, Rect, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2, epaint::Shadow,
    pos2, vec2,
};

const BASE_W: f32 = 828.0;
const BASE_H: f32 = 620.0;
const FULL_W: f32 = 1279.0;

const CARD_X: f32 = 24.0;
const CARD_Y: f32 = 40.0;
const CARD_W: f32 = 780.0;
const CARD_H: f32 = 516.0;
const CARD_R: u8 = 20;

const PREVIEW_X: f32 = 824.0;
const PREVIEW_W: f32 = 431.0;
const PREVIEW_H: f32 = 338.0;
const PREVIEW_R: u8 = 18;

const CARD_BG: Color32 = Color32::from_rgb(0xF6, 0xF6, 0xFD);
const PREVIEW_BG: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xFD);
const SELECT: Color32 = Color32::from_rgb(0xC6, 0xD9, 0xFE);
const INK: Color32 = Color32::from_rgb(0x0B, 0x0D, 0x1A);
const BODY: Color32 = Color32::from_rgb(0x1F, 0x23, 0x44);
const MUTED: Color32 = Color32::from_rgb(0x6A, 0x75, 0x97);
const PLACEHOLDER: Color32 = Color32::from_rgb(0x6B, 0x71, 0x90);
const NAVY: Color32 = Color32::from_rgb(0x3F, 0x45, 0x68);
const ACCENT: Color32 = Color32::from_rgb(0x0D, 0x6B, 0xFC);
const BUTTON: Color32 = Color32::from_rgb(0x11, 0x67, 0xFD);
const TILE_SELECTED: Color32 = Color32::from_rgb(0xD9, 0xE3, 0xFE);
const TILE_NORMAL: Color32 = Color32::from_rgb(0xED, 0xEE, 0xF9);
const TILE_PDF: Color32 = Color32::from_rgb(0xF6, 0xE3, 0xEA);
const TILE_HEADER: Color32 = Color32::from_rgb(0xD2, 0xE0, 0xFD);
const SEARCH_BG: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xF5);
const DIVIDER: Color32 = Color32::from_rgb(0xE6, 0xE7, 0xF2);
const PILL: Color32 = Color32::from_rgb(0xE5, 0xE7, 0xF4);
const PILL_SELECTED: Color32 = Color32::from_rgb(0xB7, 0xCB, 0xF5);
const RED: Color32 = Color32::from_rgb(0xE2, 0x3B, 0x34);

const ROW_INSET: f32 = 10.0;
const ROW_W: f32 = 758.0;
const ROW_H: f32 = 81.0;
const ROW0_H: f32 = 90.0;
const TILE: f32 = 64.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("in-paste")
            .with_inner_size([BASE_W, BASE_H])
            .with_decorations(true)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_resizable(false)
            .with_active(true),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "in-paste",
        options,
        Box::new(|cc| Ok(Box::new(UiApp::new(cc)))),
    )
}

struct UiApp {
    thumb: Option<TextureHandle>,
    selected: Option<usize>,
}

impl UiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        Self {
            thumb: load_thumb(&cc.egui_ctx),
            selected: None,
        }
    }
}

#[cfg(windows)]
#[repr(C)]
struct Margins {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

#[cfg(windows)]
#[cfg(windows)]
const GWL_STYLE: i32 = -16;

#[cfg(windows)]
const WS_CAPTION: isize = 0x00C0_0000;

#[cfg(windows)]
const WS_THICKFRAME: isize = 0x0004_0000;

#[cfg(windows)]
const WS_MINIMIZEBOX: isize = 0x0002_0000;

#[cfg(windows)]
const WS_MAXIMIZEBOX: isize = 0x0001_0000;

#[cfg(windows)]
const WS_SYSMENU: isize = 0x0008_0000;

#[cfg(windows)]
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
}

#[cfg(windows)]
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

static FRAME_INIT: AtomicBool = AtomicBool::new(false);

fn strip_system_frame(frame: &eframe::Frame) {
    #[cfg(windows)]
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
                        0x0001 | 0x0002 | 0x0004 | 0x0010 | 0x0020,
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
                    // 样式剥掉后把窗口收回成客户区大小，避免右侧/底部多出一截透明边。
                    SetWindowPos(
                        hwnd,
                        0,
                        0,
                        0,
                        BASE_W as i32,
                        BASE_H as i32,
                        0x0002 | 0x0004 | 0x0010 | 0x0020,
                    );
                }
            }
        }
    }
}

impl eframe::App for UiApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        strip_system_frame(frame);

        let full = ui.max_rect();
        let origin = full.min;

        let card =
            Rect::from_min_size(origin + vec2(CARD_X, CARD_Y), vec2(CARD_W, CARD_H));

        if let Some(i) = draw_main_card(ui, card, self.thumb.as_ref(), self.selected) {
            if self.selected != Some(i) {
                self.selected = Some(i);
                let want = FULL_W;
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(want, BASE_H)));
            }
        }

        if self.selected.is_some() && (full.width() - FULL_W).abs() > 1.0 {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(FULL_W, BASE_H)));
        }

        if let Some(i) = self.selected {
            // 预览卡跟着选中行移动，箭头才能始终指准那一行。
            let arrow_y = row_rect(card, i).center().y;
            let top = (arrow_y - 70.0).clamp(CARD_Y, BASE_H - PREVIEW_H - 40.0);
            let preview = Rect::from_min_size(
                origin + vec2(PREVIEW_X, top),
                vec2(PREVIEW_W, PREVIEW_H),
            );
            draw_preview(ui.painter(), preview, &ITEMS[i], arrow_y, self.thumb.as_ref());
        }

    }
}

fn load_thumb(ctx: &egui::Context) -> Option<TextureHandle> {
    let bytes = include_bytes!("../assets/thumb.png");
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture("thumb", color, TextureOptions::LINEAR))
}

fn read_font(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

fn install_fonts(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    let default_prop = defs
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    let mut prop: Vec<String> = Vec::new();
    let mut bold: Vec<String> = Vec::new();
    let mut icons: Vec<String> = Vec::new();

    let mut add = |name: &str, path: &str, target: &mut Vec<String>| {
        if let Some(bytes) = read_font(path) {
            defs.font_data
                .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
            target.push(name.to_owned());
        }
    };

    add("segoe", "C:/Windows/Fonts/segoeui.ttf", &mut prop);
    add("yahei", "C:/Windows/Fonts/msyh.ttc", &mut prop);
    add("segoe_bold", "C:/Windows/Fonts/segoeuib.ttf", &mut bold);
    add("yahei_bold", "C:/Windows/Fonts/msyhbd.ttc", &mut bold);
    add("icons", "C:/Windows/Fonts/SegoeIcons.ttf", &mut icons);

    prop.extend(default_prop.iter().cloned());
    bold.extend(default_prop.iter().cloned());

    defs.families.insert(FontFamily::Proportional, prop);
    defs.families
        .insert(FontFamily::Name(Arc::from("bold")), bold);
    if !icons.is_empty() {
        defs.families
            .insert(FontFamily::Name(Arc::from("icons")), icons);
    }

    ctx.set_fonts(defs);
}

fn icons_family() -> FontFamily {
    FontFamily::Name(Arc::from("icons"))
}

fn bold_family() -> FontFamily {
    FontFamily::Name(Arc::from("bold"))
}

fn icon(p: &Painter, center: Pos2, size: f32, ch: char, color: Color32) {
    p.text(
        center + vec2(0.0, 1.0),
        Align2::CENTER_CENTER,
        ch,
        FontId::new(size, icons_family()),
        color,
    );
}

fn icon_bold(p: &Painter, center: Pos2, size: f32, ch: char, color: Color32) {
    for (dx, dy) in [(-0.55, 0.0), (0.55, 0.0), (0.0, -0.55), (0.0, 0.55)] {
        p.text(
            center + vec2(dx, dy + 1.0),
            Align2::CENTER_CENTER,
            ch,
            FontId::new(size, icons_family()),
            color,
        );
    }
}

fn draw_card(p: &Painter, rect: Rect, radius: u8, fill: Color32) {
    p.add(
        Shadow {
            offset: [0, 10],
            blur: 34,
            spread: 0,
            color: Color32::from_black_alpha(26),
        }
        .as_shape(rect, CornerRadius::same(radius)),
    );
    p.add(
        Shadow {
            offset: [0, 2],
            blur: 8,
            spread: 0,
            color: Color32::from_black_alpha(16),
        }
        .as_shape(rect, CornerRadius::same(radius)),
    );
    p.rect_filled(rect, CornerRadius::same(radius), fill);
    p.rect_stroke(
        rect.shrink(0.5),
        CornerRadius::same(radius),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 110)),
        StrokeKind::Inside,
    );
}

fn rr(card: Rect, x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(card.min + vec2(x, y), vec2(w, h))
}

// ---------------------------------------------------------------- 主卡

struct Item {
    kind: Kind,
    title: &'static str,
    meta: &'static str,
    key: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Image,
    Link,
    Pdf,
    Rich,
}

const ITEMS: [Item; 5] = [
    Item {
        kind: Kind::Text,
        title: "这是一个示例文本，展示剪贴板工具的效果...",
        meta: "刚刚",
        key: "Ctrl + 1",
    },
    Item {
        kind: Kind::Image,
        title: "风景图片.png",
        meta: "1 分钟前",
        key: "Ctrl + 2",
    },
    Item {
        kind: Kind::Link,
        title: "https://www.rust-lang.org/",
        meta: "2 分钟前",
        key: "Ctrl + 3",
    },
    Item {
        kind: Kind::Pdf,
        title: "design_spec.pdf",
        meta: "3 分钟前 · 2.5 MB",
        key: "Ctrl + 4",
    },
    Item {
        kind: Kind::Rich,
        title: "使用 Rust 开发高效的工具...",
        meta: "5 分钟前",
        key: "Ctrl + 5",
    },
];

fn draw_main_card(
    ui: &mut egui::Ui,
    card: Rect,
    thumb: Option<&TextureHandle>,
    selected: Option<usize>,
) -> Option<usize> {
    let p = ui.painter().clone();
    draw_card(&p, card, CARD_R, CARD_BG);
    draw_toolbar(&p, card);

    let mut clicked = None;
    for (i, item) in ITEMS.iter().enumerate() {
        draw_row(&p, card, i, item, thumb, selected == Some(i));

        let resp = ui.interact(
            row_rect(card, i),
            ui.scope_id().with(("clip-row", i)),
            egui::Sense::click(),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

fn row_rect(card: Rect, index: usize) -> Rect {
    let top = if index == 0 {
        89.0
    } else {
        179.0 + (index as f32 - 1.0) * ROW_H
    };
    let height = if index == 0 { ROW0_H } else { ROW_H };
    rr(card, ROW_INSET, top, ROW_W, height)
}

fn draw_toolbar(p: &Painter, card: Rect) {
    let cy = 43.0;

    draw_grid(p, card.min + vec2(44.0, cy));

    draw_letter_t(p, card.min + vec2(108.0, cy), NAVY);

    draw_image_icon(p, card.min + vec2(177.0, cy), NAVY);
    draw_file_icon(p, card.min + vec2(243.0, cy), NAVY);
    draw_link(p, card.min + vec2(311.0, cy), NAVY, CARD_BG);

    p.line_segment(
        [
            card.min + vec2(338.0, 24.0),
            card.min + vec2(338.0, 62.0),
        ],
        Stroke::new(1.5, Color32::from_rgb(0xE8, 0xE8, 0xF4)),
    );

    // 搜索胶囊
    let search = rr(card, 359.0, 18.0, 265.0, 58.0);
    p.rect_filled(search, CornerRadius::same(29), SEARCH_BG);
    icon_bold(
        p,
        pos2(search.left() + 24.0, search.center().y),
        21.0,
        '\u{E721}',
        Color32::from_rgb(0x5E, 0x64, 0x80),
    );
    p.text(
        pos2(search.left() + 53.0, search.center().y),
        Align2::LEFT_CENTER,
        "搜索剪贴板...",
        FontId::new(17.0, FontFamily::Proportional),
        PLACEHOLDER,
    );

    icon_bold(p, card.min + vec2(681.0, cy), 26.0, '\u{E840}', NAVY);
    icon(p, card.min + vec2(737.0, cy), 21.0, '\u{E711}', NAVY);

    // 选中分类下划线
    p.rect_filled(
        rr(card, 29.0, 76.0, 30.0, 3.0),
        CornerRadius::same(2),
        ACCENT,
    );
}

fn draw_row(
    p: &Painter,
    card: Rect,
    index: usize,
    item: &Item,
    thumb: Option<&TextureHandle>,
    selected: bool,
) {
    let top = if index == 0 {
        89.0
    } else {
        179.0 + (index as f32 - 1.0) * ROW_H
    };
    let height = if index == 0 { ROW0_H } else { ROW_H };

    if selected {
        p.rect_filled(
            rr(card, ROW_INSET, top, ROW_W, height),
            CornerRadius::same(14),
            SELECT,
        );
    }

    let tile_y = top + 15.0;
    let tile = rr(card, 30.0, tile_y, TILE, TILE);
    let tile_center = tile.center();

    match item.kind {
        Kind::Image => {
            if let Some(tex) = thumb {
                p.image(
                    tex.id(),
                    tile,
                    Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                p.rect_filled(tile, CornerRadius::same(15), TILE_NORMAL);
            }
        }
        kind => {
            let bg = match kind {
                Kind::Pdf => TILE_PDF,
                _ if selected => TILE_SELECTED,
                _ => TILE_NORMAL,
            };
            p.rect_filled(tile, CornerRadius::same(15), bg);

            match kind {
                Kind::Text | Kind::Rich => {
                    draw_doc(p, tile_center, 33.0, 36.0, 3.0, NAVY);
                }
                Kind::Link => draw_link(p, tile_center, NAVY, bg),
                Kind::Pdf => draw_pdf(p, tile_center, RED),
                Kind::Image => {}
            }
        }
    }

    let title_top = top + if selected { 23.0 } else { 22.0 };
    p.text(
        card.min + vec2(110.0, title_top + 10.0),
        Align2::LEFT_CENTER,
        item.title,
        FontId::new(20.6, FontFamily::Proportional),
        INK,
    );

    let meta_top = top + if selected { 54.0 } else { 53.0 };
    p.text(
        card.min + vec2(110.0, meta_top + 8.0),
        Align2::LEFT_CENTER,
        item.meta,
        FontId::new(17.5, FontFamily::Proportional),
        MUTED,
    );

    // 快捷键胶囊
    let center_y = top + if selected { 47.0 } else { 45.0 };
    let pill = Rect::from_min_size(
        card.min + vec2(638.0, center_y - 21.0),
        vec2(110.0, 42.0),
    );
    p.rect_filled(
        pill,
        CornerRadius::same(11),
        if selected { PILL_SELECTED } else { PILL },
    );
    p.text(
        pill.center(),
        Align2::CENTER_CENTER,
        item.key,
        FontId::new(20.5, FontFamily::Proportional),
        INK,
    );
}

// ---------------------------------------------------------------- 预览卡

fn preview_title(item: &Item) -> [&'static str; 2] {
    match item.kind {
        Kind::Text => ["这是一个示例文本，", "展示剪贴板工具的效果..."],
        _ => [item.title, ""],
    }
}

fn preview_body(item: &Item) -> [&'static str; 2] {
    match item.kind {
        Kind::Text => ["这是一个示例文本，", "展示剪贴板工具的效果。"],
        Kind::Image => ["风景图片.png", "1920 × 1080 · 2.4 MB"],
        Kind::Link => ["https://www.rust-lang.org/", "Rust 编程语言官网"],
        Kind::Pdf => ["design_spec.pdf", "PDF 文档 · 2.5 MB"],
        Kind::Rich => ["使用 Rust 开发高效的工具", "Rust 工具链与生态"],
    }
}

fn draw_preview(
    p: &Painter,
    card: Rect,
    item: &Item,
    arrow_y: f32,
    thumb: Option<&TextureHandle>,
) {
    // 指向选中行的箭头
    let tip_y = arrow_y.clamp(card.top() + 12.0, card.bottom() - 12.0);
    p.add(Shape::convex_polygon(
        vec![
            pos2(card.left() - 18.0, tip_y),
            pos2(card.left() + 1.0, tip_y - 11.0),
            pos2(card.left() + 1.0, tip_y + 11.0),
        ],
        PREVIEW_BG,
        Stroke::NONE,
    ));

    draw_card(p, card, PREVIEW_R, PREVIEW_BG);

    let header_tile = rr(card, 28.0, 26.0, 76.0, 80.0);
    p.rect_filled(header_tile, CornerRadius::same(18), TILE_HEADER);
    match item.kind {
        Kind::Image => {
            if let Some(tex) = thumb {
                p.image(
                    tex.id(),
                    header_tile,
                    Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                draw_doc(p, header_tile.center(), 35.0, 40.0, 3.0, Color32::from_rgb(0x3D, 0x40, 0x67));
            }
        }
        Kind::Link => draw_link(p, header_tile.center(), NAVY, TILE_HEADER),
        Kind::Pdf => draw_pdf(p, header_tile.center(), RED),
        _ => draw_doc(p, header_tile.center(), 35.0, 40.0, 3.0, Color32::from_rgb(0x3D, 0x40, 0x67)),
    }

    let title_x = 130.0;
    let title_lines = preview_title(item);
    for (i, line) in title_lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        p.text(
            card.min + vec2(title_x, 48.0 + i as f32 * 28.0),
            Align2::LEFT_CENTER,
            *line,
            FontId::new(21.5, FontFamily::Proportional),
            INK,
        );
    }

    p.line_segment(
        [
            card.min + vec2(14.0, 121.0),
            card.min + vec2(PREVIEW_W - 14.0, 121.0),
        ],
        Stroke::new(1.2, Color32::from_rgb(0xE8, 0xE9, 0xF4)),
    );

    for (i, line) in preview_body(item).iter().enumerate() {
        p.text(
            card.min + vec2(30.0, 157.0 + i as f32 * 28.0),
            Align2::LEFT_CENTER,
            *line,
            FontId::new(21.0, FontFamily::Proportional),
            BODY,
        );
    }

    p.line_segment(
        [
            card.min + vec2(0.0, 246.0),
            card.min + vec2(PREVIEW_W, 246.0),
        ],
        Stroke::new(1.2, DIVIDER),
    );

    // 底部操作栏
    let button = rr(card, 29.0, 268.0, 111.0, 50.0);
    p.rect_filled(button, CornerRadius::same(12), BUTTON);
    draw_copy_icon(
        p,
        pos2(button.left() + 28.5, button.center().y),
        1.0,
        Color32::WHITE,
        BUTTON,
    );
    p.text(
        pos2(button.left() + 54.0, button.center().y),
        Align2::LEFT_CENTER,
        "复制",
        FontId::new(18.0, FontFamily::Proportional),
        Color32::WHITE,
    );

    for x in [162.0, 288.0] {
        p.line_segment(
            [
                card.min + vec2(x, 272.0),
                card.min + vec2(x, 314.0),
            ],
            Stroke::new(1.2, DIVIDER),
        );
    }

    icon_bold(
        p,
        card.min + vec2(200.0, 293.0),
        26.0,
        '\u{E840}',
        Color32::from_rgb(0x2C, 0x32, 0x52),
    );
    p.text(
        card.min + vec2(225.0, 293.0),
        Align2::LEFT_CENTER,
        "置顶",
        FontId::new(18.5, FontFamily::Proportional),
        Color32::from_rgb(0x1C, 0x21, 0x38),
    );

    icon_bold(
        p,
        card.min + vec2(327.0, 293.0),
        26.0,
        '\u{E74D}',
        Color32::from_rgb(0x2C, 0x32, 0x52),
    );
    p.text(
        card.min + vec2(352.0, 293.0),
        Align2::LEFT_CENTER,
        "删除",
        FontId::new(18.5, FontFamily::Proportional),
        Color32::from_rgb(0x1C, 0x21, 0x38),
    );
}

// ---------------------------------------------------------------- 自绘图标

fn draw_grid(p: &Painter, center: Pos2) {
    let s = 13.0;
    let gap = 4.0;
    let off = (s + gap) * 0.5;
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        p.rect_filled(
            Rect::from_center_size(center + vec2(dx * off, dy * off), Vec2::splat(s)),
            CornerRadius::same(4),
            ACCENT,
        );
    }
}

fn draw_letter_t(p: &Painter, c: Pos2, color: Color32) {
    let w = 28.0;
    let bar_h = 4.0;
    let top = c.y - 15.0;
    p.rect_filled(
        Rect::from_min_size(pos2(c.x - w * 0.5, top), vec2(w, bar_h)),
        CornerRadius::same(1),
        color,
    );
    p.rect_filled(
        Rect::from_min_size(pos2(c.x - 2.0, top), vec2(4.0, 28.0)),
        CornerRadius::same(1),
        color,
    );
    p.rect_filled(
        Rect::from_min_size(pos2(c.x - 7.0, top + 26.0), vec2(14.0, 4.0)),
        CornerRadius::same(1),
        color,
    );
    for x in [c.x - w * 0.5, c.x + w * 0.5 - 2.5] {
        p.rect_filled(
            Rect::from_min_size(pos2(x, top), vec2(2.5, 5.0)),
            CornerRadius::same(1),
            color,
        );
    }
}

fn draw_doc(p: &Painter, c: Pos2, w: f32, h: f32, stroke: f32, color: Color32) {
    let rect = Rect::from_center_size(c, vec2(w, h));
    p.rect_stroke(
        rect,
        CornerRadius::same((w * 0.18) as u8),
        Stroke::new(stroke, color),
        StrokeKind::Inside,
    );
    let lx = rect.left() + w * 0.26;
    let full = w * 0.56;
    for (i, frac) in [0.32, 0.52, 0.72].iter().enumerate() {
        let lw = if i == 2 { full * 0.62 } else { full };
        p.rect_filled(
            Rect::from_min_size(pos2(lx, rect.top() + h * frac - 1.3), vec2(lw, 2.6)),
            CornerRadius::same(1),
            color,
        );
    }
}

fn capsule_points(center: Pos2, half_len: f32, half_w: f32, angle: f32) -> Vec<Pos2> {
    let (sin, cos) = angle.sin_cos();
    let axis = vec2(cos, sin);
    let normal = vec2(-sin, cos);
    let end = half_len - half_w;
    let mut pts = Vec::with_capacity(28);
    for i in 0..=12 {
        let a = -std::f32::consts::FRAC_PI_2 + i as f32 / 12.0 * std::f32::consts::PI;
        pts.push(center + axis * (end + half_w * a.cos()) + normal * (half_w * a.sin()));
    }
    for i in 0..=12 {
        let a = std::f32::consts::FRAC_PI_2 + i as f32 / 12.0 * std::f32::consts::PI;
        pts.push(center + axis * (-end + half_w * a.cos()) + normal * (half_w * a.sin()));
    }
    pts
}

fn draw_link(p: &Painter, c: Pos2, color: Color32, bg: Color32) {
    let stroke = Stroke::new(2.4, color);
    let angle = -45f32.to_radians();
    let axis = vec2(angle.cos(), angle.sin());
    let normal = vec2(-axis.y, axis.x);
    let lower = capsule_points(c - axis * 4.0 - normal * 4.0, 9.5, 3.4, angle);
    let upper = capsule_points(c + axis * 4.0 + normal * 4.0, 9.5, 3.4, angle);
    p.add(Shape::closed_line(lower, stroke));
    p.add(Shape::convex_polygon(upper.clone(), bg, Stroke::NONE));
    p.add(Shape::closed_line(upper, stroke));
}

fn folded_doc_points(r: Rect, fold: f32, rad: f32) -> Vec<Pos2> {
    vec![
        pos2(r.left() + rad, r.top()),
        pos2(r.right() - fold, r.top()),
        pos2(r.right(), r.top() + fold),
        pos2(r.right(), r.bottom() - rad),
        pos2(r.right() - rad, r.bottom()),
        pos2(r.left() + rad, r.bottom()),
        pos2(r.left(), r.bottom() - rad),
        pos2(r.left(), r.top() + rad),
    ]
}

fn draw_file_icon(p: &Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, vec2(25.0, 30.0));
    let fold = 8.5;
    p.add(Shape::closed_line(
        folded_doc_points(r, fold, 3.5),
        Stroke::new(2.6, color),
    ));
    p.line_segment(
        [
            pos2(r.right() - fold, r.top()),
            pos2(r.right() - fold, r.top() + fold),
        ],
        Stroke::new(2.2, color),
    );
    p.line_segment(
        [
            pos2(r.right() - fold, r.top() + fold),
            pos2(r.right(), r.top() + fold),
        ],
        Stroke::new(2.2, color),
    );
}

fn draw_image_icon(p: &Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, vec2(28.0, 28.0));
    p.rect_stroke(
        r,
        CornerRadius::same(7),
        Stroke::new(2.6, color),
        StrokeKind::Inside,
    );
    p.circle_stroke(
        pos2(r.right() - 8.0, r.top() + 8.0),
        2.1,
        Stroke::new(2.0, color),
    );
    let mountain = vec![
        pos2(r.left() + 4.5, r.bottom() - 6.5),
        pos2(r.left() + 10.5, r.top() + 12.5),
        pos2(r.left() + 15.5, r.bottom() - 8.5),
        pos2(r.left() + 18.5, r.bottom() - 11.5),
        pos2(r.right() - 4.5, r.bottom() - 6.5),
    ];
    p.add(Shape::line(mountain, Stroke::new(2.2, color)));
}

fn draw_copy_icon(p: &Painter, c: Pos2, scale: f32, color: Color32, bg: Color32) {
    let size = vec2(14.5, 17.5) * scale;
    let back = Rect::from_center_size(c + vec2(3.0, -2.6) * scale, size);
    let front = Rect::from_center_size(c + vec2(-2.0, 2.0) * scale, size);
    p.rect_stroke(
        back,
        CornerRadius::same(3),
        Stroke::new(2.0 * scale, color),
        StrokeKind::Inside,
    );
    p.rect_filled(front, CornerRadius::same(3), bg);
    p.rect_stroke(
        front,
        CornerRadius::same(3),
        Stroke::new(2.0 * scale, color),
        StrokeKind::Inside,
    );
}

fn draw_pdf(p: &Painter, c: Pos2, color: Color32) {
    let r = Rect::from_center_size(c, vec2(30.0, 34.0));
    let fold = 9.0;
    let rad = 3.0;
    let pts = vec![
        pos2(r.left() + rad, r.top()),
        pos2(r.right() - fold, r.top()),
        pos2(r.right(), r.top() + fold),
        pos2(r.right(), r.bottom() - rad),
        pos2(r.right() - rad, r.bottom()),
        pos2(r.left() + rad, r.bottom()),
        pos2(r.left(), r.bottom() - rad),
        pos2(r.left(), r.top() + rad),
    ];
    p.add(Shape::closed_line(pts, Stroke::new(2.4, color)));
    p.line_segment(
        [
            pos2(r.right() - fold, r.top()),
            pos2(r.right() - fold, r.top() + fold),
        ],
        Stroke::new(2.0, color),
    );
    p.line_segment(
        [
            pos2(r.right() - fold, r.top() + fold),
            pos2(r.right(), r.top() + fold),
        ],
        Stroke::new(2.0, color),
    );
    p.text(
        r.center() + vec2(0.0, 1.0),
        Align2::CENTER_CENTER,
        "PDF",
        FontId::new(8.0, bold_family()),
        color,
    );
}
