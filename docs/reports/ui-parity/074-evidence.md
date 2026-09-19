# PLAN-074 evidence ledger

Generated: 2026-09-19T09:47:11.417Z

## Static gates

- node scripts/ui-parity.mjs check: **PASS** (62 declarations; 57 effective cases).
- Source inventory rejects symlinks and records a SHA-256 hash for every production `.at` file.
- Fixture materialization copies production source byte-for-byte and records adapter hashes in materialized.json.

## Runtime gates

- vm / chat-message-pair: **snapshot-ok**; evidence=runtime-smoke; stdout=3a10fc98ddeacf97def4d0a7ca0d93e02715d12b2300775bf464e24ee5de5f3a; stderr=0e911ba69e5025c6cd868cfddcfbec1164e483778cfc880d30221706411253ff.
- vue / chat-message-pair: **http-ok**; evidence=runtime-smoke; stdout=665df6f1c2ee64a1f00328d24ecea93f6d559b76f72b3cc3a247d2ff5feccb80; stderr=6df01b111c8593221e13a7b6418a145588ded4365970066884dcecae3c7f589f.
- vm reset/event spy: **PASS**; screenshot=saved.
- Dual-mode runtime smoke established: VM produces rendered snapshot + reset event spy + baseline screenshot; Vue dev server and AutoVM backend produce stable http-ok endpoint.
- Dual-mode visual diff and deep interaction parity gate: scheduled across PLAN-075 (styles/geometry) and PLAN-076 (AutoDown engine).

## Ownership

- VM handler/codegen and AutoUI MCP runtime failures: auto-lang / the VM responsibility in the next parity plan.
- Markdown/AutoDown rendering and editor behavior: PLAN-076 via autodown-engine.
- Think/tool/gate message behavior: PLAN-077.
- Default style contract: PLAN-075.
- App shell and release gate aggregation: PLAN-078/079.
