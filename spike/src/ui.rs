//! 自绘 UI：卡片 / 工具栏 / 列表行 / 搜索胶囊。
//! 非标件全部自己画，标准控件（设置页等）后续再接 gpui-component。

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Icon, IconName, Sizable};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::model::{initial_items, Item, Kind};
use crate::platform::win;
use crate::theme::*;

const TABS: [&str; 5] = ["全部", "文本", "图片", "PDF", "链接"];

pub struct SpikeApp {
    items: Vec<Item>,
    visible: Vec<usize>,
    selected: usize,
    tab: usize,
    search: Entity<InputState>,
    topmost: bool,
    cloaked: bool,
    hardened: bool,
    prev_foreground: Option<isize>,
    _search_sub: Subscription,
}

impl SpikeApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("搜索剪贴板内容…")
                .clean_on_escape()
        });

        let search_sub = cx.subscribe_in(&search, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                let q = state.read(cx).value().to_string();
                this.refilter(&q, cx);
            }
        });

        let items = initial_items();
        let visible: Vec<usize> = (0..items.len()).collect();

        Self {
            items,
            visible,
            selected: 0,
            tab: 0,
            search,
            topmost: false,
            cloaked: false,
            hardened: false,
            prev_foreground: None,
            _search_sub: search_sub,
        }
    }

    fn refilter(&mut self, query: &str, cx: &mut Context<Self>) {
        let q = query.trim().to_lowercase();
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| it.kind.matches_tab(self.tab) && it.matches_query(&q))
            .map(|(i, _)| i)
            .collect();
        if self.selected >= self.visible.len() {
            self.selected = self.visible.len().saturating_sub(1);
        }
        cx.notify();
    }

    fn move_sel(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.visible.is_empty() {
            return;
        }
        let len = self.visible.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        cx.notify();
    }

    fn ensure_harden(&mut self, window: &Window) {
        if self.hardened {
            return;
        }
        if win::harden(window).is_some() {
            self.hardened = true;
            eprintln!("[spike] hwnd hardened (toolwindow / no caption / clear title)");
        }
    }

    fn toggle_topmost(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(hwnd) = win::hwnd_of(window) else {
            return;
        };
        self.topmost = !self.topmost;
        win::set_topmost(hwnd, self.topmost);
        eprintln!("[spike] topmost = {}", self.topmost);
        cx.notify();
    }

    fn toggle_cloak(&mut self, window: &Window, cx: &mut Context<Self>) {
        let Some(hwnd) = win::hwnd_of(window) else {
            return;
        };
        self.cloaked = !self.cloaked;
        if self.cloaked {
            self.prev_foreground = Some(win::foreground());
            win::set_cloaked(hwnd, true);
            eprintln!("[spike] cloaked = true");
        } else {
            win::set_cloaked(hwnd, false);
            if let Some(prev) = self.prev_foreground.take()
                && win::is_window(prev)
            {
                win::set_foreground(prev);
            }
            eprintln!("[spike] cloaked = false");
        }
        cx.notify();
    }

    /// ↑/↓ 导航（焦点在窗口任意处时通过 key binding 触发，见 main 绑定）。
    pub fn on_move(&mut self, delta: isize, cx: &mut Context<Self>) {
        self.move_sel(delta, cx);
    }
}

impl Render for SpikeApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_harden(window);

        let win_w = window.viewport_size().width;
        let win_h = window.viewport_size().height;
        let card_w = (win_w - px(2.0 * MARGIN_X)).max(px(MIN_CARD_W));
        let card_h = (win_h - px(MARGIN_TOP + MARGIN_BOTTOM)).max(px(MIN_CARD_H));

        div()
            .id("spike-root")
            .size_full()
            .flex()
            .flex_col()
            .pt(px(MARGIN_TOP))
            .pb(px(MARGIN_BOTTOM))
            .px(px(MARGIN_X))
            .child(self.render_card(window, cx, card_w, card_h))
    }
}

impl SpikeApp {
    fn render_card(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        card_w: Pixels,
        card_h: Pixels,
    ) -> impl IntoElement {
        let selected_idx = self.selected;
        let items_snapshot: Vec<Item> = self.visible.iter().map(|&i| self.items[i]).collect();
        let tab = self.tab;
        let topmost = self.topmost;
        let cloaked = self.cloaked;
        let search = self.search.clone();

        div()
            .id("card")
            .w(card_w)
            .h(card_h)
            .flex()
            .flex_col()
            .rounded(px(CARD_R))
            .bg(c(CARD_BG))
            .overflow_hidden()
            .child(render_toolbar(
                cx,
                &search,
                tab,
                topmost,
                cloaked,
                |this, window, cx| this.toggle_topmost(window, cx),
                |this, window, cx| this.toggle_cloak(window, cx),
                window,
            ))
            .child(render_list(&items_snapshot, selected_idx))
    }
}

#[allow(clippy::too_many_arguments)]
fn render_toolbar(
    cx: &mut Context<SpikeApp>,
    search: &Entity<InputState>,
    tab: usize,
    topmost: bool,
    cloaked: bool,
    on_topmost: impl Fn(&mut SpikeApp, &mut Window, &mut Context<SpikeApp>) + 'static,
    on_cloak: impl Fn(&mut SpikeApp, &mut Window, &mut Context<SpikeApp>) + 'static,
    _window: &mut Window,
) -> impl IntoElement {
    let _ = cx;
    let topmost_cb = on_topmost;
    let cloak_cb = on_cloak;

    div()
        .id("toolbar")
        .w_full()
        .flex()
        .flex_col()
        .pt(px(18.))
        .px(px(20.))
        .gap(px(12.))
        .child(
            div()
                .id("toolbar-row1")
                .w_full()
                .h(px(58.))
                .flex()
                .items_center()
                .gap(px(12.))
                .child(
                    div()
                        .id("search-capsule")
                        .flex_1()
                        .h(px(58.))
                        .flex()
                        .items_center()
                        .px(px(20.))
                        .rounded(px(29.))
                        .bg(c(SEARCH_BG))
                        .child(
                            Input::new(search)
                                .appearance(false)
                                .prefix(Icon::new(IconName::Search).small()),
                        ),
                )
                .child(
                    div()
                        .id("win-btns")
                        .flex()
                        .gap(px(8.))
                        .child(win_btn(
                            "topmost",
                            if topmost { "置顶:开" } else { "置顶:关" },
                            topmost,
                            cx.listener(move |this, _, window, cx| topmost_cb(this, window, cx)),
                        ))
                        .child(win_btn(
                            "cloak",
                            if cloaked { "隐藏" } else { "显示" },
                            false,
                            cx.listener(move |this, _, window, cx| cloak_cb(this, window, cx)),
                        )),
                ),
        )
        .child(
            div()
                .id("tabs")
                .w_full()
                .h(px(40.))
                .flex()
                .items_end()
                .gap(px(4.))
                .children(TABS.iter().enumerate().map(|(i, label)| {
                    let active = i == tab;
                    let mut d = div()
                        .id(SharedString::from(format!("tab-{i}")))
                        .px(px(14.))
                        .pb(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(6.));
                    if active {
                        d = d
                            .border_b_2()
                            .border_color(c(ACCENT));
                    }
                    d.child(
                        div()
                            .size(px(18.))
                            .rounded(px(4.))
                            .bg(if active { c(ACCENT) } else { c(PILL) }),
                    )
                    .child(
                        div()
                            .child(*label)
                            .text_size(px(15.))
                            .text_color(if active { c(INK) } else { c(MUTED) }),
                    )
                    .into_any_element()
                })),
        )
}

fn win_btn(
    id: &str,
    label: impl Into<SharedString>,
    active: bool,
    on_click: impl Fn(&(), &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(SharedString::from(format!("btn-{id}")))
        .h(px(36.))
        .px(px(14.))
        .flex()
        .items_center()
        .rounded(px(10.))
        .bg(if active { c(ACCENT) } else { c(PILL) })
        .text_size(px(13.))
        .text_color(if active {
            rgb8(255, 255, 255)
        } else {
            c(NAVY)
        })
        .cursor_pointer()
        .hover(|s| s.bg(c(TILE_SELECTED)))
        .on_click(move |_, window, cx| on_click(&(), window, cx))
        .child(label.into())
}

fn render_list(items: &[Item], selected: usize) -> impl IntoElement {
    let rows = items.iter().enumerate().map(|(i, item)| {
        render_row(i, item, i == selected).into_any_element()
    });

    div()
        .id("list")
        .flex_1()
        .min_h_0()
        .w_full()
        .flex()
        .flex_col()
        .pt(px(10.))
        .px(px(ROW_INSET))
        .overflow_hidden()
        .children(rows)
}

fn render_row(idx: usize, item: &Item, selected: bool) -> impl IntoElement {
    let kind_label = item.kind.label();
    let tile = if selected { TILE_SELECTED } else { item.kind.tile_color() };

    let mut row = div()
        .id(SharedString::from(format!("row-{idx}")))
        .w_full()
        .h(px(if idx == 0 { ROW0_H - 9. } else { ROW_H - 1. }))
        .flex()
        .items_center()
        .gap(px(14.))
        .px(px(12.))
        .rounded(px(14.));
    if selected {
        row = row.bg(c(SELECT));
    }
    row = row.hover(|s| s.bg(c(SELECT)));

    row.child(
        div()
            .id(SharedString::from(format!("tile-{idx}")))
            .size(px(TILE))
            .rounded(px(14.))
            .bg(c(tile))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_size(px(22.))
                    .text_color(c(NAVY))
                    .child(match item.kind {
                        Kind::Text | Kind::Rich => "T",
                        Kind::Image => "I",
                        Kind::Link => "L",
                        Kind::Pdf => "P",
                    }),
            ),
    )
    .child(
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .w_full()
                    .truncate()
                    .text_size(px(17.))
                    .text_color(c(INK))
                    .child(item.title),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .text_size(px(13.))
                    .text_color(c(MUTED))
                    .child(kind_label)
                    .child("·")
                    .child(item.meta),
            ),
    )
    .when(selected, |d| {
        d.child(
            div()
                .flex()
                .gap(px(6.))
                .child(hover_icon("copy", "C"))
                .child(hover_icon("pin", "P"))
                .child(hover_icon("del", "D")),
        )
    })
}

fn hover_icon(id: &str, glyph: &'static str) -> impl IntoElement {
    div()
        .id(SharedString::from(format!("icon-{id}")))
        .size(px(HOVER_BTN))
        .rounded(px(8.))
        .bg(ca((255, 255, 255), 160))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(14.))
        .text_color(c(NAVY))
        .hover(|s| s.bg(rgb8(255, 255, 255)))
        .cursor_pointer()
        .child(glyph)
}
