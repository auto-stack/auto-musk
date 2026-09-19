# Musk widgets gallery

This gallery is the component-level parity harness for the Musk UI. It mounts the production AutoUI widget source in a small deterministic host so the Vue and VM renderers receive the same props, block states, and interaction sequence.

The gallery entry and copied source tree are generated for a case. They are deliberately ignored by Git: `scripts/ui-parity.mjs prepare` copies `src/front` and `vendor` from the Musk checkout, records SHA-256 hashes in `materialized.json`, and writes the case host to `src/front/app.at`. Production widget files are copied byte-for-byte. Only the host and explicit side-effect adapters are generated; the adapters keep API, stream, workspace, and store writes inside the gallery.

From the repository root:

```text
node scripts/ui-parity.mjs check
node scripts/ui-parity.mjs prepare --case chat-message-pair
cd examples/musk-widgets-gallery
auto run -r vue
auto run -r vm
```

Use `node scripts/ui-parity.mjs run --case chat-message-pair` for the bounded smoke runner and `node scripts/ui-parity.mjs report --plan 074` to write the current-state inventory. A missing VM snapshot, source drift, or missing screenshot is a failed evidence gate; the report must not be read as a parity pass.

Fixtures live in `tests/ui-parity/fixtures`. They contain no credentials, real commands, network targets, or production workspace paths. Reset is part of every case so a second ChatMessage instance cannot inherit expansion state from the first.
