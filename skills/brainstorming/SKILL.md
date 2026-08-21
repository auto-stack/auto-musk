---
name: brainstorming
description: You MUST use this before any creative work — adding a feature, building a component, or changing behavior. Explores intent, requirements, and design with the user before any code is written.
---


> **Plan-driven note (PLAN-030):** for feature work, prefer folding the
> brainstorm outcome into a numbered plan file (`docs/plans/NNN-*.md`) via the
> `plan-driven-development` skill / `create_plan` tool, instead of writing a
> separate design doc.

# Brainstorming

You are about to start creative work. **Do not write or edit any code yet.** First explore the user's intent and the design space with them.

## Process

1. **Understand the context.** Use `read_file`, `search`, and `run_command` to read the relevant existing code and docs. Don't guess at what's there — look.

2. **Ask clarifying questions — one at a time.** Surface the most important unknown first; wait for the answer before asking the next. Prefer 1-3 sharp questions over a wall of text.

   **Always ask via the structured questionnaire format** so the chat UI renders clickable options instead of making the user type. End your message with a fenced JSON block:

   `````text
   ```json
   {"type": "questionnaire", "questions": [
     {"id": "q1", "text": "目标用户和场景是什么？", "type": "single", "options": ["小团队内部知识库", "实时协作白板", "个人笔记+轻量分享"], "otherLabel": "其他：", "optional": false},
     {"id": "q2", "text": "需要哪些必选能力？（可多选）", "type": "multiple", "options": ["富文本编辑", "全文搜索", "权限管理"], "optional": true}
   ]}
   ```
   ````

   Rules for the questionnaire block:
   - `type`: `"single"` (radio) | `"multiple"` (checkbox) | `"text"` (free input — use only when options are truly impossible).
   - Provide 2-5 concrete, mutually exclusive options; add `otherLabel` when a custom answer is plausible.
   - Mark secondary questions `"optional": true`. The user submits answers with one click; they arrive back as `Q1: …; Q2: …`.
   - Keep the prose before the block short — the questions themselves carry the context.

3. **Propose 2-3 approaches.** Briefly compare trade-offs (simplicity vs. performance vs. extensibility). Let the user choose — this is also a good place to use a `"single"`-type questionnaire block with the approaches as options.

4. **Draft the design.** Once the approach is chosen, write a short design doc covering: goal, the approach, key files/changes, and open questions. Save it to `docs/specs/<topic>-design.md` using `write_file`.

5. **Get approval.** Present the design doc and explicitly ask "shall I proceed to a plan?" Do not start planning or coding until the user approves.

## Rules

- **No implementation in this phase.** `edit_file` / `write_file` (except the design doc) / `run_command` that changes state are forbidden here.
- Keep the design doc short — a page or less. Detail belongs in the plan.
- If the request is genuinely ambiguous in a way that changes the whole approach, ask; otherwise pick the most likely interpretation and note the assumption.
- The terminal state of this skill is the user approving the design, after which you invoke the **writing-plans** skill.

## Output

A saved design doc at `docs/specs/<topic>-design.md` and an explicit approval gate. Then say: "Design approved — I'll invoke the writing-plans skill to turn this into an implementation plan."
