//! mock 数据与过滤 —— 从 in-paste/src/model.rs 迁移。
#![allow(dead_code)]

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Image,
    Link,
    Pdf,
    Rich,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "文本",
            Self::Image => "图片",
            Self::Link => "链接",
            Self::Pdf => "PDF",
            Self::Rich => "富文本",
        }
    }

    pub fn matches_tab(self, tab: usize) -> bool {
        match tab {
            0 => true,
            1 => matches!(self, Kind::Text | Kind::Rich),
            2 => self == Kind::Image,
            3 => self == Kind::Pdf,
            4 => self == Kind::Link,
            _ => true,
        }
    }

    pub fn tile_color(self) -> crate::theme::Tup {
        match self {
            Kind::Pdf => crate::theme::TILE_PDF,
            _ => crate::theme::TILE_NORMAL,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Item {
    pub kind: Kind,
    pub title: &'static str,
    pub meta: &'static str,
    pub pinned: bool,
}

impl Item {
    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        self.title.to_lowercase().contains(query)
            || self.kind.label().to_lowercase().contains(query)
            || self.meta.to_lowercase().contains(query)
    }

    fn new(kind: Kind, title: &'static str, meta: &'static str) -> Self {
        Self {
            kind,
            title,
            meta,
            pinned: false,
        }
    }
}

pub fn initial_items() -> Vec<Item> {
    vec![
        Item::new(
            Kind::Text,
            "这是一个示例文本，展示剪贴板工具的效果...",
            "刚刚",
        ),
        Item::new(Kind::Image, "风景图片.png", "1 分钟前"),
        Item::new(Kind::Link, "https://www.rust-lang.org/", "2 分钟前"),
        Item::new(Kind::Pdf, "design_spec.pdf", "3 分钟前 · 2.5 MB"),
        Item::new(Kind::Rich, "使用 Rust 开发高效的工具...", "5 分钟前"),
        Item::new(
            Kind::Text,
            "cargo fmt --all && cargo clippy --all-targets",
            "8 分钟前",
        ),
        Item::new(
            Kind::Link,
            "https://docs.rs/gpui/latest/gpui/",
            "12 分钟前",
        ),
        Item::new(Kind::Rich, "本周迭代范围：窗口层 spike 验收", "18 分钟前"),
    ]
}
