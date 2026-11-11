//! 应用状态机：窗口尺寸与展开、视图切换、鼠标交互与全局热键。
//!
//! 布局模型见 `theme`：卡片尺寸是权威状态，窗口尺寸由它推导；
//! 用户拖动窗口边缘改变尺寸时，卡片跟着缩放（列表宽度自适应）。

use std::sync::atomic::Ordering;

use eframe::egui::{
    self, Context, CursorIcon, Key, Pos2, Rect, TextureHandle, TextureOptions, Vec2,
    ViewportCommand, pos2, vec2,
};
use eframe::{CreationContext, Frame};

use crate::hotkey::{self, Hotkey};
use crate::model::{self, Item};
use crate::theme::{self, MARGIN_BOTTOM, MARGIN_TOP, MARGIN_X, MIN_CARD_H, MIN_CARD_W, UI_SCALE};
use crate::ui::{self, list, preview, row, settings, toolbar};
use crate::{fonts, tray, window};

/// 主卡显示列表还是设置页。
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    List,
    Settings,
}

/// 自定义缩放的拖动方向。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    N,
    S,
    E,
    W,
    Ne,
    Nw,
    Se,
    Sw,
}

impl Dir {
    fn has_east(self) -> bool {
        matches!(self, Self::E | Self::Ne | Self::Se)
    }
    fn has_west(self) -> bool {
        matches!(self, Self::W | Self::Nw | Self::Sw)
    }
    fn has_north(self) -> bool {
        matches!(self, Self::N | Self::Ne | Self::Nw)
    }
    fn has_south(self) -> bool {
        matches!(self, Self::S | Self::Se | Self::Sw)
    }

    fn cursor(self) -> CursorIcon {
        match self {
            Self::N | Self::S => CursorIcon::ResizeVertical,
            Self::E | Self::W => CursorIcon::ResizeHorizontal,
            Self::Ne | Self::Sw => CursorIcon::ResizeNeSw,
            Self::Nw | Self::Se => CursorIcon::ResizeNwSe,
        }
    }
}

/// 正在进行的窗口缩放。
///
/// 基准是按下瞬间的窗口矩形（外框 + 内容尺寸），位移用**鼠标屏幕绝对坐标**算，
/// 这样窗口在缩放中移动不会把位移重复计入（曾因此出现正反馈越拉越大）。
#[derive(Clone, Copy)]
struct Resize {
    dir: Dir,
    start_win: Vec2,
    start_outer: Pos2,
}

pub struct UiApp {
    thumb: Option<TextureHandle>,
    items: Vec<Item>,
    /// 选中项在 `visible` 里的位置。
    selected: Option<usize>,
    tab: usize,
    view: View,
    scroll: f32,
    query: String,
    /// 卡片尺寸（设计点），窗口尺寸 = `theme::window_size(card, expanded)`。
    card: Vec2,
    /// 窗口置顶（钉住）：开启后失焦不隐藏。
    pinned: bool,
    /// 是否允许失焦自动隐藏（设置页可关，环境变量可强制关）。
    auto_hide: bool,
    hidden: bool,
    /// 隐藏前的前台窗口，cloak 隐藏后把焦点还给它（否则前台会留在不可见的窗口上）。
    prev_fg: Option<isize>,
    /// 首帧是否已经设过窗口置顶（`window::set_topmost` 需要拿到窗口句柄）。
    topmost_ready: bool,
    was_focused: bool,
    /// 是否已经真正拿到过一次焦点（避免启动首帧被误判为「刚失焦」而立即隐藏）。
    seen_focus: bool,
    resize: Option<Resize>,
    drag_from: Option<Pos2>,
    drag_fired: bool,
    /// 已发出但窗口尚未跟上的目标尺寸。
    target: Option<Vec2>,
    _hotkey: Option<Hotkey>,
    _tray: Option<tray::Tray>,
}

impl UiApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        fonts::install(&cc.egui_ctx);
        // 内部布局仍按设计稿点数计算，靠 zoom 统一缩放成实际界面尺寸。
        cc.egui_ctx.set_zoom_factor(UI_SCALE);

        // 测试时可用 INPASTE_NO_AUTOHIDE=1 关掉失焦自动隐藏，方便截图/自动化验证。
        let auto_hide = std::env::var_os("INPASTE_NO_AUTOHIDE").is_none();
        let hotkey = Hotkey::install(&cc.egui_ctx);

        Self {
            thumb: load_thumb(&cc.egui_ctx),
            items: model::initial_items(),
            selected: None,
            tab: 0,
            view: View::List,
            scroll: 0.0,
            query: String::new(),
            card: vec2(theme::DEFAULT_CARD_W, theme::DEFAULT_CARD_H),
            pinned: false,
            auto_hide,
            hidden: false,
            prev_fg: None,
            topmost_ready: false,
            was_focused: true,
            seen_focus: false,
            resize: None,
            drag_from: None,
            drag_fired: false,
            target: None,
            _hotkey: hotkey,
            _tray: tray::install(&cc.egui_ctx),
        }
    }

    fn expanded(&self) -> bool {
        self.view == View::List && self.selected.is_some()
    }

    /// 当前分类过滤 + 置顶优先排序后的真实下标。
    fn visible(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        let mut visible: Vec<usize> = (0..self.items.len())
            .filter(|&i| self.items[i].kind.matches_tab(self.tab))
            .filter(|&i| self.items[i].matches_query(&query))
            .collect();
        visible.sort_by_key(|&i| !self.items[i].pinned);
        visible
    }

    fn set_window(&mut self, ctx: &Context, size: Vec2) {
        self.target = Some(size);
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.request_repaint();
    }

    /// 吸附窗口尺寸：用户/系统改过窗口（或刚展开）时同步卡片。
    fn settle(&mut self, ctx: &Context, win: Vec2) {
        if let Some(target) = self.target {
            if (win.x - target.x).abs() <= 1.5 && (win.y - target.y).abs() <= 1.5 {
                self.target = None;
            } else {
                return;
            }
        }
        if self.resize.is_some() {
            return;
        }

        let expanded = self.expanded();
        let expected = theme::window_size(self.card, expanded);
        if (win.x - expected.x).abs() > 1.5 || (win.y - expected.y).abs() > 1.5 {
            // 窗口被用户或系统改动：卡片跟着变，但保证不小于最小尺寸。
            self.card = theme::card_size(win, expanded);
            let want = theme::window_size(self.card, expanded);
            if (win.x - want.x).abs() > 1.5 || (win.y - want.y).abs() > 1.5 {
                self.set_window(ctx, want);
            }
        }
    }

    /// 选中状态变化后，让窗口宽度跟上（展开/收起）。
    fn sync_expand(&mut self, ctx: &Context, screen: Rect) {
        let want = theme::window_size(self.card, self.expanded());
        if (screen.width() - want.x).abs() > 1.5 || (screen.height() - want.y).abs() > 1.5 {
            self.set_window(ctx, want);
        }
    }

    fn hide(&mut self, ctx: &Context, frame: &eframe::Frame) {
        if self.hidden {
            return;
        }
        self.hidden = true;
        self.selected = None;
        self.view = View::List;
        self.scroll = 0.0;
        self.resize = None;
        self.drag_from = None;
        // 用 DWM cloak 而不是 Visible(false)：走 ShowWindow 的隐藏/显示会在窗口顶部
        // 留下 1px 白色非客户区条带。cloak 之后窗口仍算「可见」，要自己把前台还回去。
        window::set_cloaked(frame, true);
        if let Some(hwnd) = window::frame_hwnd(frame)
            && window::foreground() == Some(hwnd)
        {
            let target = self
                .prev_fg
                .filter(|prev| *prev != hwnd && window::is_window(*prev))
                .or_else(|| window::shell_window().filter(|shell| *shell != hwnd));
            if let Some(target) = target {
                window::set_foreground(target);
                self.prev_fg = None;
            }
        }
        ctx.request_repaint();
    }

    fn show(&mut self, ctx: &Context, frame: &eframe::Frame) {
        self.hidden = false;
        self.view = View::List;
        self.selected = None;
        self.scroll = 0.0;
        self.query.clear();
        // 等焦点真正回来再置 true；否则首帧 focused 仍为 false 会被误判成「刚失焦」而立刻隐藏。
        self.was_focused = false;
        // 记下唤出前的前台窗口，隐藏时还给它。
        if let Some(hwnd) = window::frame_hwnd(frame) {
            let fg = window::foreground();
            self.prev_fg = fg.filter(|h| *h != hwnd && window::is_window(*h));
        }
        let size = theme::window_size(self.card, false);
        self.target = Some(size);
        window::set_cloaked(frame, false);
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.request_repaint();
    }

    /// 置顶开关：直接改窗口 z-order，避免 winit 的 `set_window_level`
    /// （它会重写整套装饰样式，导致标题栏闪一下）。
    fn apply_pin(&self, frame: &Frame) {
        window::set_topmost(frame, self.pinned);
    }

    /// 滚动到让第 `index` 行完整落在列表可视区内。
    fn ensure_visible(&mut self, card: Rect, index: usize) {
        let list_rect = list::list_rect(card);
        let row_rect = row::rect(card.translate(vec2(0.0, -self.scroll)), index);
        if row_rect.top() < list_rect.top() {
            self.scroll -= list_rect.top() - row_rect.top();
        } else if row_rect.bottom() > list_rect.bottom() {
            self.scroll += row_rect.bottom() - list_rect.bottom();
        }
        let max = list::max_scroll(card, self.visible().len());
        self.scroll = self.scroll.clamp(0.0, max);
    }

    /// 用鼠标屏幕绝对坐标驱动缩放：被拖的边跟着光标走，对边保持不动。
    fn update_resize(&mut self, ctx: &Context, down: bool) {
        let Some(r) = self.resize else {
            return;
        };
        if !down {
            self.resize = None;
            self.target = None;
            return;
        }
        // GetCursorPos 返回物理像素，除以 ppp 换算成设计点（150% 屏 ppp=1.0，100% 屏 ppp≈0.667）。
        let Some((cx, cy)) = window::cursor_screen() else {
            return;
        };
        let ppp = ctx.pixels_per_point();
        let cursor = pos2(cx / ppp, cy / ppp);
        let expanded = self.expanded();
        let min = theme::window_size(vec2(MIN_CARD_W, MIN_CARD_H), expanded);
        let right = r.start_outer.x + r.start_win.x;
        let bottom = r.start_outer.y + r.start_win.y;
        let mut size = r.start_win;
        let mut outer = r.start_outer;

        if r.dir.has_east() {
            size.x = (cursor.x - r.start_outer.x).max(min.x);
        }
        if r.dir.has_west() {
            size.x = (right - cursor.x).max(min.x);
            outer.x = right - size.x;
        }
        if r.dir.has_south() {
            size.y = (cursor.y - r.start_outer.y).max(min.y);
        }
        if r.dir.has_north() {
            size.y = (bottom - cursor.y).max(min.y);
            outer.y = bottom - size.y;
        }

        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(outer));
        self.card = theme::card_size(size, expanded);
    }
}

impl eframe::App for UiApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
    fn logic(&mut self, ctx: &Context, frame: &mut Frame) {
        if hotkey::TOGGLE.swap(false, Ordering::SeqCst) {
            if self.hidden {
                self.show(ctx, frame);
            } else {
                self.hide(ctx, frame);
            }
        }

        if let Some(action) = tray::take_action() {
            match action {
                tray::Action::Toggle => {
                    if self.hidden {
                        self.show(ctx, frame);
                    } else {
                        self.hide(ctx, frame);
                    }
                }
                tray::Action::Settings => {
                    if self.hidden {
                        self.show(ctx, frame);
                    }
                    self.view = View::Settings;
                    self.selected = None;
                    self.query.clear();
                }
                tray::Action::Quit => {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        // 首帧：装窗口子类化（接管非客户区），并把窗口置顶一次。
        if window::init_window(frame) {
            ctx.request_repaint();
        }
        if !self.topmost_ready {
            window::set_topmost(frame, true);
            self.topmost_ready = true;
        }

        let screen = ctx.input(|i| i.content_rect());
        // Ui-first 的首帧 content_rect 可能还是 15000×15000 的无限画布，
        // 尺寸离谱时不参与窗口/卡片同步，等下一帧拿到真实尺寸。
        if screen.width() < 8000.0 && screen.height() < 8000.0 {
            self.settle(&ctx, screen.size());
        }

        let card = Rect::from_min_size(screen.min + vec2(MARGIN_X, MARGIN_TOP), self.card);
        let p = ui.painter().clone();
        ui::draw_card(&p, card, theme::CARD_R, theme::CARD_BG);

        let pointer = ctx.input(|i| i.pointer.latest_pos());
        let pressed = ctx.input(|i| i.pointer.primary_pressed());
        let down = ctx.input(|i| i.pointer.primary_down());
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        let escape = ctx.input(|i| i.key_pressed(Key::Escape));
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or(true);
        let outer = ctx
            .input(|i| i.viewport().outer_rect)
            .map(|r| r.min)
            .unwrap_or_else(|| screen.min);

        let mut toolbar_hits: Option<toolbar::Hits> = None;
        let mut settings_hits: Option<settings::Hits> = None;
        let mut list_out: Option<list::Output> = None;
        let mut preview_area: Option<Rect> = None;

        let mut visible: Vec<usize> = Vec::new();

        // Ctrl+F：从任何页面回到列表并聚焦搜索框。
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::F)) {
            self.view = View::List;
            ctx.memory_mut(|m| m.request_focus(ui::search::id()));
        }

        match self.view {
            View::List => {
                toolbar_hits = Some(toolbar::draw(&p, card, self.tab, self.pinned, pointer));
                if let Some(hits) = &toolbar_hits {
                    let response = ui::search::draw(ui, hits.search, &mut self.query);
                    if response.changed() {
                        self.selected = None;
                        self.scroll = 0.0;
                    }
                }
                if let Some(sel) = self.selected
                    && sel >= visible.len()
                {
                    self.selected = None;
                }
                visible = self.visible();
                if let Some(sel) = self.selected
                    && sel >= visible.len()
                {
                    self.selected = None;
                }

                list_out = Some(list::draw(
                    &p,
                    card,
                    &list::Input {
                        items: &self.items,
                        visible: &visible,
                        selected: self.selected,
                        scroll: self.scroll,
                        pointer,
                        pressed,
                        thumb: self.thumb.as_ref(),
                    },
                ));

                if let Some(sel) = self.selected
                    && let Some(&real) = visible.get(sel)
                {
                    let row_rect = row::rect(card.translate(vec2(0.0, -self.scroll)), sel);
                    let arrow_y = row_rect.center().y;
                    let top = (arrow_y - 70.0).clamp(
                        screen.top() + MARGIN_TOP,
                        (screen.bottom() - MARGIN_BOTTOM - theme::PREVIEW_H).max(MARGIN_TOP),
                    );
                    let preview_rect = Rect::from_min_size(
                        pos2(card.right() + theme::PREVIEW_GAP, top),
                        vec2(theme::PREVIEW_W, theme::PREVIEW_H),
                    );
                    preview_area = Some(preview_rect);
                    preview::draw(
                        &p,
                        preview_rect,
                        &self.items[real],
                        arrow_y,
                        self.thumb.as_ref(),
                    );
                }
            }
            View::Settings => {
                settings_hits = Some(settings::draw(
                    &p,
                    card,
                    self.auto_hide,
                    self.pinned,
                    pointer,
                ));
            }
        }

        // ---------- 键盘 ----------
        if self.view == View::List {
            let up = ctx.input(|i| i.key_pressed(Key::ArrowUp));
            let down = ctx.input(|i| i.key_pressed(Key::ArrowDown));
            if up || down {
                let len = visible.len();
                if len > 0 {
                    self.selected = Some(match (self.selected, down) {
                        (Some(i), true) => (i + 1).min(len - 1),
                        (Some(i), false) => i.saturating_sub(1),
                        (None, true) => 0,
                        (None, false) => len - 1,
                    });
                    if let Some(index) = self.selected {
                        self.ensure_visible(card, index);
                    }
                }
            }

            if ctx.input(|i| i.key_pressed(Key::Enter))
                && let Some(sel) = self.selected
                && let Some(&real) = visible.get(sel)
            {
                ctx.copy_text(self.items[real].title.to_owned());
                self.hide(&ctx, frame);
            }

            // Ctrl + 1..9：复制对应行并关闭，即使当前没有选中。
            let ctrl_digit = ctx.input(|i| {
                if !i.modifiers.ctrl {
                    return None;
                }
                [
                    Key::Num1,
                    Key::Num2,
                    Key::Num3,
                    Key::Num4,
                    Key::Num5,
                    Key::Num6,
                    Key::Num7,
                    Key::Num8,
                    Key::Num9,
                ]
                .iter()
                .position(|key| i.key_pressed(*key))
                .map(|n| n + 1)
            });
            if let Some(n) = ctrl_digit
                && let Some(&real) = visible.get(n - 1)
            {
                ctx.copy_text(self.items[real].title.to_owned());
                self.hide(&ctx, frame);
            }

            if ctx.input(|i| i.key_pressed(Key::Delete))
                && let Some(sel) = self.selected
                && let Some(&real) = visible.get(sel)
            {
                self.items.remove(real);
                let len = self.visible().len();
                self.selected = if len == 0 {
                    None
                } else {
                    Some(sel.min(len - 1))
                };
                self.scroll = self.scroll.min(list::max_scroll(card, len));
            }
        }

        // ---------- 交互 ----------
        if pressed && let Some(pos) = pointer {
            if let Some(dir) = resize_hit(screen, pos) {
                self.resize = Some(Resize {
                    dir,
                    start_win: screen.size(),
                    start_outer: outer,
                });
            } else {
                let mut handled = false;
                if let Some(hits) = &toolbar_hits {
                    if hits.settings.contains(pos) {
                        self.view = View::Settings;
                        self.selected = None;
                        handled = true;
                    } else if hits.pin.contains(pos) {
                        self.pinned = !self.pinned;
                        self.apply_pin(frame);
                        handled = true;
                    } else if let Some(tab) = hits.tab_at(pos) {
                        self.tab = tab;
                        self.selected = None;
                        self.scroll = 0.0;
                        handled = true;
                    } else if hits.search.contains(pos) {
                        ctx.memory_mut(|m| m.request_focus(ui::search::id()));
                        handled = true;
                    } else if hits.is_drag(pos) {
                        self.drag_from = Some(pos);
                        self.drag_fired = false;
                        handled = true;
                    }
                }
                if !handled && let Some(hits) = &settings_hits {
                    if hits.back.contains(pos) {
                        self.view = View::List;
                    } else if hits.auto_hide.contains(pos) {
                        self.auto_hide = !self.auto_hide;
                    } else if hits.pinned.contains(pos) {
                        self.pinned = !self.pinned;
                        self.apply_pin(frame);
                    } else if hits.exit.contains(pos) {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                }
                if !handled
                    && let Some(out) = &list_out
                    && let Some(index) = out.pressed_row
                {
                    self.selected = if self.selected == Some(index) {
                        None
                    } else {
                        Some(index)
                    };
                }
                // 点击窗口留白（卡片与预览卡之外）→ 收起；钉住时不收起。
                let inside =
                    card.contains(pos) || preview_area.is_some_and(|area| area.contains(pos));
                if !handled && !inside && !self.pinned && self.resize.is_none() {
                    self.hide(&ctx, frame);
                }
            }
        }

        // 行内三个按钮的动作
        if let Some((index, action)) = list_out.as_ref().and_then(|o| o.action)
            && let Some(&real) = visible.get(index)
        {
            match action {
                list::Action::Copy => {
                    ctx.copy_text(self.items[real].title.to_owned());
                    self.hide(&ctx, frame);
                }
                list::Action::Pin => {
                    self.items[real].pinned = !self.items[real].pinned;
                    self.selected = None;
                }
                list::Action::Delete => {
                    self.items.remove(real);
                    self.selected = None;
                }
            }
        }

        // 滚轮
        if scroll_delta != 0.0
            && pointer.is_some_and(|pos| list::list_rect(card).contains(pos))
            && self.view == View::List
        {
            visible = self.visible();
            let max = list::max_scroll(card, visible.len());
            self.scroll = (self.scroll - scroll_delta).clamp(0.0, max);
        }

        // Esc：设置页 → 列表；有选中 → 收起；否则隐藏
        if escape {
            match self.view {
                View::Settings => self.view = View::List,
                View::List => {
                    let search_focused = ctx.memory(|m| m.has_focus(ui::search::id()));
                    if search_focused || !self.query.is_empty() {
                        self.query.clear();
                        ctx.memory_mut(|m| m.surrender_focus(ui::search::id()));
                        self.selected = None;
                        self.scroll = 0.0;
                    } else if self.selected.take().is_none() {
                        self.hide(&ctx, frame);
                    }
                }
            }
        }

        // 拖动窗口
        if let Some(from) = self.drag_from {
            if down {
                if let Some(pos) = pointer
                    && !self.drag_fired
                    && (pos - from).length() > 3.0
                {
                    ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    self.drag_fired = true;
                }
            } else {
                self.drag_from = None;
                self.drag_fired = false;
            }
        }

        self.update_resize(&ctx, down);
        if self.resize.is_none() && self.drag_from.is_none() {
            self.sync_expand(&ctx, ctx.input(|i| i.content_rect()));
        }

        // 失焦自动隐藏（必须先真正获得过焦点，避免启动首帧就隐藏）
        if focused {
            self.seen_focus = true;
        }
        // 面板没焦点时，前台一定属于别的应用：记下来，隐藏时把焦点还回去。
        if !focused
            && let Some(hwnd) = window::frame_hwnd(frame)
            && let Some(fg) = window::foreground()
            && fg != hwnd
            && window::is_window(fg)
        {
            self.prev_fg = Some(fg);
        }
        // 被 cloak 隐藏时键盘焦点不能留在不可见窗口上：有记录就还回去，
        // 没有记录（例如启动后还没切过别的窗口）就退回桌面。
        if self.seen_focus
            && self.was_focused
            && !focused
            && self.auto_hide
            && !self.pinned
            && self.resize.is_none()
        {
            self.hide(&ctx, frame);
        }
        self.was_focused = focused;

        // 光标
        let mut cursor = CursorIcon::Default;
        if let Some(pos) = pointer {
            if let Some(dir) = resize_hit(screen, pos) {
                cursor = dir.cursor();
            } else if let Some(hits) = &toolbar_hits {
                if hits.pin.contains(pos)
                    || hits.settings.contains(pos)
                    || hits.tab_at(pos).is_some()
                {
                    cursor = CursorIcon::PointingHand;
                }
            } else if let Some(hits) = &settings_hits
                && (hits.back.contains(pos)
                    || hits.auto_hide.contains(pos)
                    || hits.pinned.contains(pos)
                    || hits.exit.contains(pos))
            {
                cursor = CursorIcon::PointingHand;
            }
            if let Some(out) = &list_out
                && out.hovered.is_some()
            {
                cursor = CursorIcon::PointingHand;
            }
        }
        ctx.set_cursor_icon(cursor);
    }
}

/// 判断点是否落在窗口边缘的缩放手柄上（窗口坐标）。
///
/// 只在客户端区域内部判定：窗口外的点收不到鼠标消息，判定成手柄也没有意义。
fn resize_hit(screen: Rect, pos: Pos2) -> Option<Dir> {
    const B: f32 = 7.0;
    if !screen.contains(pos) {
        return None;
    }
    let west = pos.x <= screen.left() + B;
    let east = pos.x >= screen.right() - B;
    let north = pos.y <= screen.top() + B;
    let south = pos.y >= screen.bottom() - B;

    match (north, south, west, east) {
        (true, _, true, _) => Some(Dir::Nw),
        (true, _, _, true) => Some(Dir::Ne),
        (_, true, true, _) => Some(Dir::Sw),
        (_, true, _, true) => Some(Dir::Se),
        (true, _, _, _) => Some(Dir::N),
        (_, true, _, _) => Some(Dir::S),
        (_, _, true, _) => Some(Dir::W),
        (_, _, _, true) => Some(Dir::E),
        _ => None,
    }
}

fn load_thumb(ctx: &Context) -> Option<TextureHandle> {
    let bytes = include_bytes!("../assets/thumb.png");
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture("thumb", color, TextureOptions::LINEAR))
}
