# in-paste 开发计划（v2，gpui-kit）

> **状态**：gpui-kit 重写阶段 1。窗口层（无边框拖拽 / 四边缩放 / 卡片跟随窗口）已完成（2026-11-13）。
> **基线**：设计说明 [`ui.md`](../ui.md)、设计稿 [`ui.png`](../ui.png)；尺寸与预览方式以本文 §4 / §6 为准。
> **验证记录**：[`REVIEW.md`](REVIEW.md) §8（重写 spike）与 §9（窗口层收尾）。
> egui 0.37 原型已弃用，历史实现见 git 与 `REVIEW.md` §1–§7，本文不再维护。

## 1. 产品与范围

轻量级、呼出式的 Windows 剪贴板管理器（Raycast 风格），不是完整的剪贴板管理窗口。

- **窗口**：无边框、透明、圆角；**卡片 = 窗口**，默认 672×816，最小 500×420；空白处拖动，四边/四角缩放。
- **工具栏**两行：搜索胶囊；分类 tab（全部 / 文本 / 图片 / 文件 / 链接）+ 窗口置顶 / 设置。
- **列表**：类型图标块 + 单行截断摘要 + 相对时间；hover 或选中时显示复制 / 置顶 / 删除。
- **预览**：选中行后的内容预览，从“同窗横向扩展”改为 **gpui-kit Popover**（待实现）。
- **呼出方式**：全局热键 `Ctrl + Shift + V`（模块已有，尚未接线）、系统托盘。

本期不做：云同步、跨平台、富文本编辑、完整剪贴板历史管理界面。

## 2. 已确认的决策

| # | 事项 | 结论 |
|---|---|---|
| 1 | 实现框架 | **gpui-kit 0.9.1**（内置 gpui-pre 0.3.12 + gpui-component + assets），edition 2024 |
| 2 | 卡片/窗口 | 卡片铺满窗口，无透明留白；窗口最小 500×420 |
| 3 | 拖动 | 整卡 `WindowControlArea::Drag` + 交互组件挡板（见 §6.1） |
| 4 | 缩放 | `WS_THICKFRAME` + WndProc 子类接管 `WM_NCCALCSIZE` / `WM_NCHITTEST`（见 §6.2） |
| 5 | 预览 | **gpui-kit [Popover](https://gpui-kit.com/component/popover)**（2026-11-13 决定），不再做单窗横向扩展 |
| 6 | 卡片阴影 / 描边 | **不做**（白底不留灰晕） |
| 7 | 半透明 / 磨砂 | **不做** |
| 8 | `Ctrl + 数字` | **不做** |
| 9 | 两个“置顶” | 工具栏置顶 = 窗口 always-on-top；记录置顶只影响排序 |
| 10 | 复制语义 | 复制按钮与 Enter 一致：复制 + 隐藏面板（待接入业务） |

## 3. 技术栈

- **GUI**：`gpui-kit = "=0.9.1"`（gpui-pre 0.3.12 + gpui-component + assets）。
- **窗口**：GPUI 原生 Win32 后端（不经过 winit）；`platform/windows.rs` 负责样式加固、`WS_THICKFRAME` + WndProc 子类、cloak、置顶。
- **透明**：`WindowBackgroundAppearance::Transparent` + 主题根背景置透明（gpui-component RootPlugin 默认刷不透明底色）。
- **字体**：`TextSystem::add_fonts` 加载 msyh.ttc / segoeui.ttf；主题 `font_family = Microsoft YaHei`。
- **热键 / 托盘**：`global-hotkey 0.9` / `tray-icon 0.26.1`；后台线程写原子标记，主循环 50ms 轮询处理。
- **依赖**：

```toml
gpui-kit = "=0.9.1"
global-hotkey = "0.9"
raw-window-handle = "0.6"
tray-icon = "0.26.1"
```

- Release：`opt-level = 3`、`lto = "thin"`；dev 下对 gpui 相关 crate 单独开 `opt-level = 3`。

## 4. 设计 token 与尺寸

唯一改样式的地方是 [`src/theme.rs`](../src/theme.rs)；当前核心值：

- 卡片：`672 × 816`，圆角 20；窗口最小 `500 × 420`。
- 工具栏：左右 padding 30、顶部拖拽带 18、搜索行 58、行间距 12、tabs 行 48；tabs 5 个 38 图标。
- 列表：起点 146、首行 90、行高 81、左右 inset 10、底部 13；图标块 64；行内按钮 34。
- 预览（待 Popover 重构）：宽 431、高 240、圆角 18、间距 20。
- 颜色：卡片 `#F6F6FD`、选中 `#C6D9FE`、强调 `#0D6BFC`、正文 `#1F2344`、弱化 `#6A7597` 等，全部在 `theme.rs`。

> `theme::window_size` 里的“预览增量”目前只保留公式，不参与窗口 resize；Popover 落地后应删除。

## 5. 模块结构

```
src/
  main.rs        字体 / 主题 / 窗口创建 / 动作绑定 / 后台轮询
  theme.rs       设计 token（颜色、尺寸、圆角）
  core/          纯逻辑：记录模型、分类与搜索过滤
  platform/      Win32：窗口加固/缩放/cloak/置顶、全局热键、托盘
  ui/
    mod.rs       drag_zone / no_drag_div 等公共原语
    app.rs       应用状态机、选择/过滤/滚动/窗口跟随
    toolbar.rs   搜索胶囊 + tabs + 置顶/设置
    row.rs       单行：图标块、标题、时间、hover 三按钮
    preview.rs   预览卡（待替换为 Popover）
    settings.rs  设置页
    icons.rs     图标与资产
```

## 6. 窗口层（已完成）

细节与源码依据见 [`REVIEW.md`](REVIEW.md) §9；这里只保留结论。

### 6.1 拖动

- 卡片子节点铺一层 `window_control_area(WindowControlArea::Drag)`：整卡在 `WM_NCHITTEST` 阶段是 `HTCAPTION`，空白处按住即可拖窗口。
- 交互组件必须挡一下，否则按下会被当成拖标题栏：
  - 点击类（行、tabs、按钮、设置页控件）用 `no_drag_div`（mouse down 停冒泡）；
  - 按住拖动类（搜索输入框、以后滑杆/弹层）用 `.occlude()`，走正常客户区鼠标路径；
  - 不要嵌套 Drag 区。
- 组件挡板漏掉会表现为：点击没反应、行 hover 三按钮不出现（`pending_mouse_down` 卡住）。

### 6.2 缩放

- 创建时 `is_resizable: false`（有意为之，让 GPUI `border_offset` 为 0）；`harden` 里加回 `WS_THICKFRAME`。
- WndProc 子类：`WM_NCCALCSIZE` 返回 0（客户区 = 整窗）；`WM_NCHITTEST` 四边/四角 6 逻辑 px 返回 `HT*`；缩放边缘的 `WM_NCLBUTTONDOWN` 直接交 `DefWindowProc`。
- 用户拉窗口 → `observe_window_bounds` 把 `window.viewport_size()` 回写 `card_w/card_h`，卡片始终铺满。
- 不做最大化/最小化；最小尺寸 500×420；顶部最外 6px 归缩放，18px 拖拽带实际剩 12px。

## 7. 功能现状

### 7.1 已实现

- 窗口：无边框透明圆角、卡片贴边、拖动、四边/四角缩放、最小尺寸、窗口置顶、系统托盘。
- 工具栏：搜索胶囊（IME、`Ctrl+F`、`Esc` 清空）、5 个分类 tab 过滤、置顶/设置入口。
- 列表：mock 数据、搜索/分类过滤、选中/hover、行内三按钮的 UI 反馈、滚动与空态。
- 设置页：返回、失焦自动隐藏开关（仅状态）、窗口置顶开关（真实 topmost）、退出。
- 键盘：`↑` / `↓` / `Esc` / `Ctrl+F`。
- 托盘：左键与菜单“显示 / 隐藏”当前是**聚焦窗口**；菜单设置/退出可用。

### 7.2 未接入（业务阶段）

- 真实剪贴板采集与写入；Enter / 复制按钮 / 删除 / 记录置顶的实际行为。
- 全局热键接线（`platform/hotkey.rs` 已有，`TOGGLE` 未被轮询）。
- cloak 显隐、失焦自动隐藏、点外部收起（`auto_hide` 目前只有 UI 状态）。
- 持久化、开机自启、打包。
- 预览 Popover。

### 7.3 调试期保留

- 系统托盘、窗口置顶；其余业务行为等 UI 定稿后再接。

## 8. 剩余里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| **M1 预览 Popover** | 用 gpui-kit `Popover` 替换 `preview.rs` 与单窗横向扩展；清理 `PREVIEW_*` / `expanded()` 遗留 | 选中行后浮层内容正确、层级/焦点不抢输入框、关闭干净 |
| **M2 剪贴板核心** | Windows 剪贴板监听（文本/图片/文件）、去重、写入、复制后收起 | 复制 → 系统剪贴板一致；面板按语义收起 |
| **M3 呼出与隐藏** | 接全局热键、cloak 显隐、失焦自动隐藏、点外部收起、托盘显示/隐藏 | 热键唤出/收起稳定，不抢前台；设置开关真实生效 |
| **M4 持久化与分发** | SQLite（或等价）持久化、设置持久化、开机自启、打包 | 重启数据/设置不丢；安装包可运行 |
| **M5 打磨** | DPI 100/125/150、1000 条滚动、空态/加载失败、截图验收 | 验收清单全过，无闪烁/残影 |

## 9. 验收清单（当前阶段）

- [ ] 卡片始终铺满窗口：四边贴边、四角圆角完整、无透明留白。
- [ ] 空白处拖动窗口；行、按钮、输入框可正常点击；输入框可拖选文本。
- [ ] 四边/四角缩放；最小 500×420；不做最大化。
- [ ] 工具栏顺序、搜索胶囊（中文 IME）、tabs 选中态、置顶/设置入口与设计稿一致。
- [ ] 行结构、hover/选中三按钮、滚动、空态正常。
- [ ] 设置页返回/开关/退出正常；置顶后窗口保持最前。
- [ ] 托盘左键/菜单可用。
- [ ] `cargo fmt --check`、`cargo clippy --all-targets`、`cargo build` 全过。
- [ ] 100% / 150% DPI 实机验证。

## 10. 风险与取舍

| 风险 | 影响 | 对策 |
|---|---|---|
| gpui-pre 锁定 0.3.12，部分 API 是空实现（如 `start_window_move`） | 拖动/缩放要自己接 Win32 | 已用整卡 Drag + WndProc 子类；升级 gpui-kit 时重新验证 |
| “整卡 Drag”与“组件点击”在 WM_NCHITTEST 层互斥 | 漏挡板 → 点击变拖窗口 | `no_drag_div` / `occlude` 契约写进 `ui/mod.rs` 注释与 REVIEW §9 |
| WndProc 子类保存静态旧 proc | 多窗口会互相覆盖 | 当前单窗口假设；多窗口前改成 per-hwnd |
| DWM NC 关闭 + 自绘圆角 | 无系统边框视觉反馈 | 边缘光标仍由 `WM_SETCURSOR` 提供；可后续加 hover 描边 |
| Popover 层级/焦点 | 抢搜索框焦点或拖拽区冲突 | M1 单独验证；必要时 `occlude` |
| 剪贴板格式/监听 | 漏事件或重复 | M2 用 `AddClipboardFormatListener` 或带序列号轮询 |
| DPI / 多屏 | 尺寸取整、跨屏拖动 | 实机 100/150% 验证；窗口逻辑像素由 GPUI 处理 |

## 11. 技术债

- `preview.rs` 与 `theme::window_size` 的预览增量、`expanded()` 的窗口耦合：Popover 落地时一起删。
- `platform/windows.rs` 的 `begin_drag`、`set_cloaked`、`platform/hotkey.rs` 目前未接线。
- `settings.rs` 里有一份 `#[allow(dead_code)]` 的旧 `render`，接线业务时删除。
- 系统托盘的“显示/隐藏”目前只是聚焦窗口，待 cloak 接入后修正。

## 12. 质量门

- `cargo fmt --check` / `cargo clippy --all-targets` / `cargo build` 必须全过。
- 窗口层改动需实机验证拖动、缩放、点击/滚动/输入框拖选回归。
- 截图与调试产物放 `.tmp/`（不入库）。
