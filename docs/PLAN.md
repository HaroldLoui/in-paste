# in-paste 开发计划（v1）

> 状态：已评审（2026-11-09），M0 技术验证通过；评审记录见 [`REVIEW.md`](REVIEW.md)  
> 基线需求：[`ui.md`](../ui.md)、设计稿 [`ui.png`](../ui.png)  
> 本期目标：**只做 UI 原型**（mock 数据），不做真实剪贴板监听、持久化、全局热键、托盘。

## 1. 产品与范围

轻量级、呼出式的 Windows 剪贴板管理器（Raycast 风格），不是完整管理窗口：

- 无边框圆角浮窗，固定尺寸约 780 × 520。
- 顶部工具栏：五个分类图标 tab（全部 / 文本 / 图片 / 文件 / 链接）+ 分隔线 + 搜索胶囊 + 窗口置顶 + 关闭。
- 中部列表：类型图标块 + 单行截断摘要 + 相对时间 + 右侧 `Ctrl + N` 提示胶囊；选中行淡蓝高亮。
- 右侧**独立预览浮层**（不是固定面板）：选中记录时出现，圆角、阴影、左缘三角指向选中行，含头部/正文/底部按钮（复制 / 置顶 / 删除）。

本期不做（后续里程碑另议）：真实剪贴板监听、SQLite 持久化、全局热键、托盘、开机自启、自动粘贴、云同步。

## 2. 已确认的决策

| # | 事项 | 结论 |
|---|---|---|
| 1 | `ui.md` 中 "Windows Reactor" 关键词 | 仅作为 **Win11 风格参考**，实现框架仍为 egui |
| 2 | 预览触发方式 | **选中才预览**（点击 / 键盘切换都改变选中）；hover 只做行底色微变，不弹预览 |
| 3 | 列表类型图标块尺寸 | 首版取 **56 × 56**（ui.md 建议值），后续按实际观感再调 |
| 4 | 半透明 / 磨砂玻璃 | **不做**。主窗与浮层均为不透明浅色卡片；"透明"只用于圆角外沿与窗体外阴影 |
| 5 | 预览浮层 | 必须是**独立顶层窗口**（位于主窗矩形之外），egui 中由第二个 viewport 实现 |
| 6 | 复制语义 | 复制按钮与 Enter 一致：**复制 + 关闭主窗**；置顶 / 删除不关闭、不改变主窗布局 |
| 7 | `Ctrl + 数字` | 复制该条并关闭（不是仅选中）；胶囊按当前过滤结果的前 9 条编号 |
| 8 | 两个"置顶" | `window_pinned`（工具栏，窗口 always-on-top）与 `item_pinned`（预览底部，记录置顶）分开实现 |

## 3. 技术栈与 M0 决策门

- GUI：**egui / eframe 0.37**，glow 后端。0.37 的 App trait 是 Ui-first：`fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame)`，旧 `update(ctx, frame)` 已不存在。
- 预览浮层：eframe **第二个 viewport**（无边框 + 透明 + 自绘）。
- 图标：**手绘矢量**（`painter` 绘制），不引入图标字体，保证任意 DPI 清晰。
- 字体：系统字体。egui 默认不含 CJK，必须运行时加载 `C:\Windows\Fonts\msyh.ttc`（回退 `msyh.ttf` / `simhei.ttf`），并配 Segoe UI 作为拉丁字形回退。`default_fonts` 关闭后 `FontDefinitions` 为空，必须 `ctx.set_fonts()`。
- 复制：egui 自带 `ctx.copy_text`；eframe features 必须含 `clipboard`，否则 egui-winit 只写进程内 fallback，系统剪贴板不更新。
- 图片：`image` crate 解码后 `ctx.load_texture`。

**M0 是唯一翻盘口**：三个风险点任一不通过，则转 Windows Reactor 方案（M1–M5 的组件与状态划分不变，仅替换渲染层）。

1. 无边框 + 透明 + 20px 圆角 + 自绘外阴影，在 100% / 125% / 150% DPI 下的观感；
2. 中文 IME：搜索框内组合输入、候选窗位置、实时过滤；
3. 第二个 viewport：透明、无边框、贴主窗右侧、跟随选中行、独立阴影、失焦与点击行为。

## 4. 设计 Token（从设计稿取样，首版）

| Token | 值 |
|---|---|
| `window.bg` | `#F3F2FD`（落地为不透明近白微紫） |
| `window.size` | 780 × 520 |
| `window.radius` | 20 |
| `window.shadow` | `0 12px 40px rgba(30, 42, 70, 0.14)` |
| `toolbar.height` | ~78 |
| `accent` | `#1468FD` |
| `text.primary` | `#1F2A44` |
| `text.secondary` | `#8A91A6` |
| `search.bg / height / radius` | `#E7E7F5` / 44 / 22 |
| `tab.indicator` | 32 × 3，`#1468FD` |
| `list.top` | 87（工具栏 78 + 9） |
| `row.height / radius / selected` | 88 / 12 / `#C6D9FE`（行距 89） |
| `icon.tile` | 56 × 56（首版），r16；文本块 `#D9E1F8`，文件块近白 |
| `shortcut.pill` | `rgba(30,42,70,6%)`，r10，13px |
| `preview.width / height / radius / bg / gap` | 430 / 337 / 16 / `#F6F6FD` / 20 |
| `preview.arrow` | 18 × 40 三角形，尖端距主卡右缘 2px，对齐选中行垂直中心 |
| `button.primary` | h50，r14，`#1468FD`，白字 |
| `divider` | `#E6E8F3` |

> 注：设计稿背景是 mockup 壁纸，取样颜色受其轻微影响；落地时以不透明近似值为准。

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
├─ main.rs          # eframe 启动、字体/主题初始化
├─ app.rs           # App 实现，窗口命令与浮层生命周期
├─ state.rs         # AppState / Command / 派生过滤与选中逻辑
├─ model.rs         # ClipItem / ClipKind / mock 数据 / 相对时间格式
├─ theme.rs         # 颜色、字号、间距、圆角、阴影、动效时长
├─ fonts.rs         # 系统字体加载与回退
├─ icons.rs         # 矢量图标绘制
├─ window.rs        # 主窗口与预览 viewport 的创建、定位
├─ shortcuts.rs     # 快捷键 → Command 映射
└─ ui/
   ├─ mod.rs
   ├─ toolbar.rs
   ├─ list.rs
   ├─ row.rs
   └─ preview.rs    # 浮层内容（在第二 viewport 中运行）
```

## 7. 依赖（仅原型）

```toml
[dependencies]
eframe = { version = "0.37", default-features = false, features = ["glow", "clipboard"] }
egui = "0.37"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
```

不引入：SQLite、serde、托盘、全局热键、剪贴板监听、自动粘贴相关的 crate。

## 8. 验收清单

- [ ] 780×520 无边框圆角窗口 + 柔和外阴影，不透明浅色背景，无侧边栏、无固定预览面板。
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
