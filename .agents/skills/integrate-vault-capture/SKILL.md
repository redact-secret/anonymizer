---
name: integrate-vault-capture
description: "Design or implement anonymizer reversible token placement through a minimal vault-facing capture contract, including abort and rollback conformance; use for issue #7."
---

# Integrate vault capture

Read `AGENTS.md`, authoritative documents, issue #7 and epic #1, the accepted replacement-plan contract, and the selected vault public contract/version. Do not infer that a vault capability exists from a proposed owner or issue text.

Keep anonymizer responsible only for accepted slices, capture submission, receiving opaque tokens, placing replacements, and aborting failed transformations. Vault owns token identity, mappings, lifecycle, budgets, policy, persistence, crypto, and keys. Token possession and model claims do not establish authorization.

Prefer bulk capture through a small removable integration. Define begin/capture/commit/abort behavior as supported by the actual vault contract; do not promise atomicity without it. Specify failures during any capture, invalid returned tokens, output-capacity failure, commit failure, cancellation where supported, and abort failure. Preflight local validation before side effects where practical.

Return no partially transformed user-visible output. Document how partially committed mappings are rolled back or cleaned up and what residual state remains if cleanup fails. Retain no independent token-to-value map and put neither mappings nor authority state in the manifest. Treat tokens as potentially sensitive and sanitize sink error propagation.

Use a synthetic dummy sink for failure injection and selected real contract conformance where available. Verify success, capture-stage failure, cleanup, safe diagnostics, and the default build's exclusion of vault persistence dependencies. Report unverified authority and rollback guarantees explicitly; do not add restoration or a credential store.
