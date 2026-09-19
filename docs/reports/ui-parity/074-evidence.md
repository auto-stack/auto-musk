# PLAN-074 evidence ledger

Generated: 2026-09-19T08:00:11.984Z

## Static gates

- node scripts/ui-parity.mjs check: **PASS** (62 declarations; 57 effective cases).
- Source inventory rejects symlinks and records a SHA-256 hash for every production `.at` file.
- Fixture materialization copies production source byte-for-byte and records adapter hashes in materialized.json.

## Runtime gates

- vm / chat-message-pair: **snapshot-ok**; evidence=runtime-smoke; stdout=08004efb9fd490c73d22c99dd5aceb74fd352226f4a1198472366eb1337b90c2; stderr=0e911ba69e5025c6cd868cfddcfbec1164e483778cfc880d30221706411253ff.
- vue / chat-message-pair: **missing:Timed out waiting for http://127.0.0.1:17474**; evidence=missing-runtime-evidence; stdout=eeb0f513338c2ade62c928e0ac842309cc931396d9143cca57cc0c6569cd6b31; stderr=71b46d29857a2dbc72edf611e6bf94c468689d07ce0dd82836cbbb09169849ca.
- vm reset/event spy: **FAIL**; screenshot=saved.
- No screenshot or layout evidence is recorded until both renderers produce a stable gallery surface.

## Ownership

- VM handler/codegen and AutoUI MCP runtime failures: auto-lang / the VM responsibility in the next parity plan.
- Markdown/AutoDown rendering and editor behavior: PLAN-076 via autodown-engine.
- Think/tool/gate message behavior: PLAN-077.
- Default style contract: PLAN-075.
- App shell and release gate aggregation: PLAN-078/079.
