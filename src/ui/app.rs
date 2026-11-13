//! 应用状态机（UI 阶段）：视图切换、选中/过滤、滚动、键盘导航。
//!
//! 为方便调试，**保留托盘与窗口置顶**；其余业务（剪贴板写入、cloak 显隐、
//! 失焦隐藏、全局热键、行内增删改）暂不接入，等 UI 打磨完再接。

use std::time::Duration;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use crate::core::{self, Item};
use crate::platform::{self, Tray, TrayAction, windows::win};
use crate::theme::*;
use crate::ui::{drag_zone, preview, row, settings, toolbar};

actions!(in_paste, [MoveUp, MoveDown, Escape, FocusSearch,]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    List,
    Settings,
}

pub struct UiApp {
    window: AnyWindowHandle,
    items: Vec<Item>,
    selected: Option<usize>,
    /// 当前悬停的可见行（用于 hover 显示行内按钮）。
    pub hovered: Option<usize>,
    tab: usize,
    view: View,
    scroll: f32,
    search: Entity<InputState>,
    card_w: f32,
    card_h: f32,
    /// 工具栏/设置页「置顶」开关；true 时调 win32 topmost（调试用）。
    pinned: bool,
    auto_hide: bool,
    _tray: Option<Tray>,
}

impl UiApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("搜索剪贴板...")
                .clean_on_escape()
        });

        cx.subscribe_in(&search, window, |this, _state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.selected = None;
                this.scroll = 0.0;
                cx.notify();
            }
        })
        .detach();

        let view = Self {
            window: window.window_handle(),
            items: core::initial_items(),
            selected: None,
            hovered: None,
            tab: 0,
            view: View::List,
            scroll: 0.0,
            search,
            card_w: DEFAULT_CARD_W,
            card_h: DEFAULT_CARD_H,
            pinned: false,
            auto_hide: true,
            _tray: platform::tray::install(),
        };

        // 窗口是权威尺寸；卡片跟随 viewport。预览卡暂不参与尺寸推导，
        // 后续会改成 popover，不在这里耦合。
        cx.observe_window_bounds(window, |this, window, cx| {
            let viewport = window.viewport_size();
            let next_w = f32::from(viewport.width).max(MIN_CARD_W);
            let next_h = f32::from(viewport.height).max(MIN_CARD_H);
            if (this.card_w - next_w).abs() > 1.0 || (this.card_h - next_h).abs() > 1.0 {
                this.card_w = next_w;
                this.card_h = next_h;
                cx.notify();
            }
        })
        .detach();

        view
    }

    fn expanded(&self) -> bool {
        self.view == View::List && self.selected.is_some()
    }

    fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().to_string()
    }

    fn visible(&self, query: &str) -> Vec<usize> {
        let query = query.trim().to_lowercase();
        let mut visible: Vec<usize> = (0..self.items.len())
            .filter(|&i| self.items[i].kind.matches_tab(self.tab))
            .filter(|&i| self.items[i].matches_query(&query))
            .collect();
        visible.sort_by_key(|&i| !self.items[i].pinned);
        visible
    }

    // ---------------------------------------------------------------- 动作（供 toolbar/row/settings 调用）

    pub fn set_tab(&mut self, tab: usize) {
        self.tab = tab;
        self.selected = None;
        self.scroll = 0.0;
    }

    /// 置顶开关：翻转 UI 状态并调 win32 topmost（调试期保留）。
    pub fn toggle_pin(&mut self, cx: &mut Context<Self>) {
        self.pinned = !self.pinned;
        let pinned = self.pinned;
        let handle = self.window;
        let _ = handle.update(cx, |_, window, _| {
            if let Some(hwnd) = win::hwnd_of(window) {
                win::set_topmost(hwnd, pinned);
            }
        });
    }

    pub fn open_settings(&mut self) {
        self.view = View::Settings;
        self.selected = None;
    }

    pub fn back_to_list(&mut self) {
        self.view = View::List;
    }

    pub fn toggle_auto_hide(&mut self) {
        self.auto_hide = !self.auto_hide;
    }

    pub fn exit(&mut self, cx: &mut Context<Self>) {
        let handle = self.window;
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }

    pub fn on_row_select(&mut self, pos: usize, cx: &mut Context<Self>) {
        self.selected = if self.selected == Some(pos) {
            None
        } else {
            Some(pos)
        };
        // 预览卡暂不参与窗口尺寸（后续会换成 popover），这里只刷新选中态。
        cx.notify();
    }

    /// 行内按钮：UI 阶段只做选中反馈，不写剪贴板、不改数据、不藏面板。
    pub fn on_row_action(&mut self, _real: usize, action: row::RowAction, cx: &mut Context<Self>) {
        match action {
            row::RowAction::Copy | row::RowAction::Pin | row::RowAction::Delete => {
                // 业务后接：复制到剪贴板 / 置顶排序 / 删除记录
                self.selected = None;
                cx.notify();
            }
        }
    }

    // ---------------------------------------------------------------- 键盘

    fn move_sel(&mut self, delta: isize, cx: &mut Context<Self>) {
        let q = self.query(cx);
        let visible = self.visible(&q);
        if visible.is_empty() {
            return;
        }
        let len = visible.len() as isize;
        let next = match (self.selected, delta > 0) {
            (Some(i), true) => (i as isize + 1).min(len - 1),
            (Some(i), false) => (i as isize - 1).max(0),
            (None, true) => 0,
            (None, false) => len - 1,
        };
        self.selected = Some(next as usize);
        self.ensure_visible(next as usize);
        cx.notify();
    }

    fn ensure_visible(&mut self, index: usize) {
        let top = if index == 0 {
            LIST_TOP
        } else {
            LIST_TOP + ROW0_H + (index as f32 - 1.0) * ROW_H
        };
        let h = if index == 0 { ROW0_H } else { ROW_H };
        let list_top = LIST_TOP;
        let list_bottom = self.card_h - LIST_BOTTOM_PAD;
        let vis_top = top - self.scroll;
        let vis_bottom = top + h - self.scroll;
        if vis_top < list_top {
            self.scroll -= list_top - vis_top;
        } else if vis_bottom > list_bottom {
            self.scroll += vis_bottom - list_bottom;
        }
        let total = ROW0_H + (self.items.len().saturating_sub(1)) as f32 * ROW_H;
        let max_scroll = (total - (list_bottom - list_top)).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max_scroll);
    }

    fn on_escape(&mut self, cx: &mut Context<Self>) {
        match self.view {
            View::Settings => {
                self.view = View::List;
                cx.notify();
            }
            View::List => {
                let q = self.query(cx);
                if !q.is_empty() {
                    let handle = self.window;
                    let search = self.search.clone();
                    let _ = handle.update(cx, |_, window, cx| {
                        search.update(cx, |s, cx| s.set_value("", window, cx));
                    });
                    self.selected = None;
                    self.scroll = 0.0;
                    cx.notify();
                } else if self.selected.take().is_some() {
                    cx.notify();
                }
                // 无查询且无选中：UI 阶段不隐藏面板（业务后接）
            }
        }
    }

    fn focus_search(&mut self, cx: &mut Context<Self>) {
        self.view = View::List;
        let handle = self.window;
        let search = self.search.clone();
        let _ = handle.update(cx, |_, window, cx| {
            search.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.notify();
    }

    // ---------------------------------------------------------------- 托盘轮询（调试期保留）

    /// 由 main 的定时任务调用：处理托盘菜单事件。
    /// 托盘「显示/隐藏」暂改为聚焦窗口（cloak 显隐属业务，后接）。
    pub fn poll(&mut self, cx: &mut Context<Self>) {
        let Some(action) = platform::tray::take_action() else {
            return;
        };
        match action {
            TrayAction::Toggle => {
                self.view = View::List;
                let handle = self.window;
                let search = self.search.clone();
                let _ = handle.update(cx, |_, window, cx| {
                    window.activate_window();
                    search.update(cx, |state, cx| state.focus(window, cx));
                });
                cx.notify();
            }
            TrayAction::Settings => {
                self.view = View::Settings;
                self.selected = None;
                cx.notify();
            }
            TrayAction::Quit => self.exit(cx),
        }
    }

    // ---------------------------------------------------------------- 渲染

    fn render_ui(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let q = self.query(cx);
        let visible = self.visible(&q);
        if let Some(sel) = self.selected
            && sel >= visible.len()
        {
            self.selected = None;
        }
        let selected = self.selected;
        let expanded = self.expanded();
        let pinned = self.pinned;
        let auto_hide = self.auto_hide;
        let tab = self.tab;
        let scroll = self.scroll;
        let view_kind = self.view;
        let card_w = self.card_w;
        let card_h = self.card_h;
        let search = self.search.clone();
        let this = cx.entity();

        let mut card = div()
            .id("card")
            .w(px(card_w))
            .h(px(card_h))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .rounded(px(CARD_R))
            .bg(c(CARD_BG))
            .overflow_hidden()
            // 整卡声明为窗口拖动区：交互组件在 mouse down 时停冒泡，
            // 只有空白处（含工具栏留白/列表空白）才会真正拖动窗口。
            .child(
                drag_zone("drag-card")
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full(),
            );

        match view_kind {
            View::List => {
                card = card.child(toolbar::render(&this, tab, pinned, &search));

                let total = ROW0_H + (self.items.len().saturating_sub(1)) as f32 * ROW_H;
                let list_h = card_h - LIST_TOP - LIST_BOTTOM_PAD;
                let max_scroll = (total - list_h).max(0.0);
                let v_scroll = this.clone();

                let list_el = div()
                    .id("list")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .flex_col()
                    .pt(px(10.))
                    .px(px(ROW_INSET))
                    .overflow_hidden()
                    .on_scroll(move |e, _, cx| {
                        let dy = match e.delta {
                            ScrollDelta::Pixels(p) => f32::from(p.y),
                            ScrollDelta::Lines(p) => p.y * 40.0,
                        };
                        v_scroll.update(cx, |this, cx| {
                            this.scroll = (this.scroll - dy).clamp(0.0, max_scroll);
                            cx.notify();
                        });
                    });

                let rows = if visible.is_empty() {
                    vec![
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(17.))
                            .text_color(c(MUTED))
                            .child("没有匹配的剪贴板记录")
                            .into_any_element(),
                    ]
                } else {
                    let offset = -scroll;
                    vec![
                        div()
                            .id("list-content")
                            .w_full()
                            .flex()
                            .flex_col()
                            .mt(px(offset))
                            .children(visible.iter().enumerate().map(|(pos, &real)| {
                                let item = &self.items[real];
                                let is_sel = selected == Some(pos);
                                let is_hovered = self.hovered == Some(pos);
                                row::render(&this, pos, real, item, is_sel, is_hovered)
                                    .into_any_element()
                            }))
                            .into_any_element(),
                    ]
                };

                card = card.child(list_el.children(rows));
            }
            View::Settings => {
                card = card.child(settings::render_with_state(&this, auto_hide, pinned));
            }
        }

        let preview_el = if expanded
            && let Some(sel) = selected
            && let Some(&real) = visible.get(sel)
        {
            Some(preview::render(&self.items[real]).into_any_element())
        } else {
            None
        };

        div()
            .id("root")
            .size_full()
            .flex()
            .items_start()
            .gap(px(PREVIEW_GAP))
            .child(card)
            .children(preview_el)
            .into_any_element()
    }
}

impl Render for UiApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_ui(cx).into_any_element()
    }
}

/// 注册键盘动作（main 调用一次）。
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, None),
        KeyBinding::new("down", MoveDown, None),
        KeyBinding::new("escape", Escape, None),
        KeyBinding::new("ctrl-f", FocusSearch, None),
        // enter / delete 绑定业务（复制、删除），UI 阶段不注册
    ]);
}

/// 把动作接到 view（open_window 后调用）。
pub fn wire_actions(view: &Entity<UiApp>, cx: &mut App) {
    let v = view.clone();
    cx.on_action(move |_: &MoveUp, cx| {
        v.update(cx, |this, cx| this.move_sel(-1, cx));
    });
    let v = view.clone();
    cx.on_action(move |_: &MoveDown, cx| {
        v.update(cx, |this, cx| this.move_sel(1, cx));
    });
    let v = view.clone();
    cx.on_action(move |_: &Escape, cx| {
        v.update(cx, |this, cx| this.on_escape(cx));
    });
    let v = view.clone();
    cx.on_action(move |_: &FocusSearch, cx| {
        v.update(cx, |this, cx| this.focus_search(cx));
    });
}

/// 后台轮询：托盘事件（调试期保留）。
pub fn spawn_poll(view: &Entity<UiApp>, cx: &mut App) {
    let weak = view.downgrade();
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            let Some(view) = weak.upgrade() else {
                break;
            };
            view.update(cx, |this, cx| {
                this.poll(cx);
            });
        }
    })
    .detach();
}
