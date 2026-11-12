//! 应用状态机：视图切换、选中/过滤、窗口显隐与尺寸、键盘与托盘/热键。

use std::time::Duration;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;

use crate::core::{self, Item};
use crate::platform::{self, Hotkey, Tray, TrayAction, windows::win};
use crate::theme::{self, *};
use crate::ui::{preview, row, settings, toolbar};

actions!(
    in_paste,
    [
        MoveUp,
        MoveDown,
        Confirm,
        DeleteSelected,
        Escape,
        FocusSearch,
    ]
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    List,
    Settings,
}

pub struct UiApp {
    window: AnyWindowHandle,
    items: Vec<Item>,
    selected: Option<usize>,
    tab: usize,
    view: View,
    scroll: f32,
    search: Entity<InputState>,
    card_w: f32,
    card_h: f32,
    pinned: bool,
    auto_hide: bool,
    hidden: bool,
    prev_fg: Option<isize>,
    hardened: bool,
    topmost_ready: bool,
    was_active: bool,
    seen_active: bool,
    _hotkey: Option<Hotkey>,
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

        let auto_hide = std::env::var_os("INPASTE_NO_AUTOHIDE").is_none();

        Self {
            window: window.window_handle(),
            items: core::initial_items(),
            selected: None,
            tab: 0,
            view: View::List,
            scroll: 0.0,
            search,
            card_w: DEFAULT_CARD_W,
            card_h: DEFAULT_CARD_H,
            pinned: false,
            auto_hide,
            hidden: false,
            prev_fg: None,
            hardened: false,
            topmost_ready: false,
            was_active: true,
            seen_active: false,
            _hotkey: Hotkey::install(),
            _tray: platform::tray::install(),
        }
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

    fn window_size(&self) -> (f32, f32) {
        theme::window_size(self.card_w, self.card_h, self.expanded())
    }

    fn sync_window(&self, cx: &mut Context<Self>) {
        let (w, h) = self.window_size();
        let handle = self.window;
        let _ = handle.update(cx, |_, window, _| {
            window.resize(size(px(w), px(h)));
        });
    }

    // ---------------------------------------------------------------- 动作（供 toolbar/row/settings 调用）

    pub fn set_tab(&mut self, tab: usize) {
        self.tab = tab;
        self.selected = None;
        self.scroll = 0.0;
    }

    pub fn toggle_pin(&mut self, cx: &mut Context<Self>) {
        self.pinned = !self.pinned;
        self.apply_pin_now(cx);
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

    pub fn on_row_select(&mut self, pos: usize) {
        self.selected = if self.selected == Some(pos) {
            None
        } else {
            Some(pos)
        };
    }

    pub fn on_row_action(&mut self, real: usize, action: row::RowAction, cx: &mut Context<Self>) {
        match action {
            row::RowAction::Copy => {
                let title = self.items[real].title.to_owned();
                cx.write_to_clipboard(ClipboardItem::new_string(title));
                self.hide(cx);
            }
            row::RowAction::Pin => {
                self.items[real].pinned = !self.items[real].pinned;
                self.selected = None;
            }
            row::RowAction::Delete => {
                self.items.remove(real);
                self.selected = None;
                self.sync_window(cx);
            }
        }
    }

    // ---------------------------------------------------------------- 显隐

    fn hide(&mut self, cx: &mut Context<Self>) {
        if self.hidden {
            return;
        }
        self.hidden = true;
        self.selected = None;
        self.view = View::List;
        self.scroll = 0.0;

        let handle = self.window;
        let prev_fg = self.prev_fg;
        let _ = handle.update(cx, |_, window, _| {
            let Some(hwnd) = win::hwnd_of(window) else {
                return;
            };
            win::set_cloaked(hwnd, true);
            if win::foreground() == hwnd {
                let target = prev_fg
                    .filter(|prev| *prev != hwnd && win::is_window(*prev))
                    .or_else(|| {
                        let s = win::shell_window();
                        (s != 0 && s != hwnd).then_some(s)
                    });
                if let Some(target) = target {
                    win::set_foreground(target);
                }
            }
        });
        cx.notify();
    }

    fn show(&mut self, cx: &mut Context<Self>) {
        self.hidden = false;
        self.view = View::List;
        self.selected = None;
        self.scroll = 0.0;
        self.was_active = false;

        let (w, h) = theme::window_size(self.card_w, self.card_h, false);
        let handle = self.window;
        let search = self.search.clone();
        let prev_fg_slot = &mut self.prev_fg;
        let _ = handle.update(cx, |_, window, cx| {
            if let Some(hwnd) = win::hwnd_of(window) {
                let fg = win::foreground();
                if fg != 0 && fg != hwnd && win::is_window(fg) {
                    *prev_fg_slot = Some(fg);
                }
                win::set_cloaked(hwnd, false);
            }
            window.resize(size(px(w), px(h)));
            window.activate_window();
            search.update(cx, |state, cx| state.focus(window, cx));
        });
        cx.notify();
    }

    fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        if self.hidden {
            self.show(cx);
        } else {
            self.hide(cx);
        }
    }

    fn apply_pin_now(&self, cx: &mut Context<Self>) {
        let pinned = self.pinned;
        let handle = self.window;
        let _ = handle.update(cx, |_, window, _| {
            if let Some(hwnd) = win::hwnd_of(window) {
                win::set_topmost(hwnd, pinned);
            }
        });
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

    fn confirm_copy(&mut self, cx: &mut Context<Self>) {
        let q = self.query(cx);
        let visible = self.visible(&q);
        if let Some(sel) = self.selected
            && let Some(&real) = visible.get(sel)
        {
            let title = self.items[real].title.to_owned();
            cx.write_to_clipboard(ClipboardItem::new_string(title));
            self.hide(cx);
        }
    }

    fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let q = self.query(cx);
        let visible = self.visible(&q);
        if let Some(sel) = self.selected
            && let Some(&real) = visible.get(sel)
        {
            self.items.remove(real);
            let visible = self.visible(&q);
            self.selected = if visible.is_empty() {
                None
            } else {
                Some(sel.min(visible.len() - 1))
            };
            cx.notify();
        }
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
                } else if self.selected.take().is_none() {
                    self.hide(cx);
                } else {
                    cx.notify();
                }
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

    // ---------------------------------------------------------------- 后台轮询

    pub fn poll(&mut self, cx: &mut Context<Self>) {
        if platform::hotkey::take_toggle() {
            self.toggle_panel(cx);
            return;
        }
        if let Some(action) = platform::tray::take_action() {
            match action {
                TrayAction::Toggle => self.toggle_panel(cx),
                TrayAction::Settings => {
                    if self.hidden {
                        self.show(cx);
                    }
                    self.view = View::Settings;
                    self.selected = None;
                    cx.notify();
                }
                TrayAction::Quit => self.exit(cx),
            }
            return;
        }

        if self.hidden {
            return;
        }

        let handle = self.window;
        let active = handle
            .update(cx, |_, window, _| window.is_window_active())
            .unwrap_or(false);

        if active {
            self.seen_active = true;
            self.was_active = true;
        } else {
            let prev = handle.update(cx, |_, window, _| {
                let hwnd = win::hwnd_of(window)?;
                let fg = win::foreground();
                (fg != 0 && fg != hwnd && win::is_window(fg)).then_some(fg)
            });
            if let Ok(Some(fg)) = prev {
                self.prev_fg = Some(fg);
            }
            if self.seen_active && self.was_active && self.auto_hide && !self.pinned {
                self.hide(cx);
            }
            self.was_active = false;
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
            .flex()
            .flex_col()
            .rounded(px(CARD_R))
            .bg(c(CARD_BG))
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());

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
                                row::render(&this, pos, real, item, is_sel).into_any_element()
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

        let v_root = this.clone();
        div()
            .id("root")
            .size_full()
            .flex()
            .items_start()
            .gap(px(PREVIEW_GAP))
            .pt(px(MARGIN_TOP))
            .pb(px(MARGIN_BOTTOM))
            .px(px(MARGIN_X))
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                v_root.update(cx, |this, cx| {
                    if !this.pinned && !this.hidden {
                        this.hide(cx);
                    }
                });
            })
            .child(card)
            .children(preview_el)
            .into_any_element()
    }
}

impl Render for UiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.hardened && win::harden(window).is_some() {
            self.hardened = true;
            eprintln!("[in-paste] hwnd hardened");
        }
        if !self.topmost_ready {
            if let Some(hwnd) = win::hwnd_of(window) {
                win::set_topmost(hwnd, true);
            }
            self.topmost_ready = true;
        }

        if self.hidden {
            return div().id("root-hidden").size_full().into_any_element();
        }

        self.render_ui(cx).into_any_element()
    }
}

/// 注册键盘动作（main 调用一次）。
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, None),
        KeyBinding::new("down", MoveDown, None),
        KeyBinding::new("enter", Confirm, None),
        KeyBinding::new("delete", DeleteSelected, None),
        KeyBinding::new("escape", Escape, None),
        KeyBinding::new("ctrl-f", FocusSearch, None),
    ]);
}

/// 把动作接到 view（open_window 后调用）。
pub fn wire_actions(view: &Entity<UiApp>, cx: &mut App) {
    macro_rules! wire {
        ($($action:ident => $method:ident),* $(,)?) => {
            $(
                let v = view.clone();
                cx.on_action(move |_: &$action, cx| {
                    let _ = v.update(cx, |this, cx| this.$method(cx));
                });
            )*
        };
    }
    wire!(
        MoveUp => move_up_action,
        MoveDown => move_down_action,
        Confirm => confirm_copy,
        DeleteSelected => delete_selected,
        Escape => on_escape,
        FocusSearch => focus_search,
    );
}

impl UiApp {
    fn move_up_action(&mut self, cx: &mut Context<Self>) {
        self.move_sel(-1, cx);
    }
    fn move_down_action(&mut self, cx: &mut Context<Self>) {
        self.move_sel(1, cx);
    }
}

/// 后台轮询任务：热键 / 托盘 / 失焦隐藏。
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
