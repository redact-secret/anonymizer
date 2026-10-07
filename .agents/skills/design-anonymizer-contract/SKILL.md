---
name: design-anonymizer-contract
description: "Design or review anonymizer finding, overlap arbitration, replacement-plan, and safe manifest contracts; use for issues #3\u2013#5 or API boundary decisions."
---

# Design anonymizer contract

Read `AGENTS.md`, the four authoritative documents, the relevant issue, and epic #1. Inspect existing decisions and public sibling contracts before proposing types. The README API and architecture policy axes are candidates, not settled contracts.

For finding contracts, define UTF-8 byte offsets, validation and invalid-input behavior, zero-length spans, duplicates, stable source/kind identity, confidence, ordering, and bounded metadata. Use borrowed data where practical; do not retain matched strings.

For arbitration, compare duplicate, contained, partial, adjacent, repeated, credential/PII, deterministic/NER, and equal-priority cases. Write a decision table with a total deterministic tie rule. Explain any change to core semantics; do not choose narrower or higher-confidence spans automatically as a security guarantee.

For plans, establish an immutable sorted non-overlapping representation and failure before partial transformation. Define capacity/overflow handling and deterministic replacement identity. Manifests may explain decisions using safe bounded metadata, never plaintext, mappings, sensitive source fragments, or vault authority state.

Document accepted decisions and alternatives in the appropriate existing architecture section or a substantive focused document. Identify unresolved upstream contracts and compatibility versions; do not freeze speculative public APIs. Return decision rationale and testable invariants, distinguishing proposals from implemented behavior.
