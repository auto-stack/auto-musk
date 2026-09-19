# PLAN-074 evidence ledger

Generated: 2026-09-19T09:18:50.690Z

## Static gates

- node scripts/ui-parity.mjs check: **PASS** (62 declarations; 57 effective cases).
- Source inventory rejects symlinks and records a SHA-256 hash for every production `.at` file.
- Fixture materialization copies production source byte-for-byte and records adapter hashes in materialized.json.

## Runtime gates

- vm / chat-message-pair: **snapshot-ok**; evidence=runtime-smoke; stdout=536b8a5b7c40508bae04a0812ba896a2da2d7172ca0110d8e76884e7d2903dac; stderr=0e911ba69e5025c6cd868cfddcfbec1164e483778cfc880d30221706411253ff.
- vue / chat-message-pair: **http-ok**; evidence=runtime-smoke; stdout=58c3e6e5f1d318ce70c31e9c0b6e3c4dd1551cc1e3dae0bac153546792b5aed3; stderr=26737fcc4ab62765b587b83efedbbebdee89f7e6117342a26ab31fb90aff240c.
- vm reset/event spy: **PASS**; screenshot=saved.
- Dual-mode runtime smoke established: VM produces rendered snapshot + reset event spy + baseline screenshot; Vue dev server and AutoVM backend produce stable http-ok endpoint.
- Dual-mode visual diff and deep interaction parity gate: scheduled across PLAN-075 (styles/geometry) and PLAN-076 (AutoDown engine).

## Ownership

- VM handler/codegen and AutoUI MCP runtime failures: auto-lang / the VM responsibility in the next parity plan.
- Markdown/AutoDown rendering and editor behavior: PLAN-076 via autodown-engine.
- Think/tool/gate message behavior: PLAN-077.
- Default style contract: PLAN-075.
- App shell and release gate aggregation: PLAN-078/079.
