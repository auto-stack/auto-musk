# PLAN-074 evidence ledger

Generated: 2026-09-19T07:47:13.584Z

## Static gates

- node scripts/ui-parity.mjs check: **PASS** (62 declarations; 57 effective cases).
- Source inventory rejects symlinks and records a SHA-256 hash for every production `.at` file.
- Fixture materialization copies production source byte-for-byte and records adapter hashes in materialized.json.

## Runtime gates

- vm / chat-message-pair: **startup-failed**; evidence=missing-runtime-evidence; stdout=ad3f68a2fe989325e4d34c04b64a1f4971a96cb40647d7e96f17b327fc10c635; stderr=eb2b6f8441bf3c231312f3d3b63fe89b9507d1243526ef37498c786bc170ca81.
- vue / chat-message-pair: **missing:Timed out waiting for http://127.0.0.1:17474**; evidence=missing-runtime-evidence; stdout=f3b52d4fabd34cba5ff106d9850990e23790fc45ac3b4af81d81380fe098f899; stderr=af16dd5a352d26ebb05f982176f225e13cfcf1401f89675a8c451235705f218f.
- No screenshot or layout evidence is recorded until both renderers produce a stable gallery surface.

## Ownership

- VM handler/codegen and AutoUI MCP runtime failures: auto-lang / the VM responsibility in the next parity plan.
- Markdown/AutoDown rendering and editor behavior: PLAN-076 via autodown-engine.
- Think/tool/gate message behavior: PLAN-077.
- Default style contract: PLAN-075.
- App shell and release gate aggregation: PLAN-078/079.
