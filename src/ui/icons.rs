//! 图标：gpui-kit 自带 Lucide Icon + 简单几何自绘。

use std::borrow::Cow;

use gpui_kit::assets::{Assets as ComponentAssets, IconName, icon_assets};
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::{Tup, UI_SCALE, c};

icon_assets!(
    ExtraIcons,
    [Copy, Link, Type, Trash, Image, Pin, LayoutGrid]
);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        ComponentAssets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = ComponentAssets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

pub fn icon(name: IconName, size: f32) -> impl IntoElement {
    Icon::new(name).size(px(size * UI_SCALE))
}

pub fn icon_colored(name: IconName, size: f32, color: Tup) -> impl IntoElement {
    Icon::new(name)
        .size(px(size * UI_SCALE))
        .text_color(c(color))
}
