//! 主卡顶部工具栏：第一行是整行搜索胶囊，第二行是分类图标 + 置顶 / 设置。
//!
//! 两行布局让窄卡片也有足够的图标间距；第二行右侧留白同时充当
//! 「拖动窗口」的抓取区。

use eframe::egui::{Color32, CornerRadius, Painter, Pos2, Rect, pos2, vec2};

use crate::icons;
use crate::theme::{ACCENT, CARD_BG, NAVY, PILL, SEARCH_BG, TILE_NORMAL};

use crate::ui::rr;

/// 分类图标行的垂直中心。
pub const TABS_CY: f32 = 112.0;

const PAD_X: f32 = 30.0;
const TAB_FIRST: f32 = 44.0;
const TAB_GAP: f32 = 72.0;
const PIN_INSET: f32 = 99.0;
const SETTINGS_INSET: f32 = 43.0;

/// 各个可点击/可拖动区域。
pub struct Hits {
    pub search: Rect,
    pub tabs: [Rect; 5],
    pub pin: Rect,
    pub settings: Rect,
    drag: [Rect; 4],
}

impl Hits {
    /// 该点是否落在窗口拖动热区（工具栏留白、卡片上下留白）。
    pub fn is_drag(&self, pos: Pos2) -> bool {
        self.drag.iter().any(|r| r.contains(pos))
    }

    /// 该点命中的分类页签。
    pub fn tab_at(&self, pos: Pos2) -> Option<usize> {
        self.tabs.iter().position(|r| r.contains(pos))
    }
}

fn icon_button(center: Pos2) -> Rect {
    Rect::from_center_size(center, vec2(38.0, 38.0))
}

pub fn draw(p: &Painter, card: Rect, tab: usize, pinned: bool, pointer: Option<Pos2>) -> Hits {
    let w = card.width();
    let highlighted = |r: Rect| pointer.is_some_and(|pos| r.contains(pos));

    // 第一行：整行搜索胶囊
    let search = rr(card, PAD_X, 18.0, w - 2.0 * PAD_X, 58.0);
    p.rect_filled(search, CornerRadius::same(29), SEARCH_BG);
    icons::icon_bold(
        p,
        pos2(search.left() + 24.0, search.center().y),
        21.0,
        '\u{E721}',
        Color32::from_rgb(0x5E, 0x64, 0x80),
    );
    // 第二行：分类图标
    let mut tabs = [Rect::NOTHING; 5];
    for (i, rect) in tabs.iter_mut().enumerate() {
        let center = card.min + vec2(TAB_FIRST + TAB_GAP * i as f32, TABS_CY);
        *rect = icon_button(center);
        if highlighted(*rect) {
            p.rect_filled(*rect, CornerRadius::same(12), TILE_NORMAL);
        }
    }

    let color = |i: usize| if i == tab { ACCENT } else { NAVY };
    icons::grid(p, tabs[0].center(), color(0));
    icons::letter_t(p, tabs[1].center(), color(1));
    icons::image(p, tabs[2].center(), color(2));
    icons::file(p, tabs[3].center(), color(3));
    icons::link(p, tabs[4].center(), color(4), CARD_BG);

    // 选中页签下方的指示线
    p.rect_filled(
        rr(
            card,
            tabs[tab].center().x - card.left() - 15.0,
            132.0,
            30.0,
            3.0,
        ),
        CornerRadius::same(2),
        ACCENT,
    );

    // 第二行右侧：置顶 / 设置
    let pin_center = card.min + vec2(w - PIN_INSET, TABS_CY);
    let settings_center = card.min + vec2(w - SETTINGS_INSET, TABS_CY);
    let pin = icon_button(pin_center);
    let settings = icon_button(settings_center);
    if highlighted(pin) {
        p.rect_filled(pin, CornerRadius::same(10), PILL);
    }
    if highlighted(settings) {
        p.rect_filled(settings, CornerRadius::same(10), PILL);
    }
    icons::icon_bold(
        p,
        pin_center,
        24.0,
        '\u{E840}',
        if pinned { ACCENT } else { NAVY },
    );
    icons::icon_bold(p, settings_center, 23.0, '\u{E713}', NAVY);

    // 拖动热区：两条水平留白 + 第二行右侧空白 + 卡片底边
    let drag = [
        rr(card, 0.0, 0.0, w, 16.0),
        rr(card, 0.0, 78.0, w, 16.0),
        rr(
            card,
            TAB_FIRST + TAB_GAP * 5.0,
            92.0,
            (w - (PIN_INSET + 24.0) - (TAB_FIRST + TAB_GAP * 5.0)).max(0.0),
            40.0,
        ),
        rr(card, 0.0, card.height() - 12.0, w, 12.0),
    ];

    Hits {
        search,
        tabs,
        pin,
        settings,
        drag,
    }
}
