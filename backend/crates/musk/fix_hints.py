p = "src/plan_delivery.rs"
s = open(p, encoding="utf-8").read()
pairs = [
  ('Ok(json!({"checkpoint": "prepared", "delivery_commit": delivery_commit, "files": spec_files}))',
   'Ok(json!({"checkpoint": "prepared", "delivery_commit": delivery_commit, "files": spec_files, "next_action": "land"}))'),
  ('Ok(json!({"checkpoint": "prepared", "delivery_commit": commit, "idempotent": true}))',
   'Ok(json!({"checkpoint": "prepared", "delivery_commit": commit, "idempotent": true, "next_action": "land"}))'),
  ('Ok(json!({"checkpoint": "landed", "delivery_commit": new_head, "old_commit": old_head}))',
   'Ok(json!({"checkpoint": "landed", "delivery_commit": new_head, "old_commit": old_head, "next_action": "refresh"}))'),
  ('Ok(json!({"checkpoint": "landed", "delivery_commit": commit, "idempotent": true}))',
   'Ok(json!({"checkpoint": "landed", "delivery_commit": commit, "idempotent": true, "next_action": "refresh"}))'),
  ('Ok(json!({"checkpoint": "ledger_refreshed", "targets": touched}))',
   'Ok(json!({"checkpoint": "ledger_refreshed", "targets": touched, "next_action": "archive"}))'),
  ('Ok(json!({"checkpoint": "archived", "completion_kind": "delivered", "plan_path": format!("docs/plans/archived/{}", pf.filename)}))',
   'Ok(json!({"checkpoint": "archived", "completion_kind": "delivered", "plan_path": format!("docs/plans/archived/{}", pf.filename), "next_action": "cleanup"}))'),
  ('Ok(json!({"checkpoint": "archived", "idempotent": true, "completion_kind": "delivered"}))',
   'Ok(json!({"checkpoint": "archived", "idempotent": true, "completion_kind": "delivered", "next_action": "cleanup"}))'),
  ('Ok(json!({"checkpoint": "cleaned"}))',
   'Ok(json!({"checkpoint": "cleaned", "next_action": "submit complete_plan_stage(document, pass) — all five checkpoints settled"}))'),
  ('Ok(json!({"checkpoint": "cleaned", "idempotent": true}))',
   'Ok(json!({"checkpoint": "cleaned", "idempotent": true, "next_action": "submit complete_plan_stage(document, pass)"}))'),
]
applied = 0
for old, new in pairs:
    if old in s:
        s = s.replace(old, new, 1)
        applied += 1
open(p, "w", encoding="utf-8", newline="\n").write(s)
print(f"delivery hints applied {applied}/{len(pairs)}")

p = "src/relay/plan_control.rs"
s = open(p, encoding="utf-8").read()
old = '"delivery checkpoints missing: {missing:?} — run plan_delivery actions first"'
new = ('"delivery checkpoints missing: {missing:?} — call plan_delivery with the FIRST missing '
       'action in order (prepare/land/refresh/archive/cleanup, one call per checkpoint); '
       'do NOT re-call an action that already returned its checkpoint"')
assert old in s, "doc gate in plan_control"
s = s.replace(old, new, 1)
open(p, "w", encoding="utf-8", newline="\n").write(s)
print("doc gate hint ok")

p = "scripts/plan-flow-probe.mjs"
s = open(p, encoding="utf-8").read()
old = "且测试套件覆盖零值用例 add(0,0) === 0（复审将重验测试文件本身，缺零值用例即 fail）。"
new = "且测试文件必须包含字面断言行 `assert.strictEqual(add(0, 0), 0);`（复审逐字重验测试文件，缺该行即 fail）。"
assert old in s, "ac wording"
s = s.replace(old, new, 1)
open(p, "w", encoding="utf-8", newline="\n").write(s)
print("fixture ok")
