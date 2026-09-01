# seed_blocks.py — Block 全家福种子数据（chats + 镜像 conversation + report html 落盘；
# relay run 经 POST /api/dev/seed-run 注入内存 store——RunStore 不落盘）。
# 用法: python scripts/seed_blocks.py <workspace_root> [--api http://127.0.0.1:9080] [--dry]
# 先停 serve 落盘文件，起 serve 后重跑一次完成 run 注入（或直接带 --api 一次完成：
# 脚本会自动"没起就先只落盘"）。
import json
import os
import sys
import time
import urllib.request

T = int(time.time()) - 300
ARGS = sys.argv[1:]
WS_ID = ARGS[0] if ARGS and not ARGS[0].startswith("--") else r"D:\autostack\auto-musk\backend"
API = ARGS[ARGS.index("--api") + 1] if "--api" in ARGS else None
DRY = "--dry" in ARGS

CHAT_ID = "block-showcase-chat"
RUN_ID = "run-block-showcase"

# ─────────────────────────── chat 会话 ───────────────────────────

MD_TEXT = """## 文本块（Markdown 全谱）

这是 **Auto/vue 版**聊天消息里的 `text` block——支持 *行内样式*、[链接](https://example.com)、行内代码 `Http.post`，以及下面这些富形态：

### 列表

1. 有序列表项一
2. 有序列表项二
   - 嵌套无序项
   - 嵌套无序项二

> 引用块：ThinkBlock 默认折叠，按 token 估算展示。

### 表格

| Block 类型 | 触发方式 | 呈现组件 |
|---|---|---|
| 文本块 | assistant 正文 | Markdown（markstream） |
| 思考块 | msg.thinking | ThinkBlock |
| 工具卡 | tool_calls | ToolBlock → 分发 |
| 问卷卡 | 问卷 JSON（末条） | QuestionnaireCard |
| Run 窗口 | spawn_relay 工具 | RelayRunBox |
| 报告卡 | report 工具 / run 完成 | ReportCard |

### 代码块

```rust
fn ws_pick() Value {
    try {
        let data = Http.post("/api/workspace/pick", {})
        return data.path ?? None
    } catch {
        return None
    }
}
```

```ts
const blocks = await messageDisplayBlocks(msg, streaming);
// text → 思考 → 工具，按到达顺序渲染，块不删除
for (const b of blocks) render(b);
```

---"""

THINKING = """用户要看全部 Block 类型。规划一下展示顺序：
1. 先来一段 markdown 富文本（列表/表格/代码）证明文本块；
2. 工具卡按 GenericToolCard 的几个形态各来一条；
3. spawn_relay 嵌一个真正的 Run 窗口（chat 里内嵌）；
4. report 工具渲染 ReportCard；
5. 最后一条消息放问卷 JSON，触发 QuestionnaireCard。
注意问卷卡只随最新一条 assistant 消息显示，所以必须放最后。"""

TOOL_CALLS = [
    {
        "name": "run_command",
        "arguments": {"cmd": "cargo test -p musk --lib workspace"},
        "result": "running 16 tests\ntest workspace::tests::loads_with_default_workspace ... ok\ntest workspace::tests::open_new_workspace_assigns_dirname_id ... ok\ntest workspace::tests::open_duplicate_path_reuses_id ... ok\n\ntest result: ok. 16 passed; 0 failed; 0 ignored",
        "status": "success",
        "id": "tc-1",
    },
    {
        "name": "read_file",
        "arguments": {"path": "src/front/workspace_selector.at", "limit": 40},
        "result": "// workspace_selector.at — WorkspaceSelector 组件\n// 面板宽度适配内容 + 搜索框 + 当前项打勾\nwidget WorkspaceSelector() {\n    computed { ... }\n    msg Msg { Toggle, Choose(str), Close, UpdateQuery(str), PickFolder }\n    ...",
        "status": "success",
        "id": "tc-2",
    },
    {
        "name": "edit_file",
        "arguments": {
            "path": "src/front/workspace_selector.at",
            "old_string": "width: max-content; min-width: 100%; max-width: min(26rem, 90vw);",
            "new_string": "width: max-content; min-width: 100%; max-width: min(24rem, 90vw);",
        },
        "result": "patched 1 hunk successfully",
        "status": "success",
        "id": "tc-3",
    },
    {
        "name": "write_file",
        "arguments": {
            "path": "examples/block-showcase/README.md",
            "content": "# Block Showcase\n\n- cmd（run_command）\n- diff（edit_file）\n- write（write_file）\n- file（read_file）\n- listing（list_dir）\n- doc（read_plan / 长结果）\n- generic（其余工具）\n",
        },
        "result": "wrote 9 lines to examples/block-showcase/README.md",
        "status": "success",
        "id": "tc-4",
    },
    {
        "name": "list_dir",
        "arguments": {"path": "src/front/ports"},
        "result": "composables.vm.at <file 1123>\ncomposables.web.at <file 2044>\nicons.web.at <file 3120>\nplatform.vm.at <file 780>\nplatform.web.at <file 640>\nrenderer.vm.at <file 890>\nrenderer.web.at <file 1230>\nupload.web.at <file 455>",
        "status": "success",
        "id": "tc-5",
    },
    {
        "name": "spawn_relay",
        "arguments": {"run_id": RUN_ID, "task": "Block 全家福 Run——演示 Run 窗口全部块型"},
        "result": json.dumps({"run_id": RUN_ID, "status": "completed"}, ensure_ascii=False),
        "status": "success",
        "id": "tc-6",
    },
    {
        "name": "report",
        "arguments": {
            "run_id": RUN_ID,
            "title": "Block 全家福汇报",
            "summary": "## 汇总\n\n- 全部 **工具视图形态**已演示：cmd / diff / write / file / listing / doc / generic\n- 测试 **16/16** 全绿\n- 交付物见下方 chips，点击可打开",
            "goals_met": "3/3",
            "tests_pass": "16",
            "drift_detected": "None",
            "cost": "12345",
            "confidence": "High",
            "deliverables": [
                "examples/block-showcase/README.md",
                ".autoos/reports/run-block-showcase/report.html",
            ],
            "files_changed": ["src/front/workspace_selector.at"],
            "tool_calls": 6,
            "duration_s": 90,
            "completed_steps": 3,
            "total_steps": 3,
        },
        "result": "",
        "status": "success",
        "id": "tc-7",
    },
]

QUESTIONNAIRE_TEXT = """前面的块都演示完了。最后是**问卷 Block**（QuestionnaireCard）——它只挂在最新一条 assistant 消息上，提交后会折叠成摘要。

```json
{
  "type": "questionnaire",
  "questions": [
    {
      "id": "q1",
      "text": "哪种 Block 类型对你最有用？",
      "type": "single",
      "options": ["文本/表格/代码", "思考块", "工具卡", "Run 窗口", "报告卡"]
    },
    {
      "id": "q2",
      "text": "还希望补充哪些演示？（可多选）",
      "type": "multiple",
      "options": ["Deck 幻灯片层", "Errand 卡片", "TaskPlan 卡片", "Gate 审批流"]
    },
    {
      "id": "q3",
      "text": "其他建议",
      "type": "text",
      "placeholder": "随便写点什么…"
    }
  ]
}
```"""


def chat_session():
    return {
        "id": CHAT_ID,
        "name": "Block 全家福演示",
        "mode": "superpowers",
        "messages": [
            {
                "id": "bsmsg-u1",
                "role": "user",
                "content": "请把 Auto/vue 版支持的所有 Block 类型都演示一遍：Markdown 文本/表格/代码、思考块、各类工具卡、内嵌 Run 窗口、完成报告卡，最后再给我一份可交互问卷。",
                "created_at": T,
            },
            {
                "id": "bsmsg-a1",
                "role": "assistant",
                "thinking": THINKING,
                "content": MD_TEXT,
                "tool_calls": TOOL_CALLS,
                "created_at": T + 20,
            },
            {
                "id": "bsmsg-a2",
                "role": "assistant",
                "content": QUESTIONNAIRE_TEXT,
                "created_at": T + 60,
            },
        ],
        "created_at": T,
        "updated_at": T + 70,
        "workspace_id": "",
    }


# ─────────────────────────── relay run ───────────────────────────

RUN_EVENTS = [
    {"type": "step_started", "timestamp": T, "step_id": "plan", "role_id": "planner"},
    {"type": "relay_update", "timestamp": T + 1, "step_id": "plan", "role_id": "planner", "status": "started"},
    {"type": "turn_delta", "timestamp": T + 3, "role_id": "planner", "text": "## 计划阶段\n\n我将按顺序演示 **Run 窗口**的全部块型：\n\n1. 文本输出（本段与下一段会合并为一条）\n2. 工具卡七形态：cmd / diff / write / file / listing / doc / generic\n3. 错误与预算告警条\n4. 审批门（Gate）\n5. 完成报告卡（ReportCard）\n\n| 块型 | 触发工具 | 展开视图 |\n|---|---|---|\n| cmd | run_command | 终端块 |\n| diff | edit_file | 左右对照 |\n| file | read_file | 参数 chips + 正文 |\n| listing | list_dir | 文件清单 |"},
    {"type": "turn_tool_call", "timestamp": T + 8, "role_id": "planner", "tool_id": "t1", "tool_name": "run_command", "arguments": {"cmd": "cargo test -p musk --lib workspace"}},
    {"type": "turn_tool_result", "timestamp": T + 12, "role_id": "planner", "tool_id": "t1", "result": "running 16 tests\ntest workspace::tests::loads_with_default_workspace ... ok\ntest workspace::tests::open_new_workspace_assigns_dirname_id ... ok\n\ntest result: ok. 16 passed; 0 failed; 0 ignored"},
    {"type": "turn_tool_call", "timestamp": T + 14, "role_id": "planner", "tool_id": "t2", "tool_name": "edit_file", "arguments": {"path": "src/front/workspace_selector.at", "old_string": "max-width: min(26rem, 90vw);", "new_string": "max-width: min(24rem, 90vw);"}},
    {"type": "turn_tool_result", "timestamp": T + 16, "role_id": "planner", "tool_id": "t2", "result": "1 hunk patched"},
    {"type": "turn_tool_call", "timestamp": T + 18, "role_id": "planner", "tool_id": "t3", "tool_name": "write_file", "arguments": {"path": "examples/block-showcase/README.md", "content": "# Block Showcase\n\n- cmd（run_command）\n- diff（edit_file）\n- write（write_file）\n"}},
    {"type": "turn_tool_result", "timestamp": T + 20, "role_id": "planner", "tool_id": "t3", "result": "wrote 5 lines"},
    {"type": "turn_tool_call", "timestamp": T + 22, "role_id": "planner", "tool_id": "t4", "tool_name": "list_dir", "arguments": {"path": "src/front/ports"}},
    {"type": "turn_tool_result", "timestamp": T + 24, "role_id": "planner", "tool_id": "t4", "result": "composables.vm.at <file 1123>\ncomposables.web.at <file 2044>\nicons.web.at <file 3120>\nplatform.vm.at <file 780>\nplatform.web.at <file 640>"},
    {"type": "turn_tool_call", "timestamp": T + 26, "role_id": "planner", "tool_id": "t5", "tool_name": "read_file", "arguments": {"path": "README.md", "limit": 80}},
    {"type": "turn_tool_result", "timestamp": T + 30, "role_id": "planner", "tool_id": "t5", "result": "# auto-musk — Forge-successor AI coding agent\n\nAuto/vue 版前端由 gen/front/vue 提供，musk serve 直接服务 dist。\n\n聊天侧块模型：文本/思考/工具按到达顺序渲染；Run 窗口由 relay run 事件驱动，\n工具展开视图按工具名分七种形态；完成后 RunReportPayload 驱动 PPT 风格报告卡。\n本结果超过 400 字符，因此走 doc（文档）视图而非 file 视图——这也是块分派的\n规则之一：读类工具或长结果一律按 Markdown 小字号文档渲染。" * 2},
    {"type": "turn_tool_call", "timestamp": T + 32, "role_id": "planner", "tool_id": "t6", "tool_name": "web_search", "arguments": {"query": "auto-lang widget 转译规则"}},
    {"type": "turn_tool_result", "timestamp": T + 34, "role_id": "planner", "tool_id": "t6", "result": "ok"},
    {"type": "turn_error", "timestamp": T + 36, "role_id": "planner", "message": "web_search rate limited — 已降级为本地索引"},
    {"type": "turn_budget_warning", "timestamp": T + 38, "role_id": "planner", "remaining": 100000},
    {"type": "step_completed", "timestamp": T + 40, "step_id": "plan", "handoff_summary": "计划完成：文本/工具/错误/预算块全部落地，进入执行阶段。"},
    {"type": "step_started", "timestamp": T + 42, "step_id": "execute", "role_id": "coder"},
    {"type": "turn_delta", "timestamp": T + 44, "role_id": "coder", "text": "执行阶段：改造完成，全量测试绿，等待人工审批。"},
    {"type": "gate_waiting", "timestamp": T + 46, "step_id": "execute", "gate": "human"},
    {"type": "gate_resolved", "timestamp": T + 50, "step_id": "execute", "decision": "approve"},
    {"type": "step_started", "timestamp": T + 52, "step_id": "document", "role_id": "documenter"},
    {"type": "turn_delta", "timestamp": T + 54, "role_id": "documenter", "text": "文档阶段：生成汇报报告与交付物清单。"},
    {"type": "report_emitted", "timestamp": T + 58, "format": "html", "title": "Block 全家福汇报", "path": ".autoos/reports/run-block-showcase/report.html"},
    {"type": "token_spend", "timestamp": T + 60, "cumulative": 12345, "step_tokens": 4321},
    {
        "type": "run_completed",
        "timestamp": T + 90,
        "report": {
            "run_id": RUN_ID,
            "title": "Block 全家福汇报",
            "summary": "## 汇总\n\n- 全部 **7 类工具视图**已演示：cmd / diff / write / file / listing / doc / generic\n- 测试 **16/16** 全绿，漂移 None\n- 报告本体与交付物见下方 chips",
            "goals_met": "3/3",
            "tests_pass": "16",
            "drift_detected": "None",
            "cost": "12345",
            "confidence": "High",
            "deliverables": [
                "examples/block-showcase/README.md",
                ".autoos/reports/run-block-showcase/report.html",
            ],
            "files_changed": ["src/front/workspace_selector.at"],
            "tool_calls": 6,
            "duration_s": 90,
            "completed_steps": 3,
            "total_steps": 3,
            "report": {
                "format": "html",
                "title": "Block 全家福汇报",
                "path": ".autoos/reports/run-block-showcase/report.html",
                "structured": None,
            },
        },
    },
]


def handoff(frm, to, summary):
    return {
        "from": frm,
        "to": to,
        "target": to,
        "summary": summary,
        "decisions": [],
        "open_questions": [],
        "work_product": [],
        "context_for_next": {"files_to_read": [], "files_to_test": [], "warnings": []},
        "token_usage": {"step_tokens": 1000, "cumulative": 12345, "budget_remaining": 9998765},
    }


RUN_JSON = {
    "run_id": RUN_ID,
    "engine": {
        "flow": {
            "id": "block-showcase",
            "steps": [
                {"id": "plan", "profession_id": "planner", "role_id": "planner", "agent_config_id": None, "gate": "auto", "max_turns": None, "exit": "next"},
                {"id": "execute", "profession_id": "coder", "role_id": "coder", "agent_config_id": None, "gate": "human", "max_turns": None, "exit": "next"},
                {"id": "document", "profession_id": "documenter", "role_id": "documenter", "agent_config_id": None, "gate": "auto", "max_turns": None, "exit": "next"},
            ],
        },
        "current_step": 3,
        "status": {"status": "completed"},
        "run_id": RUN_ID,
        "step_history": [
            {"step_id": "plan", "role_id": "planner", "started_at": T, "completed_at": T + 40, "iteration": 1, "handoff": handoff("planner", "coder", "计划完成：全部块型已落地。")},
            {"step_id": "execute", "role_id": "coder", "started_at": T + 42, "completed_at": T + 60, "iteration": 1, "handoff": handoff("coder", "documenter", "执行完成：测试 16/16 绿。")},
            {"step_id": "document", "role_id": "documenter", "started_at": T + 62, "completed_at": T + 90, "iteration": 1, "handoff": handoff("documenter", "coder", "汇报报告已生成。")},
        ],
        "loop_counters": {},
        "pending_gate": None,
        "gate_feedback": {},
        "gate_resolved_for_step": None,
        "resumed_step_id": None,
        "cumulative_tokens": 12345,
        "budget_tracker": {"run_budget": {"limit": 10000000, "warning_at": 7000000, "strategy": "HardStop"}, "step_budgets": {}, "cumulative": 0, "per_step": {}},
        "mode": "auto",
    },
    "created_at": T,
    "updated_at": T + 90,
    "events": RUN_EVENTS,
    "metadata": {"title": "Block 全家福 Run", "initial_task": "演示 Run 窗口全部块型", "originating_chat_session": CHAT_ID},
}

_seq = 0
_tid = 0


def _turn(frm, kind, content="", tool=None, gate=None, ts=T):
    global _seq, _tid
    t = {"id": f"bs{_tid:06x}", "seq": _seq, "from": frm, "kind": kind, "content": content, "timestamp": ts}
    _tid += 1
    _seq += 1
    if tool is not None:
        t["tool"] = tool
    if gate is not None:
        t["gate"] = gate
    return t


def tool_pair(ev_call, ev_result):
    return [
        _turn(ev_call["role_id"], "tool_call", "", tool={"name": ev_call["tool_name"], "args": ev_call["arguments"], "result": ""}, ts=ev_call["timestamp"]),
        _turn("system", "tool_result", "", tool={"name": "", "args": None, "result": ev_result["result"]}, ts=ev_result["timestamp"]),
    ]


CONVERSATION_TURNS = []
CONVERSATION_TURNS.append(_turn("planner", "system", "Step 'plan' started (planner)", ts=T))
CONVERSATION_TURNS.append(_turn("planner", "system", "Step 'plan' started", ts=T + 1))
CONVERSATION_TURNS.append(_turn("planner", "message", RUN_EVENTS[2]["text"], ts=T + 3))
for i in range(3, 15, 2):
    CONVERSATION_TURNS.extend(tool_pair(RUN_EVENTS[i], RUN_EVENTS[i + 1]))
CONVERSATION_TURNS.append(_turn("system", "system", "Error: web_search rate limited — 已降级为本地索引", ts=T + 36))
CONVERSATION_TURNS.append(_turn("system", "system", "Budget warning: 100000 tokens remaining", ts=T + 38))
CONVERSATION_TURNS.append(_turn("system", "system", "Step 'plan' completed: 计划完成：文本/工具/错误/预算块全部落地，进入执行阶段。", ts=T + 40))
CONVERSATION_TURNS.append(_turn("coder", "system", "Step 'execute' started (coder)", ts=T + 42))
CONVERSATION_TURNS.append(_turn("coder", "message", RUN_EVENTS[19]["text"], ts=T + 44))
CONVERSATION_TURNS.append(_turn("system", "gate", "Waiting for gate approval: execute", gate={"step_id": "execute", "status": "waiting", "feedback": None}, ts=T + 46))
CONVERSATION_TURNS.append(_turn("human", "gate", "Gate execute approve", gate={"step_id": "execute", "status": "approve", "feedback": None}, ts=T + 50))
CONVERSATION_TURNS.append(_turn("documenter", "system", "Step 'document' started (documenter)", ts=T + 52))
CONVERSATION_TURNS.append(_turn("documenter", "message", RUN_EVENTS[23]["text"], ts=T + 54))
CONVERSATION_TURNS.append(_turn("system", "system", "汇报报告已生成：Block 全家福汇报（html，path：.autoos/reports/run-block-showcase/report.html）", ts=T + 58))
CONVERSATION_TURNS.append(_turn("system", "system", "Flow completed", ts=T + 90))

CONVERSATION_JSON = {
    "id": RUN_ID,
    "kind": "flow",
    "workspace_id": "",
    "driver": {"type": "flow", "flow_id": "block-showcase"},
    "status": "completed",
    "turns": CONVERSATION_TURNS,
    "title": "Block 全家福 Run",
    "mode": None,
    "flow_id": "block-showcase",
    "current_step": 3,
    "cumulative_tokens": 12345,
    "created_at": T,
    "updated_at": T + 90,
}

REPORT_HTML = """<!doctype html>
<html lang="zh"><head><meta charset="utf-8"><title>Block 全家福汇报</title>
<style>body{font-family:system-ui,sans-serif;max-width:760px;margin:2rem auto;padding:0 1rem;color:#e6e6e6;background:#161618}
h1{color:#8b7cf6}td,th{border:1px solid #333;padding:.35rem .6rem}table{border-collapse:collapse}</style></head>
<body><h1>Block 全家福汇报</h1><p>Run: run-block-showcase · 3/3 steps · 16/16 tests · confidence High</p>
<table><tr><th>视图</th><th>工具</th></tr><tr><td>cmd</td><td>run_command</td></tr><tr><td>diff</td><td>edit_file</td></tr>
<tr><td>write</td><td>write_file</td></tr><tr><td>file</td><td>read_file</td></tr><tr><td>listing</td><td>list_dir</td></tr>
<tr><td>doc</td><td>read_plan/长结果</td></tr><tr><td>generic</td><td>其余</td></tr></table></body></html>"""


def main():
    autoos = os.path.join(WS_ID, ".autoos")
    ws_name = os.path.basename(os.path.normpath(WS_ID))
    paths = {
        "chats": os.path.join(autoos, "chats.json"),
        "conv_dir": os.path.join(autoos, "conversations", RUN_ID),
        "report_dir": os.path.join(autoos, "reports", RUN_ID),
    }
    print(f"[seed] workspace: {WS_ID} (id={ws_name})")
    if DRY:
        print("[seed] dry run — 计划写入:")
        for k, v in paths.items():
            print(f"  {k}: {v}")
        return

    # 1) chats.json — 合并追加（保留已有会话）
    chats_path = paths["chats"]
    chats = {}
    if os.path.exists(chats_path):
        with open(chats_path, encoding="utf-8") as f:
            chats = json.load(f)
    chats[CHAT_ID] = chat_session()
    with open(chats_path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(chats, f, ensure_ascii=False, indent=2)
    print(f"[seed] chats.json += {CHAT_ID} (共 {len(chats)} 会话)")

    # 2) 镜像 conversation（与 run 同 id；turns 内嵌 meta.json——无 turns.jsonl 时生效）
    CONVERSATION_JSON["workspace_id"] = ws_name
    os.makedirs(paths["conv_dir"], exist_ok=True)
    with open(os.path.join(paths["conv_dir"], "meta.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(CONVERSATION_JSON, f, ensure_ascii=False, indent=2)
    print(f"[seed] conversations/{RUN_ID}/meta.json ({len(CONVERSATION_TURNS)} turns)")

    # 3) 报告 html（deck/报告本体）
    os.makedirs(paths["report_dir"], exist_ok=True)
    with open(os.path.join(paths["report_dir"], "report.html"), "w", encoding="utf-8", newline="\n") as f:
        f.write(REPORT_HTML)
    print(f"[seed] reports/{RUN_ID}/report.html")

    # 4) relay run 注入（内存 store；serve 需带 dev_seed 路由的新构建）
    if not API:
        print("[seed] 未给 --api，跳过 run 注入（起 serve 后重跑本脚本完成注入）")
        return
    body = json.dumps(RUN_JSON, ensure_ascii=False).encode("utf-8")
    url = f"{API.rstrip('/')}/api/dev/seed-run?workspace={ws_name}"
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            print(f"[seed] run 注入: {resp.status} {resp.read().decode('utf-8')}")
    except Exception as e:
        print(f"[seed] run 注入失败: {e}")


if __name__ == "__main__":
    main()
