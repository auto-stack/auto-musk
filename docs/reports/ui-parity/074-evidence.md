# PLAN-074 evidence ledger

Generated: 2026-09-19T07:53:58.827Z

## Static gates

- node scripts/ui-parity.mjs check: **PASS** (62 declarations; 57 effective cases).
- Source inventory rejects symlinks and records a SHA-256 hash for every production `.at` file.
- Fixture materialization copies production source byte-for-byte and records adapter hashes in materialized.json.

## Runtime gates

- vm / chat-message-pair: **snapshot-ok**; evidence=runtime-smoke; stdout=b393a20c498fc7b90fd7eed85940f2d3a8c13a12470f66cd08d6459f8707eca4; stderr=6ce63d58226529c951a01b9a4b9e77cf1154c05819b7141d4b689ff97e79abf9.
- vue / chat-message-pair: **missing:Timed out waiting for http://127.0.0.1:17474**; evidence=missing-runtime-evidence; stdout=eeb0f513338c2ade62c928e0ac842309cc931396d9143cca57cc0c6569cd6b31; stderr=71b46d29857a2dbc72edf611e6bf94c468689d07ce0dd82836cbbb09169849ca.
- No screenshot or layout evidence is recorded until both renderers produce a stable gallery surface.

## Ownership

- VM handler/codegen and AutoUI MCP runtime failures: auto-lang / the VM responsibility in the next parity plan.
- Markdown/AutoDown rendering and editor behavior: PLAN-076 via autodown-engine.
- Think/tool/gate message behavior: PLAN-077.
- Default style contract: PLAN-075.
- App shell and release gate aggregation: PLAN-078/079.
