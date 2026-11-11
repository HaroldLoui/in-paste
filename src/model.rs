//! 列表数据与预览文案。
//!
//! 本期仍用固定的 mock 数据，但已经改成运行时 `Vec<Item>`，
//! 以便支持置顶、删除和按分类过滤；后续接真实剪贴板时替换 `initial_items` 即可。

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Image,
    Link,
    Pdf,
    Rich,
}

impl Kind {
    /// 用于搜索匹配的类型名。
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "文本",
            Self::Image => "图片",
            Self::Link => "链接",
            Self::Pdf => "PDF",
            Self::Rich => "富文本",
        }
    }

    /// 该类型是否属于第 `tab` 个分类（0 = 全部）。
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
}

#[derive(Clone, Copy)]
pub struct Item {
    pub kind: Kind,
    pub title: &'static str,
    pub meta: &'static str,
    pub pinned: bool,
}

impl Item {
    /// 是否命中搜索词（`query` 已转小写并去除首尾空白）。
    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let body = self.preview_body();
        self.title.to_lowercase().contains(query)
            || self.kind.label().to_lowercase().contains(query)
            || body.iter().any(|line| line.to_lowercase().contains(query))
    }

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

    fn new(kind: Kind, title: &'static str, meta: &'static str) -> Self {
        Self {
            kind,
            title,
            meta,
            pinned: false,
        }
    }
}

/// 初始 mock 数据（Ctrl + N 由列表位置动态生成）。
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
            "https://docs.rs/eframe/latest/eframe/",
            "12 分钟前",
        ),
        Item::new(Kind::Rich, "本周迭代范围：DPI 修复与窗口布局", "18 分钟前"),
    ]
}
