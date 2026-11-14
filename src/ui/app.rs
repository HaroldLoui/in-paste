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
use crate::ui::preview_window::{
    PREVIEW_MARGIN, PREVIEW_SCREEN_MARGIN, PREVIEW_WIN_H, PREVIEW_WIN_W, PreviewSide, PreviewState,
    PreviewWindow,
};
use crate::ui::{drag_zone, row, settings, toolbar};

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
    preview_state: Entity<PreviewState>,
    preview_window: Option<isize>,
    main_bounds: Bounds<Pixels>,
    main_scale: f32,
    main_hwnd: isize,
    _tray: Option<Tray>,
}

impl UiApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("搜索剪贴板..."));
        let preview_state = cx.new(|_| PreviewState::default());

        cx.subscribe_in(&search, window, |this, _state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.selected = None;
                this.scroll = 0.0;
                this.refresh_preview(cx);
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
            preview_state,
            preview_window: None,
            main_bounds: window.bounds(),
            main_scale: window.scale_factor(),
            main_hwnd: win::hwnd_of(window).unwrap_or(0),
            _tray: platform::tray::install(),
        };

        // 主窗口尺寸/位置是权威；卡片跟随 viewport，预览窗口跟随卡片和选中行。
        cx.observe_window_bounds(window, |this, window, cx| {
            this.main_bounds = window.bounds();
            this.main_scale = window.scale_factor();
            let viewport = window.viewport_size();
            let next_w = f32::from(viewport.width).max(MIN_CARD_W);
            let next_h = f32::from(viewport.height).max(MIN_CARD_H);
            if (this.card_w - next_w).abs() > 1.0 || (this.card_h - next_h).abs() > 1.0 {
                this.card_w = next_w;
                this.card_h = next_h;
                cx.notify();
            }
            this.refresh_preview(cx);
        })
        .detach();

        view
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

    fn selected_item(&self, cx: &App) -> Option<Item> {
        if self.view != View::List {
            return None;
        }
        let pos = self.selected?;
        let query = self.query(cx);
        let visible = self.visible(&query);
        visible.get(pos).map(|&real| self.items[real])
    }

    /// 选中行在卡片内的逻辑位置，以及预览窗口的左上角/相对主窗的位置。
    fn preview_placement(&self, pos: usize) -> (f32, f32, PreviewSide) {
        let row_h = row::height(pos) - 1.0;
        let row_top = LIST_TOP
            + if pos == 0 {
                0.0
            } else {
                (ROW0_H - 1.0) + (pos as f32 - 1.0) * (ROW_H - 1.0)
            }
            - self.scroll;

        // 主窗 window bounds 可能包含不可见的 DWM 边框；用 viewport 和卡片尺寸差
        // 估一个内容区原点，避免预览跟着边框偏移。
        let inset_x = ((f32::from(self.main_bounds.size.width) - self.card_w) / 2.0).max(0.0);
        let inset_y = ((f32::from(self.main_bounds.size.height) - self.card_h) / 2.0).max(0.0);
        let content_x = f32::from(self.main_bounds.origin.x) + inset_x;
        let content_y = f32::from(self.main_bounds.origin.y) + inset_y;

        let y = content_y + row_top + row_h / 2.0 - PREVIEW_H / 2.0 - PREVIEW_MARGIN;
        let right_x = content_x + self.card_w + PREVIEW_GAP - PREVIEW_MARGIN;
        let left_x = content_x - PREVIEW_GAP + PREVIEW_MARGIN - PREVIEW_WIN_W;

        // 右侧放得下就放右侧；放不下且左侧放得下就放左侧。
        // 两边都不够时选空间更大的一边（极端情况下仍可能压到主窗或屏幕边）。
        let mut side = PreviewSide::Right;
        if let Some((work_left, _, work_right, _)) = win::work_area(self.main_hwnd, self.main_scale)
        {
            let fits_right = right_x + PREVIEW_WIN_W + PREVIEW_SCREEN_MARGIN <= work_right;
            let fits_left = left_x >= work_left + PREVIEW_SCREEN_MARGIN;
            if !fits_right && fits_left {
                side = PreviewSide::Left;
            } else if !fits_right && !fits_left {
                let right_space = work_right - (right_x + PREVIEW_WIN_W);
                let left_space = left_x - work_left;
                if left_space > right_space {
                    side = PreviewSide::Left;
                }
            }
        }

        let x = match side {
            PreviewSide::Right => right_x,
            PreviewSide::Left => left_x,
        };
        (x, y, side)
    }

    fn create_preview_window(&mut self, cx: &mut Context<Self>, x: f32, y: f32) -> Option<isize> {
        let state = self.preview_state.clone();
        let owner = self.main_hwnd;
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                point(px(x), px(y)),
                size(px(PREVIEW_WIN_W), px(PREVIEW_WIN_H)),
            ))),
            titlebar: None,
            focus: false,
            show: true,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        };

        let opened = open_window(options, cx, move |window, cx| {
            let view = cx.new(|cx| PreviewWindow::new(state.clone(), cx));
            if let Some(hwnd) = win::hwnd_of(window) {
                // 先藏起来；exstyle/owner 在创建阶段完成，WndProc 在创建后再补装。
                win::prepare_preview(hwnd, owner);
                win::set_cloaked(hwnd, true);
            }
            view
        });

        let (handle, _view) = match opened {
            Ok(opened) => opened,
            Err(err) => {
                eprintln!("[in-paste] preview window failed: {err:#}");
                return None;
            }
        };
        let hwnd = match handle.update(cx, |_, window, _| win::hwnd_of(window)) {
            Ok(Some(hwnd)) => hwnd,
            _ => {
                let _ = handle.update(cx, |_, window, _| window.remove_window());
                return None;
            }
        };
        // GPUI 已经在 Windows 事件层把 WM_MOUSEACTIVATE 固定成 MA_ACTIVATE，
        // 必须在窗口创建完成后子类化，才能让点击预览不抢焦点。
        win::fix_preview_activation(hwnd);
        win::set_cloaked(hwnd, true);
        self.preview_window = Some(hwnd);
        Some(hwnd)
    }

    /// 启动时预创建预览窗口并 cloak，避免第一次选中时再走窗口创建。
    pub fn precreate_preview(&mut self, cx: &mut Context<Self>) {
        if self.preview_window.is_some_and(win::is_window) {
            return;
        }
        let (x, y, _) = self.preview_placement(0);
        let _ = self.create_preview_window(cx, x, y);
    }

    fn ensure_preview_window(&mut self, cx: &mut Context<Self>) -> Option<isize> {
        if let Some(hwnd) = self.preview_window
            && win::is_window(hwnd)
        {
            return Some(hwnd);
        }
        self.preview_window = None;
        let pos = self.selected?;
        let (x, y, _) = self.preview_placement(pos);
        self.create_preview_window(cx, x, y)
    }

    fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        let Some(item) = self.selected_item(cx) else {
            if let Some(hwnd) = self.preview_window {
                win::set_cloaked(hwnd, true);
            }
            return;
        };
        let Some(hwnd) = self.ensure_preview_window(cx) else {
            return;
        };
        let Some(pos) = self.selected else {
            return;
        };
        let (x, y, side) = self.preview_placement(pos);

        self.preview_state.update(cx, |state, cx| {
            if state.item != Some(item) || state.side != side {
                state.item = Some(item);
                state.side = side;
                cx.notify();
            }
        });
        win::move_window(hwnd, x, y, PREVIEW_WIN_W, PREVIEW_WIN_H, self.main_scale);
        win::set_cloaked(hwnd, false);
        win::set_topmost(hwnd, self.pinned);
    }

    // ---------------------------------------------------------------- 动作（供 toolbar/row/settings 调用）

    pub fn set_tab(&mut self, tab: usize, cx: &mut Context<Self>) {
        self.tab = tab;
        self.selected = None;
        self.scroll = 0.0;
        self.refresh_preview(cx);
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
        if let Some(hwnd) = self.preview_window {
            win::set_topmost(hwnd, pinned);
        }
    }

    pub fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.view = View::Settings;
        self.selected = None;
        self.refresh_preview(cx);
    }

    pub fn back_to_list(&mut self) {
        self.view = View::List;
    }

    pub fn toggle_auto_hide(&mut self) {
        self.auto_hide = !self.auto_hide;
    }

    pub fn exit(&mut self, cx: &mut Context<Self>) {
        // 现在是两个原生窗口（主窗 + 预览窗），单独 remove 主窗会把预览窗留下；
        // 退出走应用级 quit，让 GPUI 一并关闭所有窗口。
        cx.quit();
    }

    pub fn on_row_select(&mut self, pos: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = if self.selected == Some(pos) {
            None
        } else {
            Some(pos)
        };
        self.refresh_preview(cx);
        // 选中行后把键盘焦点留在搜索框：预览窗口是 no-activate 的，
        // 不会抢焦点；这里保持“选中后继续输入搜索”的手感。
        self.search.update(cx, |state, cx| state.focus(window, cx));
        cx.notify();
    }

    /// 行内按钮：UI 阶段只做选中反馈，不写剪贴板、不改数据、不藏面板。
    pub fn on_row_action(&mut self, _real: usize, action: row::RowAction, cx: &mut Context<Self>) {
        match action {
            row::RowAction::Copy | row::RowAction::Pin | row::RowAction::Delete => {
                // 业务后接：复制到剪贴板 / 置顶排序 / 删除记录
                self.selected = None;
                self.refresh_preview(cx);
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
        self.refresh_preview(cx);
        self.focus_search(cx);
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
        self.refresh_preview(cx);
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
                self.refresh_preview(cx);
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
                            this.refresh_preview(cx);
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

        // 预览是独立窗口，主窗口的尺寸/位置/内容都不再为它让路。
        div().id("root").size_full().child(card).into_any_element()
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
