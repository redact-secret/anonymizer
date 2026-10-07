---
name: implement-anonymizer-pipeline
description: Implement a scoped Rust scaffold or normalization, arbitration, planning, and irreversible construction stage in anonymizer; use for issues #2–#6.
---

# Implement anonymizer pipeline

Read `AGENTS.md`, authoritative documents, the assigned issue, and epic #1. Inspect Cargo/CI and existing modules first. If scaffolding is explicitly in scope, establish only the minimal Rust workspace, feature structure, supported-toolchain policy, and actual checks required by #2; do not invent an MSRV or expose convenience abstractions as stable APIs.

Implement the assigned stage against accepted contracts. Keep normalization, arbitration, planning, and construction separable. Validate ranges before slicing, borrow source text, avoid owned matched substrings, and never rescan recognizers' findings. Missing precedence, collision, or span policy needs an explicit documented decision.

Construct output only from a validated sorted non-overlapping plan plus borrowed input. Calculate bounded capacity with checked arithmetic, append untouched slices and replacements in order, and preserve unselected bytes exactly. Define numbering and literal-placeholder collisions without exposing source values.

Keep default dependencies lightweight and integrations optional. Use documented sibling public surfaces; do not implement detectors, mappings, authorization, crypto, or reconstruction. Safe diagnostics must not format source values even on failure paths.

Add behavior tests appropriate to the stage, including empty, adjacent, multibyte, expansion/shrinkage, invalid-input, tie, and unchanged-byte cases. Run actual Cargo/CI checks when available plus default and affected feature combinations. Report unavailable checks and API/security/performance gaps rather than claiming qualification.
