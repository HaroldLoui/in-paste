//! 系统字体加载。
//!
//! egui 默认不带 CJK 字形，这里把 Segoe UI + 微软雅黑装进 Proportional，
//! 再单独注册 `bold` 与 `icons`（Segoe Fluent Icons）两个字体族。
//! 任何一款字体缺失都只是回退到默认字体，不影响启动。

use std::sync::Arc;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

pub fn install(ctx: &Context) {
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
        if let Ok(bytes) = std::fs::read(path) {
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

pub fn icons_family() -> FontFamily {
    FontFamily::Name(Arc::from("icons"))
}

pub fn bold_family() -> FontFamily {
    FontFamily::Name(Arc::from("bold"))
}
