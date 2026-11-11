//! 列表数据与预览文案。
//!
//! 本期是 UI 原型，数据固定；后续接真实剪贴板时只需替换 `ITEMS` 的来源。

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Image,
    Link,
    Pdf,
    Rich,
}

pub struct Item {
    pub kind: Kind,
    pub title: &'static str,
    pub meta: &'static str,
    pub key: &'static str,
}

impl Item {
    /// 预览卡头部的两行标题，第二行可为空。
    pub fn preview_title(&self) -> [&'static str; 2] {
        match self.kind {
            Kind::Text => ["这是一个示例文本，", "展示剪贴板工具的效果..."],
            _ => [self.title, ""],
        }
    }

    /// 预览卡正文的两行内容。
    pub fn preview_body(&self) -> [&'static str; 2] {
        match self.kind {
            Kind::Text => ["这是一个示例文本，", "展示剪贴板工具的效果。"],
            Kind::Image => ["风景图片.png", "1920 × 1080 · 2.4 MB"],
            Kind::Link => [self.title, ""],
            Kind::Pdf => [self.title, "PDF 文档 · 2.5 MB"],
            Kind::Rich => [self.title, ""],
        }
    }
}

pub const ITEMS: [Item; 8] = [
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
    Item {
        kind: Kind::Text,
        title: "cargo fmt --all && cargo clippy --all-targets",
        meta: "8 分钟前",
        key: "Ctrl + 6",
    },
    Item {
        kind: Kind::Link,
        title: "https://docs.rs/eframe/latest/eframe/",
        meta: "12 分钟前",
        key: "Ctrl + 7",
    },
    Item {
        kind: Kind::Rich,
        title: "本周迭代范围：DPI 修复与窗口布局",
        meta: "18 分钟前",
        key: "Ctrl + 8",
    },
];
