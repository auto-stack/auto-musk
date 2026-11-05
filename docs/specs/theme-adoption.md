# 主题接入（theme-adoption）— auto-musk

> 来源：auto-os/PLAN-061（主合同）。宿主策略定义见 OS
> `docs/specs/shell/host-theme-policy-and-delivery.md`；批次登记见 OS
> `docs/specs/apps/product-theme-batch-061.md`。本页只记本产品的实际表面
> 与合法固定值，不重复宿主机制。

## 接入形态（2026-11-05，PLAN-061 T-02）

- `vendor/themes`：UI 管理渠道固定版本快照（themes 0.2.0，内容 digest
  `9fa07742dc68aa5377ae06cb2d966386002063f184c515b6c5dddf8a6bef838e`）。
- pac.at：theme 标量键缺省 = follow 宿主；`dep themes { path: "vendor/themes" }`；
  **PLAN-075 结构化品牌主题保留**（`theme { extends: "scaffold",
  colors: { primary/ring/primary-foreground } }`，对齐 Musk 品牌紫）。
- app.at：`use themes.themes: Auto` + `dark_mode` 契约位 + `design
  { theme: "Auto", mode: if .dark_mode … }`。

## 品牌 overlay 与宿主共存（本产品独有消费形态）

pac theme{} composed palette 作为**默认主题**种子（write-if-unset）：
默认 boot 品牌紫生效（#6466f1 族采样实测）。宿主策略完整候选
（family+mode）到货时整体接管 family/mode（Solarized/dark dominant
#002b36 实测）——品牌 overlay 不压宿主族/模式选择。

## 合法固定值（内容/业务面，不随主题）

- 品牌紫 primary/ring（238 55% 58% 族）：品牌身份，经 theme{} overlay
  保留；小鹿 logo（assets 双份量化内嵌）：品牌资产。
- 长任务/批准界面/白名单与安全数据交互：业务面零主题触面（本批不消费
  不验证不改动）。
- 会话/工作区/文件视图数据：业务面。

## 主题腿实录（tests/test_061_theme.py，5/5）

A 显式暗（dominant #09090b）/ B 显式亮 + 品牌紫 177 采样点（overlay 保留
负证）/ C Solarized/dark 宿主候选（dominant #002b36 精确）/ D rail 真实
切换 + 活体翻转 current_view 守恒 / E 冷恢复 healthy。
