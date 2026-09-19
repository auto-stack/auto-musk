# UI parity fixtures

Fixtures are deterministic input contracts for the `musk-widgets-gallery`. They describe messages, block states, and expected side-effect boundaries. They do not contain screenshots or copied production templates.

The gallery materializer validates the selected fixture, mounts production `.at` sources, and uses an in-memory store plus instrumented API stubs. The store adapter only records expansion/cancel events; it does not render, transform Markdown, or call a real backend. Keep new fixtures free of secrets, machine paths, current time, random IDs, network URLs, and commands with write effects.

Each new component case must name its owning plan and state contract in `cases.json`. The generated inventory cases cover reachable declarations that do not yet have an explicit interaction case, so adding a widget cannot silently bypass the parity ledger.
