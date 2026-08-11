# Design 005:APP Harness 配置组织(OS 引用 + APP 自建)

> **Status**: Design Draft — 待评审
> **前身**: `auto-os-config/designs/unified-harness-scoping.md`(统一 Harness + 作用域架构)
> **决策**: 合并视图(同一界面既选 OS 级又管自建);Phase 1 只继承不定制;项目根 = CWD。
> **范围**: 以 Auto Musk 为首个 APP 样例,设计配置界面如何组织"引用 OS 通用 + 自建独有"两部分。

---

## 1. 场景(你举的例子)

Auto Musk 需要:
- **引用 OS 通用的 agents**:`coder`、`reviewer`(在 OS 级 `~/.config/autoos/roles/` 定义,musk "选用")
- **自建 2 个独有 agent**:`musk-feature-dev`、`musk-bug-fixer`(musk 专属,同格式)
- skills 和 modes 同理(引用 + 自建)

**关键约束(Phase 1 简化)**:只做**直接继承**(原样用,不改字段)和**自建**(完全独立定义)。不做"定制"(inherit + 字段覆盖)。

---

## 2. 目录结构(存储层)

```
~/.config/autoos/                     ← OS 级(所有 app 共享)
  roles/                              ← OS 通用 roles
    coder.at + coder.soul.md
    reviewer.at + reviewer.soul.md
  skills/                             ← OS 通用 skills
    test-driven-development/SKILL.md
  modes/                              ← OS 通用 modes
    superpowers.at
  apps/
    musk/                             ← APP 级(Auto Musk 专属)
      config.at                       ← musk 运行时配置 + harness 引用清单
      harness/
        roles/                        ← musk 自建 roles
          musk-feature-dev.at + .soul.md
          musk-bug-fixer.at + .soul.md
        skills/                       ← musk 自建 skills
          musk-conventions/SKILL.md
        modes/                        ← musk 自建 modes
          musk-sprint.at
```

**config.at 里的 harness 引用清单**(已在 Plan 008 / 当前实现中):
```at
musk {
  daemon_url: "http://127.0.0.1:17654"
  harness {
    roles: [coder, reviewer]            # ← 引用的 OS 级 roles
    skills: [test-driven-development]   # ← 引用的 OS 级 skills
    modes: [superpowers]                # ← 引用的 OS 级 modes
  }
}
```

**APP 自建 harness** 直接存在 `apps/musk/harness/{roles,skills,modes}/`,格式与 OS 级**完全一致**。不需要在 config.at 里声明——目录扫描自动发现。

---

## 3. 解析顺序(运行时)

Agent 构建时的 harness 查找:
```
L0 编译内置(builtin) → L1 OS 级(~/.config/autoos/<kind>/) → L2 APP 级(apps/musk/harness/<kind>/)
```
- 同名覆盖:L2 > L1 > L0(APP 自建同名可覆盖 OS 级)
- APP 的 `harness` 引用清单 = **白名单**:只有在清单里的 OS 级 harness 才对这个 APP 可见
- APP 自建 harness = **总是可见**(不需引用清单)

**所以 musk 实际可用的 harness**:
- roles = 清单里的 OS 级(`coder`, `reviewer`) + 全部 APP 自建(`musk-feature-dev`, `musk-bug-fixer`)
- skills = 清单里的 OS 级 + 全部 APP 自建
- modes = 清单里的 OS 级 + 全部 APP 自建

---

## 4. UI 组织:合并视图(每 kind 一个区块)

Auto Musk 配置页的 **Harness 区** 按 kind(roles/skills/modes)分区块,每个区块**合并展示**:

```
┌─ Auto Musk ──────────────────────────────────────────────────┐
│                                                              │
│ ┌─ Daemon Connection ──────────────────────┐                │
│ │  ... (现有:daemon_url, auto-start...)    │                │
│ └──────────────────────────────────────────┘                │
│                                                              │
│ ┌─ Harness ─────────────────────────────────────────────────┐│
│ │                                                           ││
│ │  ┌─ Roles ──────────────────────────────────────────────┐ ││
│ │  │                                                       │ ││
│ │  │  OS-level (inherit)            2/7 selected           │ ││
│ │  │  ☑ 🟦 coder        max          [View →]              │ ││
│ │  │  ☑ 🟦 reviewer     pro          [View →]              │ ││
│ │  │  ☐ 🟦 architect    max          [View →]              │ ││
│ │  │  ☐ 🟦 tester       mid          [View →]              │ ││
│ │  │  ...                                                  │ ││
│ │  │                                                       │ ││
│ │  │  App-level (custom)            2 defined              │ ││
│ │  │  🟩 musk-feature-dev  max      [Edit] [Delete]        │ ││
│ │  │  🟩 musk-bug-fixer    pro      [Edit] [Delete]        │ ││
│ │  │                              [+ New Role]             │ ││
│ │  └───────────────────────────────────────────────────────┘ ││
│ │                                                           ││
│ │  ┌─ Skills ─────────────────────────────────────────────┐ ││
│ │  │  OS-level (inherit)            1/7 selected           │ ││
│ │  │  ☑ 🟦 test-driven-development  [View →]              │ ││
│ │  │  ☐ 🟦 brainstorming            [View →]              │ ││
│ │  │                                                       │ ││
│ │  │  App-level (custom)            1 defined              │ ││
│ │  │  🟩 musk-conventions          [Edit] [Delete]         │ ││
│ │  │                              [+ New Skill]            │ ││
│ │  └───────────────────────────────────────────────────────┘ ││
│ │                                                           ││
│ │  ┌─ Modes ──────────────────────────────────────────────┐ ││
│ │  │  ... (同上结构)                                       │ ││
│ │  └───────────────────────────────────────────────────────┘ ││
│ │                                                           ││
│ │                                           [Save harness]  ││
│ └───────────────────────────────────────────────────────────┘│
│                                                              │
│ [Save Configuration]                                        │
└──────────────────────────────────────────────────────────────┘
```

### 4.1 来源徽章

| 徽章 | 含义 | 来源 |
|---|---|---|
| 🟦 | OS 级(通用,所有 app 可引用) | `~/.config/autoos/<kind>/` |
| 🟩 | APP 级(自建,musk 专属) | `apps/musk/harness/<kind>/` |
| 🔒 | 编译内置(只读) | 代码内置 |

一眼看出"这个 role 哪来的"。

### 4.2 OS 级区(勾选清单)

- **复选框**:勾选 = 这个 APP 引用(继承)该 OS 级 harness;取消 = 不引用
- **只读**:OS 级 harness 在这里**不能编辑**(要改去 Harness ▸ Roles 模块)
- **[View →]**:点开看详情(soul/tier/skills),只读。想定制 → 提示"复制为 APP 自建"
- 已选数 / OS 总数:`2/7 selected`

### 4.3 APP 自建区(CRUD)

- **[Edit]**:进入编辑器(与 Roles 模块的编辑器**完全一致**——name/description/tier/allowed_tiers/skills/soul)
- **[Delete]**:删除(删的是 `apps/musk/harness/roles/<name>.at`)
- **[+ New Role]**:新建一个 APP 自建 role
- 保存路径:`apps/musk/harness/roles/<name>.at` + sidecar `.soul.md`

### 4.4 "复制为自建"(inherit → custom 桥梁)

用户在 OS 级看到一个想改的 role(比如 coder 想改 temperature):
1. 点 [View →] → 详情页有"复制为 APP 自建"按钮
2. 点击 → 复制到 `apps/musk/harness/roles/coder-custom.at`(可加 `inherit: "coder"` 借字段)
3. 在 APP 自建区出现,可编辑
4. 原始 OS 级 coder 不受影响

这就是 Phase 1 的"定制"路径:不是字段覆盖,而是**显式复制一份**。无歧义。

---

## 5. 后端 API(新增)

复用现有的 `/api/roles`、`/api/skills`、`/api/modes`(OS 级),新增 APP 级端点:

```
GET  /api/app-harness/{kind}                 → OS 可选清单(勾选) + APP 自建列表
GET  /api/app-harness/{kind}/{name}          → APP 自建详情
PUT  /api/app-harness/{kind}/{name}          → 创建/更新 APP 自建
DELETE /api/app-harness/{kind}/{name}        → 删除 APP 自建
```

`{kind}` = `roles` | `skills` | `modes`。

`GET /api/app-harness/{kind}` 返回:
```json
{
  "os_available": [
    { "name": "coder", "tier": "max", "selected": true, ... },
    { "name": "reviewer", "tier": "pro", "selected": false, ... }
  ],
  "app_custom": [
    { "name": "musk-feature-dev", "tier": "max", ... },
    { "name": "musk-bug-fixer", "tier": "pro", ... }
  ]
}
```

引用清单的更新(勾选/取消)走现有的 `PUT /api/app-config`(更新 `harness.roles/skills/modes` 数组)。

---

## 6. 与 Harness 模块(Agents/Skills/Roles)的关系

| 做什么 | 去哪 |
|---|---|
| **管理 OS 通用 harness**(增删改) | Harness ▸ Roles/Skills/Agents 模块 |
| **管理 APP 自建 harness**(增删改) | Auto Musk 配置页的 Harness 区 |
| **选择 APP 引用哪些 OS 级** | Auto Musk 配置页的 Harness 区(勾选) |
| **看 OS 级详情**(只读) | 两处都可(Harness 模块或 Auto Musk 的 [View →]) |

**不重复**:Harness 模块是 OS 级的 CRUD 入口;Auto Musk 是 APP 级的 CRUD + 引用入口。职责不重叠。

---

## 7. 关键设计决策

| 决策 | 结论 | 理由 |
|---|---|---|
| 视图组织 | 合并视图(每 kind 一个区块,内含 OS 勾选 + APP 自建) | 一页看完,不跳转;用户脑模型是"musk 用了哪些" |
| OS 级在此页可编辑? | 不可(只读 + View + 复制为自建) | 避免两个编辑器冲突;OS 级的编辑归 Harness 模块 |
| 自建格式 | 与 OS 级完全一致(同 .at + sidecar .soul.md) | Phase 1 约束:不做定制,只继承或新建 |
| 引用清单存储 | config.at 的 harness { } 节点(已实现) | 已有,不另造 |
| 自建存储 | apps/musk/harness/<kind>/ 目录扫描 | 与 OS 级目录扫描一致,无需声明 |
| "复制为自建" | View 详情页提供按钮,复制到 APP 目录 | Phase 1 的"定制"替代:显式复制,无歧义 |
| 来源徽章 | 🟦 OS / 🟩 APP / 🔒 内置 | 一眼区分来源 |

---

## 8. 实施路线(确认设计后)

| 阶段 | 内容 | 依赖 |
|---|---|---|
| 1 | 后端:`/api/app-harness/{kind}` 端点(OS 清单 + APP 自建扫描) | apps/musk/harness/ 目录约定 |
| 2 | 后端:APP 自建 CRUD(PUT/DELETE + .at 写出,复用 serialize_at_role) | RoleRegistry 的 app 目录扩展 |
| 3 | 前端:app-config-page 的 Harness 区改为合并视图(OS 勾选 + APP 自建 CRUD) | API 就绪 |
| 4 | 前端:"复制为自建"按钮 + View 详情抽屉 | — |
| 5 | 验证:Playwright(勾选引用 + 自建 CRUD + 徽章) | — |

---

## 9. 开放问题

- **OS 级被删除后**:如果 musk 引用了 coder,但 coder 从 OS 级被删了,引用清单里的 coder 变"悬空"——显示警告?还是静默跳过?(建议:警告 + 自动取消勾选)
- **skills/modes 的"复制为自建"**:roles 有 inherit 机制;skills 是纯目录(无 inherit);modes 是 .at(有 inherit?)——每 kind 的复制语义可能不同
- **多 APP 扩展**:第二个 APP(forge)出现时,目录自然扩展到 `apps/forge/harness/`,这套方案零改动
