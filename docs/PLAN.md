# in-paste 开发计划（v1）

> 状态：**gpui-kit 重写阶段 1**（分支 `feat/gpui-rewrite`）；窗口层拖拽/缩放已完成（2026-11-13，见 [`REVIEW.md`](REVIEW.md) §9 与本文 §12）。原 egui 实现见 git 历史。
> 基线需求：[`ui.md`](../ui.md)、设计稿 [`ui.png`](../ui.png)  
> 阶段 0 spike 已通过（7/7，结论见 [`REVIEW.md`](REVIEW.md) §8.2）；当前进入阶段 1 正式重写。

> 注：§1、§8 是 v1 计划原文，交互相关的「Ctrl+N 胶囊 / 预览底部按钮 / 关闭按钮」已在 2026-11-11 按外部反馈调整，以 §11 为准。**Ctrl+数字 快捷键已决定不做**（2026-11-12）。

## 1. 产品与范围

轻量级、呼出式的 Windows 剪贴板管理器（Raycast 风格），不是完整管理窗口：

- 无边框圆角浮窗，**设计坐标 720 × 920**（主卡 672 × 816）。
- 工具栏分**两行**：第一行搜索胶囊；第二行五个分类图标 tab（全部 / 文本 / 图片 / 文件 / 链接）+ 置顶 / 设置 + 选中下划线。
- 中部列表：类型图标块 + 单行截断摘要 + 相对时间；选中行淡蓝高亮，行内出现复制 / 置顶 / 删除按钮。
- 选中记录时窗口向右横向扩展，同窗透明区绘制右侧预览卡。

本期不做（后续里程碑另议）：真实剪贴板监听、SQLite 持久化、开机自启、自动粘贴、云同步。全局热键与托盘已在重写版中实现。

## 2. 已确认的决策

| # | 事项 | 结论 |
|---|---|---|
| 1 | 实现框架 | **gpui-kit 0.9.1**（2026-11-12 起；原 egui 0.37 见 git 历史） |
| 2 | 预览触发方式 | **选中才预览**（点击 / 键盘切换都改变选中）；hover 只做行底色微变 |
| 3 | 半透明 / 磨砂玻璃 | **不做**。主窗透明留白 + 不透明浅色卡片 |
| 4 | 预览浮层 | 单窗口横向扩展（窗口 resize），同窗透明区绘制 |
| 5 | 复制语义 | 复制按钮与 Enter 一致：**复制 + 隐藏面板** |
| 6 | `Ctrl + 数字` | **不做**（2026-11-12 决定） |
| 7 | 两个"置顶" | 工具栏置顶 = 窗口 always-on-top + 失焦不隐藏；记录置顶只影响排序 |
| 8 | 卡片阴影 / 描边 | **去掉**（2026-11-12 反馈：只要列表主体，白底不留灰晕） |

## 3. 技术栈（gpui-kit 重写版）

- GUI：**gpui-kit 0.9.1**（内置 gpui-pre 0.3.12 + gpui-component + assets）。
- 窗口：GPUI 原生 Win32 后端（不经过 winit）；`platform/windows.rs` 负责 hwnd 加固（剥 caption / WS_EX_TOOLWINDOW / 清标题 / 关 DWM NC）、加 `WS_THICKFRAME` 并子类化 WndProc 接管 `WM_NCCALCSIZE` / `WM_NCHITTEST` 实现四边缩放、cloak、置顶。拖拽用整卡 `WindowControlArea::Drag` + 交互组件挡板（见 §12）。
- 透明：`WindowBackgroundAppearance::Transparent` + 主题根背景置透明（gpui-component RootPlugin 默认刷不透明底色，必须覆盖）。
- 图标：`gpui_kit::assets::IconName`（Lucide 全量）+ 少量几何自绘（grid / T）。
- 字体：`TextSystem::add_fonts` 加载 msyh.ttc / segoeui.ttf，主题 `font_family = Microsoft YaHei`。
- 热键 / 托盘：`global-hotkey` / `tray-icon`（与 egui 版相同），后台线程写原子标记，50ms 轮询任务处理。
- 架构切分：`core`（纯逻辑）/ `platform`（Win32+热键+托盘）/ `theme`（token）/ `ui`（视图）。

**阶段 0 spike 已验证**（[`REVIEW.md`](REVIEW.md) §8.2）：无残影、IME 正常、cloak/置顶/hwnd 可用、白底无灰晕。

## 4. 设计 Token

见 `src/theme.rs`（唯一改样式的地方）。核心值与设计稿一致：accent `#0D6BFC`、卡片 `#F6F6FD`、选中 `#C6D9FE`、圆角 20、行高 81（首行 90）、列表起点 146。

## 5. 里程碑

| 阶段 | 内容 | 产出 / 验收 | 工时 |
|---|---|---|---|
| **M0 技术验证** | ✅ 已完成（2026-11-09）：三项决策门通过 | `docs/m0/` 对比截图 + [`REVIEW.md`](REVIEW.md)；中文输入并实时过滤；浮层定位/回焦可用 | 0.5d |
| **M1 骨架与设计系统** | 窗口（无边框/尺寸/圆角/阴影/拖动/失焦隐藏）；`theme.rs` token；`fonts.rs`；`icons.rs` 9 个矢量图标；主布局 toolbar + list；浮层焦点回还与最后一行裁剪 | 空窗口外框与设计稿一致 | 1–1.5d |
| **M2 工具栏 + 列表** | 5 个 tab（选中蓝 + 下划线 + hover 底色）；搜索胶囊（实时过滤）；窗口置顶/关闭；虚拟化滚动列表（行高 88）；行组件（图标块/截断/相对时间/Ctrl 胶囊/选中/hover）；mock 数据 8–10 条 | 与设计稿逐项对比；滚动不跳 | 2–2.5d |
| **M3 预览浮层** | 第二 viewport 创建与定位（右 → 左 → 下、clamp 到工作区、箭头对齐）；内容三区（文本 / 图片 / 文件 / URL）；底部按钮组；切换 120ms 交叉淡化；进入浮层不关闭 | 任意行位置正确；贴边自动翻转；无闪烁 | 2–3d |
| **M4 状态与交互** | 单一 `AppState`（selected 用 id 而非索引）；`Command` 枚举；↑ / ↓ / Enter / Esc / Delete / Ctrl+F / Ctrl+1..9；搜索框内仍可切换 | 键鼠状态一致；删空 / 过滤空不 panic | 1.5–2d |
| **M5 打磨与验收** | 字号/间距/阴影校准；空态、搜索无结果、图片加载失败；DPI 100/125/150 截图；1000 条滚动性能 | 验收清单 + 截图集 | 1.5–2d |

合计约 **9–12 个工作日**（个人项目晚间投入会更长）。

## 6. 模块结构

```
src/
├─ main.rs          # eframe 启动，组装 window / app
├─ app.rs           # App 实现：视图状态、窗口尺寸与隐藏/显示、拖动缩放、交互分发
├─ hotkey.rs        # 全局热键 Ctrl+Shift+V（global-hotkey，独立线程写原子标记）
├─ model.rs         # 记录类型 / mock 数据 / 分类过滤
├─ theme.rs         # 颜色、字号、间距、圆角、尺寸公式与最小尺寸
├─ fonts.rs         # 系统字体加载与回退
├─ icons.rs         # 矢量图标绘制
├─ window.rs        # 窗口创建、无边框样式剥离、WM_NCCALCSIZE 子类化、cloak 隐藏、光标/前台查询
└─ ui/
   ├─ mod.rs        # 卡片与阴影绘制、圆角矩形辅助
   ├─ toolbar.rs    # 搜索胶囊 + 分类图标 + 窗口置顶 / 设置 + 拖动热区
   ├─ list.rs       # 列表滚动、点击命中、行内按钮动作
   ├─ row.rs        # 单行：图标块 / 标题 / 时间 / hover 或选中时的三个小按钮
   ├─ preview.rs    # 右侧预览卡（单窗口横向扩展）
   └─ settings.rs   # 设置页：呼出快捷键、自动隐藏、窗口置顶、退出
```

## 7. 依赖（仅原型）

```toml
[dependencies]
eframe = { version = "0.37", default-features = false, features = ["glow", "clipboard"] }
egui = "0.37"
global-hotkey = "0.9"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
raw-window-handle = "0.6"
```

不引入：SQLite、serde、托盘、剪贴板监听、自动粘贴相关的 crate。

## 8. 验收清单

- [ ] 720 × 920（设计点，实际按 `UI_SCALE = 2/3` 缩放）无边框圆角窗口 + 柔和外阴影，不透明浅色背景，无侧边栏、无固定预览面板。
- [ ] 工具栏顺序、图标类型、选中蓝 + 下划线、hover 底色、搜索胶囊与占位文字「搜索剪贴板...」均与设计稿一致。
- [ ] 行结构（图标块 / 单行省略 / 相对时间 / `Ctrl + N` 胶囊）、选中淡蓝、hover 变化、点击选中、↑↓ 切换、Enter 复制并关闭、Ctrl+数字复制、内部滚动。
- [ ] 预览浮层独立于主窗、右侧 20px 间距、左缘三角指向选中行、圆角阴影、头部/正文/底部三段、复制蓝 + 置顶/删除灰、切换不闪烁、空间不足自动翻转。
- [ ] 列表与浮层共享同一选中状态；鼠标与键盘状态一致。
- [ ] mock 数据覆盖：文本、图片、URL、PDF、其他文件。
- [ ] 中文输入正常（IME），中英文混排字体协调。
- [ ] DPI 100% / 125% / 150% 下布局与字体不崩。

## 9. 风险

| 风险 | 影响 | 对策 |
|---|---|---|
| 中文 IME 在 egui 中体验不佳 | 搜索框不可用 | M0 验证；失败则转 Windows Reactor |
| 第二 viewport 透明 / 定位 / DPI 换算问题 | 预览浮层错位或黑底 | M0 验证；用主窗 outer_rect + 行 rect 计算全局坐标并 clamp |
| 自绘阴影观感与设计稿差距 | 视觉不达标 | M1 起调参，M5 统一校准 |
| TTC 多 face 索引与字体回退 | 中文豆腐块 | M1 验证 `FontData.index` 与逐字形回退 |
| egui 版本升级导致 API 变化 | 返工 | 锁定 0.37，升级单独排期 |

## 10. 后续（本期之外）

真实剪贴板监听 → SQLite 持久化 → 全局热键呼出 → 托盘与开机自启 → （可选）自动粘贴。

## 11. 实现现状（2026-11-11，egui 版）

> 注：本节记录 egui 版实现；gpui-kit 版窗口层以 §12 为准，§11.1 / §11.2 中的拖动缩放与尺寸公式已不适用。

### 11.1 交互模型

- **无快捷键胶囊**：列表行不显示 `Ctrl + N`，也不处理 `Ctrl + 数字`；用鼠标点选。
- **行内三按钮**：行被 hover 或选中时，右侧出现「复制 / 置顶（记录）/ 删除」三个 34×34 按钮；复制 = 复制内容 + 收起面板。
- **预览卡**：选中行后出现在右侧；已去掉页脚按钮组与分隔线，高度 338 → 240。所有行操作都在行内完成。
- **齿轮替换关闭按钮**：工具栏右侧是「窗口置顶」与「设置」；收起面板改为点卡片外、`Esc`、失焦自动隐藏、全局热键四种方式。
- **设置页**：呼出快捷键展示（`Ctrl + Shift + V`）、失焦自动隐藏开关、窗口置顶开关、退出、返回。
- **搜索**：搜索胶囊是真实输入框，支持中文 IME、`Ctrl + F` 聚焦、`Esc` 清空；按标题 / 预览正文 / 类型实时过滤，无结果显示空态。
- **键盘**：`↑` / `↓` 切换选中并滚动到可见，`Enter` 复制当前项并收起，`Ctrl + 1..9` 复制对应行，`Delete` 删除当前项。
- **系统托盘**：左键显示 / 隐藏面板；右键菜单提供显示 / 隐藏、设置、退出。
- **两种「置顶」**：工具栏置顶 = 窗口 `AlwaysOnTop` 且失焦不自动隐藏；记录置顶只影响排序与图标颜色。
- **呼出/收起**：全局热键 `Ctrl + Shift + V`；失焦自动隐藏（设置可关，`INPASTE_NO_AUTOHIDE=1` 可强制关）；点击卡片/预览外收起；`Esc` 从设置返回列表、再按收起。
- **拖动与缩放**：两行工具栏之间的留白、第二行右侧留白、卡片上下留白可拖动窗口；四边四角 7px 热区可缩放，最小卡片 460×420（窗口 508×484）。卡片尺寸是权威状态，窗口变大时卡片跟着变宽。

### 11.2 窗口与 DPI

- **无装饰创建**（`with_decorations(false)`）。注意 winit 0.30 的「无边框」其实是**保留 `WS_CAPTION|WS_BORDER|...` 样式、靠 `WM_NCCALCSIZE` 把非客户区吃掉**，并会在窗口属性 diff 时用 `to_window_styles()` 反复重写整套样式。早期做法是每帧把装饰样式剥掉，会和 winit 的样式重写互相打架（实测每次重写把客户区撑大 22×56）。
- **首帧窗口子类化**（`window::init_window`）：`SetWindowSubclass` 接管两个消息——
  - `WM_STYLECHANGING`：在系统应用前过滤掉 `WS_CAPTION / WS_THICKFRAME / WS_MINIMIZEBOX / WS_MAXIMIZEBOX / WS_SYSMENU`，并强制 `WS_EX_TOOLWINDOW`、清 `WS_EX_APPWINDOW`。winit 之后再重写样式也只会被过滤，窗口不会再真正进入「有 caption」状态。
  - `WM_NCCALCSIZE`：客户区 = 整窗。同时消掉 winit 给无装饰阴影预留的 1px 非客户区（之前顶部 1px 亮线的来源）。
- **关掉 DWM 非客户区渲染**：首帧 `DwmSetWindowAttribute(DWMWA_NCRENDERING_POLICY, DWMNCRP_DISABLED)` + `RedrawWindow(RDW_FRAME)`，清掉 DWM 在窗口创建阶段缓存的装饰帧。
- **隐藏用 DWM cloak**（`DWMWA_CLOAK`）而不是 `ShowWindow`：后者重新显示后会在顶部留 1px 白色条带；隐藏时把前台窗口还给唤出前的窗口。
- **置顶不走 winit**：`ViewportCommand::WindowLevel` 会触发 winit 的 window-flags diff、重写整套样式；改为 `window::set_topmost` 直接 `SetWindowPos(HWND_TOPMOST / HWND_NOTOPMOST)`，不碰样式。
- **GUI 子系统**：`src/main.rs` 始终 `windows_subsystem = "windows"`（debug 也是）。debug 若走 console 子系统，系统会额外创建一个控制台窗口；它位于置顶面板后面，而面板上下留白是透明的，控制台标题栏会从留白里透出来，看起来就像「窗口装饰闪一下」——这是该现象的主因。
- **尺寸公式**：窗口 = `theme::window_size(card, expanded)` = `(card.x + 48 + expanded ? (20 + 431) : 0, card.y + 104)`，卡片不小于 460×420。`UI_SCALE = 2/3` 统一缩放。
- **实测尺寸**：150% 主屏收起 720×920、展开 1171×920；100% 副屏收起 480×613、展开 781×613；跨屏拖动按 DIP 保持视觉尺寸（150% ↔ 100% 均验证）。
- **混合 DPI**（2560×1600@150% + 2560×1440@100%）：启动、展开、拖动、缩放、跨屏均验证通过。

### 11.3 已知缺陷（2026-11-11）

- **标题残影（已修复）**：抓帧确认残影只有窗口标题文字（没有标题栏底板），在激活、点击置顶、焦点切换时会出现。窗口始终是工具窗、不需要标题，因此首帧初始化时用 `SetWindowTextW(hwnd, L"")` 清空标题后，激活/置顶/焦点切换均不再出现 `in-paste` 残影。
- **调试输出**：GUI 子系统下 debug 的 stdout/stderr 不再出现在控制台；验证脚本仍可用 `-RedirectStandardError` 抓取。

### 11.4 质量门

`cargo build` 与 `cargo clippy --all-targets` 均 0 warning，`cargo fmt --check` 通过。验证脚本与截图放在 `.tmp/`（未入库）。

### 11.5 未实现

真实剪贴板监听、SQLite 持久化、开机自启、自动粘贴（均属后续里程碑）。

## 12. 窗口层现状（2026-11-13，gpui-kit）

窗口层（拖拽 + 缩放）已完成，最终形态以本节与 [`REVIEW.md`](REVIEW.md) §9 为准：

- **布局**：卡片 = 窗口，窗口不再保留透明留白；`theme::window_size` 只负责卡片（预览增量保留但暂不参与，后续换 popover）。最小 500×420。
- **拖动**：卡片铺一层 `WindowControlArea::Drag`；行 / tabs / 按钮 / 设置页控件用 `no_drag_div`（mouse down 停冒泡）保住点击；搜索输入框等"按住拖动"控件用 `.occlude()`，因为它需要正常客户区鼠标移动（NC 路径下 `pressed_button` 恒为 `None`）。
- **缩放**：创建时 `is_resizable: false`（有意为之，让 GPUI `border_offset` 为 0）；`harden` 里加回 `WS_THICKFRAME` 并用 `SetWindowLongPtrW(GWLP_WNDPROC)` 子类化：`WM_NCCALCSIZE` 返回 0（客户区 = 整窗）、`WM_NCHITTEST` 四边/四角 6 逻辑 px 返回 `HT*`、缩放边缘的 `WM_NCLBUTTONDOWN` 直接交 `DefWindowProc`。
- **跟随**：用户拖窗口 → `observe_window_bounds` 把 `window.viewport_size()` 回写 `card_w/card_h`；主卡 `flex_shrink_0`，选行渲染预览元素时不压缩主卡。
- **预览卡**：暂不参与窗口尺寸，后续改为 gpui-kit Popover；切过去时删除 `PREVIEW_*` 与 `expanded()` 相关窗口逻辑。
- **质量门**：`cargo fmt --check` / `cargo clippy --all-targets` / `cargo build` 全过。
