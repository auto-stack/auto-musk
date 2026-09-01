# PLAN-050 T11 — 四界面 VM 对拍清单（2026-08-29）

环境：auto-lang master ga69d0021b（含 Plan 483 双焦点修复）+ plan-050-dev worktree
源（掩码/T8 面板/onenter），登录后经 AutoUI MCP 截图。截图在本目录。

## 验收标准逐项（VM 侧）

1. **主导航栏** ✅（t11-1-rail.png）：标题行 h-12 带底线（border-b ✓）；4 导航项
   左对齐 + 图标（message-square/list-todo/scroll/book-open 直绘 ✓）+ 激活项
   高亮底色（bg-primary/10 ✓）；底部工作区名显示真实降级文案"选择工作目录"
   （C7 ✓，`${currentName}` 裸串消失）。
2. **设置界面** ✅（t11-2-settings.png）：齿轮点开面板可见（VM 内联展开降级形态，
   C6 ✓）；mode/accent 切换按钮可达（i18n 文案上屏，C7 ✓）。
3. **文件夹选择界面** ✅（t11-3-ws.png）：点"选择工作目录"内嵌展开列表
   （切换 Workspace/最近打开/打开其他文件夹/路径输入/打开按钮）——T2 裁定的
   VM 降级通过态 ✓；打开动作依赖 VM storage/Http 降级（登记）。
4. **会话二级导航** ✅（t11-1-rail.png）：NavSidebar 头 + NavListItem 卡片列表
   可见、两行式（标题 + {count} 条）、选中描边、左对齐（C3/C1 ✓）。悬停删除钮
   未单独取证（卡片悬停态，留复查）。

## 已知残留（不阻塞，登记）

- 消息区 gate 卡文案 `${titleText}/${runId}...` 裸串：非本计划四界面范围
  （同族 computed 机制已修，gate 卡的取值链路归后续批次）。
- 浏览器侧对拍截图：待补（web 轨 127.0.0.1:8081 同四界面截取）。
- 登录后首帧 gate 按钮 Focus 环（MCP press 副作用），非缺陷。

## 待办（T12）

- [ ] vue 三门禁（auto build strict / vitest / lib-parity 30/30）
- [ ] style-parity 58 用例
- [ ] vm-link-probe PASS（已预验 61084 bytes）+ first-run reds=0
- [ ] KNOWN-DEBT 增 050 行
- [ ] auto-musk-dev（auto-lang）与 plan-050-dev（musk）worktree 折叠清理
