---
name: brainstorming
description: You MUST use this before any creative work — adding a feature, building a component, or changing behavior. Explores intent, requirements, and design with the user before any code is written.
---

# Brainstorming

You are about to start creative work. **Do not write or edit any code yet.** First explore the user's intent and the design space with them.

## Process

1. **Understand the context.** Use `read_file`, `search`, and `run_command` to read the relevant existing code and docs. Don't guess at what's there — look.

2. **Ask clarifying questions — one at a time.** Surface the most important unknown first; wait for the answer before asking the next. Prefer 1-3 sharp questions over a wall of text.

3. **Propose 2-3 approaches.** Briefly compare trade-offs (simplicity vs. performance vs. extensibility). Let the user choose.

4. **Draft the design.** Once the approach is chosen, write a short design doc covering: goal, the approach, key files/changes, and open questions. Save it to `docs/specs/<topic>-design.md` using `write_file`.

5. **Get approval.** Present the design doc and explicitly ask "shall I proceed to a plan?" Do not start planning or coding until the user approves.

## Rules

- **No implementation in this phase.** `edit_file` / `write_file` (except the design doc) / `run_command` that changes state are forbidden here.
- Keep the design doc short — a page or less. Detail belongs in the plan.
- If the request is genuinely ambiguous in a way that changes the whole approach, ask; otherwise pick the most likely interpretation and note the assumption.
- The terminal state of this skill is the user approving the design, after which you invoke the **writing-plans** skill.

## Output

A saved design doc at `docs/specs/<topic>-design.md` and an explicit approval gate. Then say: "Design approved — I'll invoke the writing-plans skill to turn this into an implementation plan."
