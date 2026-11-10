//! 应用外壳：加载资源、响应点击、切换主卡与预览卡。

use eframe::egui::{Color32, Rect, TextureHandle, TextureOptions, ViewportCommand, vec2};

use crate::model::ITEMS;
use crate::theme::{
    BASE_H, CARD_BG, CARD_H, CARD_R, CARD_W, CARD_X, CARD_Y, FULL_W, PREVIEW_H, PREVIEW_W,
    PREVIEW_X,
};
use crate::{fonts, ui, window};

pub struct UiApp {
    thumb: Option<TextureHandle>,
    selected: Option<usize>,
}

impl UiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::install(&cc.egui_ctx);
        Self {
            thumb: load_thumb(&cc.egui_ctx),
            selected: None,
        }
    }
}

impl eframe::App for UiApp {
    fn clear_color(&self, _visuals: &eframe::egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        window::strip_system_frame(frame);

        let full = ui.max_rect();
        let origin = full.min;

        let card = Rect::from_min_size(origin + vec2(CARD_X, CARD_Y), vec2(CARD_W, CARD_H));
        let p = ui.painter().clone();
        ui::draw_card(&p, card, CARD_R, CARD_BG);
        ui::toolbar::draw(&p, card);

        if let Some(i) = ui::list::draw(ui, card, self.thumb.as_ref(), self.selected) {
            if self.selected != Some(i) {
                self.selected = Some(i);
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::InnerSize(vec2(FULL_W, BASE_H)));
            }
        }

        // 窗口被系统或用户改窄后，保持展开态。
        if self.selected.is_some() && (full.width() - FULL_W).abs() > 1.0 {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::InnerSize(vec2(FULL_W, BASE_H)));
        }

        if let Some(i) = self.selected {
            let arrow_y = ui::row::rect(card, i).center().y;
            let top = (arrow_y - 70.0).clamp(CARD_Y, BASE_H - PREVIEW_H - 40.0);
            let preview =
                Rect::from_min_size(origin + vec2(PREVIEW_X, top), vec2(PREVIEW_W, PREVIEW_H));
            ui::preview::draw(
                ui.painter(),
                preview,
                &ITEMS[i],
                arrow_y,
                self.thumb.as_ref(),
            );
        }
    }
}

fn load_thumb(ctx: &eframe::egui::Context) -> Option<TextureHandle> {
    let bytes = include_bytes!("../assets/thumb.png");
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels: Vec<Color32> = image
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let color = eframe::egui::ColorImage::new(size, pixels);
    Some(ctx.load_texture("thumb", color, TextureOptions::LINEAR))
}
