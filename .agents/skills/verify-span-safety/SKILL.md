---
name: verify-span-safety
description: "Design, add, or review anonymizer span and Unicode safety tests, property/fuzz coverage, and resource bounds; use for issue #8 or transformation safety changes."
---

# Verify span safety

Read `AGENTS.md`, `SECURITY.md`, `ARCHITECTURE.md`, `CONVENTIONS.md`, relevant contracts, and issue #8 plus epic #1 when issue-driven. Inspect existing test/fuzz tooling before introducing dependencies. Use synthetic inputs and sanitized failure diagnostics.

Cover reversed, out-of-range, zero-length, duplicate, contained, partial, and adjacent ranges; UTF-8 boundary violations; combining, bidi/control, and zero-width characters; placeholder-like literals; large inputs, finding counts, and replacement growth. Malformed UTF-8 belongs at byte/FFI adapters, not valid Rust `&str` inputs.

Verify independent observable invariants: no panic in the accepted public-input domain; documented rejection for invalid inputs; sorted non-overlapping final plans; deterministic output and decisions for identical configuration; exact preservation of unselected byte segments; bounded output and arithmetic; no plaintext in errors or manifest. Do not confuse source and output offsets when replacements change length.

Choose unit cases, generated properties, fuzz targets, and a small meaningful seed corpus based on the contract. Ensure the oracle does not duplicate the same arbitration algorithm. Test resource-limit boundaries without unbounded allocation or production data. Track capture abort/rollback conformance separately when reversible code is affected.

Run available relevant checks with recorded fuzz duration/configuration and seeds when applicable. Distinguish a passing bounded run from proof or complete coverage. If no Cargo/test harness exists, deliver a substantive test plan only when requested, and identify execution as not assessable.
