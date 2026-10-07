# Agent instructions

## Purpose and authority

Read `README.md`, `ARCHITECTURE.md`, `CONVENTIONS.md`, and `SECURITY.md`
before changing architecture, workflows, or implementation. These documents
remain authoritative for scope, ownership, engineering conventions, and security.

`anonymizer` is the Redact Secret ecosystem's forward transformation engine.
It composes findings from `redact-secret`, `fastner`, and caller recognizers into
one deterministic replacement plan and irreversible or vault-backed reversible
output. Primary implementation language is Rust. Write repository documentation,
issue titles, commit messages, and public API comments in English.

This is currently a documentation-only foundation: no Cargo workspace, runtime
implementation, CI, benchmark harness, or test suite exists. The README API and
architecture feature names are proposals, not frozen or implemented contracts.
Inspect the current tree before choosing commands or claiming support.

## Ownership boundaries

- Anonymizer owns finding normalization, source identity, overlap arbitration,
  replacement planning, safe transformation metadata, irreversible placeholders,
  reversible token placement, and final output construction.
- `redact-secret` owns canonical credential/PII detection and retains standalone
  redaction APIs. Do not turn it into a scan-only dependency.
- `fastner` owns independent statistical NER. Neither recognizer should depend
  on anonymizer; do not duplicate detection or NER logic here.
- `redact-secret-vault` owns token identity, original-value mappings, capture
  lifecycle, TTL, revocation, budgets, tenant/principal/source/sink/purpose policy,
  persistence, cryptography, and keys. Do not create a second mapping store.
- `restore` owns controlled reconstruction. Anonymizer neither restores values
  nor decides restore authority. Token possession never establishes authorization.

Use documented sibling public APIs only. If a needed contract is missing,
record the upstream requirement rather than importing private modules, test-only
internals, generated artifacts, or unpublished package paths. Creating or posting
an upstream issue requires authorization; a local design note can record the gap.

## Pipeline contracts

Preserve the whole-input pipeline: borrowed input and supplied findings → span
normalization → deterministic arbitration → immutable ordered replacement plan
→ capacity calculation → one ordered output construction pass.

- Validate ranges and UTF-8 byte boundaries before slicing or transformation.
  Specify zero-length, reversed, out-of-range, duplicate, and overlap behavior.
- Keep findings compact: ranges, bounded kinds, stable sources, compact confidence;
  avoid owned matched substrings and open-ended per-finding metadata maps.
- Arbitration is an explicit product contract. Document precedence and ties with
  decision tables; candidate policy axes are not an accepted precedence order.
  Preserve canonical core semantics unless an explicit policy change is accepted.
- Plans must be sorted and non-overlapping. Preserve bytes outside accepted spans
  exactly. Define deterministic numbering and placeholder-literal collisions.
- TransformationManifest metadata must exclude plaintext, source fragments,
  token-to-value mappings, and vault authorization state. Do not assume opaque
  reversible tokens are safe diagnostic labels.
- Reversible capture needs explicit abort/rollback semantics. A failed capture
  or planning stage must return no partially transformed user-visible result;
  externally committed mappings also need a defined cleanup contract.
- Whole-input support does not imply streaming. Design streaming separately
  after whole-input behavior is stable and benchmarked.

## Performance and dependencies

Repository boundaries must not force process boundaries. Preserve direct native
Rust composition, static linking, LTO, and a zero-serialization hot path.
Do not require JSON, Serde, HTTP, IPC, or subprocesses for local Rust use.

Borrow text, use byte ranges, prefer bulk calls, normalize/sort/merge once, avoid
rescanning or redundant Unicode normalization, pre-size output, and construct it
in one ordered pass. Prefer static dispatch where practical and private APIs until
an external contract is demonstrated.

Optional integrations must be removable from the default build. Do not pull NER
models, vault persistence, database drivers, crypto SDKs, or network stacks into
it by default. Review new dependencies under `SECURITY.md`, including transitive
size, unsafe code, build scripts, side effects, MSRV, and relevant WASM support.
Benchmark architecture changes before accepting abstraction overhead.

## Security and qualification

Treat source text and external findings as untrusted. Use synthetic examples and
fixtures; never collect or publish live credentials, real personal data, private
captures, reversible mappings, or vault tokens.

Never put raw matched values in errors, debug output, logs, metrics, traces,
panics, snapshots, manifests, or benchmark reports. Report suspected sensitive
data by location and category without echoing it. Keep errors fixed and bounded.

Bound input size, finding/replacement counts, placeholder length, capture count,
and output growth. Check arithmetic overflow and avoid accidental quadratic
behavior on attacker-controlled input. Prefer safe Rust; any `unsafe` requires
architectural justification, local safety invariants, dedicated tests, and
benchmark evidence as specified in `SECURITY.md`.

Distinguish planned, implemented, tested, and qualified behavior. Anonymization
reduces exposure of detected findings; it does not establish complete detection
or universally safe output. Follow private vulnerability reporting discipline;
do not publish undisclosed details or contact others without authorization.

## Issues and Git workflow

Read the current issue and parent epic before issue-driven work. Issues track
work; accepted contracts belong in repository documentation. Refresh GitHub
before relying on issue status or scope.

Open-issue routing checked on 2026-10-07 in
https://github.com/redact-secret/anonymizer/issues:

- #1: v0.1 deterministic anonymization engine epic; parent of #2–#9.
- #2: Rust scaffold, CI, lightweight features, minimal public API.
- #3: normalized finding/span/source/confidence contract.
- #4: deterministic cross-source overlap arbitration and decision table.
- #5: immutable replacement plan and safe TransformationManifest.
- #6: one-pass irreversible constructor and placeholder behavior.
- #7: minimal vault-facing capture contract and reversible failure semantics.
- #8: adversarial spans/Unicode, property tests, fuzzing, resource bounds.
- #9: performance/allocation/binary-size baselines and reproducible release gates.

Issue descriptions are requirements, not evidence that features exist. This
routing does not authorize implementing all issues during an unrelated task.
Work on a feature branch; target `main` for authorized PRs. Preserve existing
staged, unstaged, and untracked user work. Do not commit, publish, close issues,
or merge merely because local work finished. Never merge your own PR without
explicit authorization for this repository and task.

## Local skills and context discovery

Canonical skills live in `.agents/skills/`. Expose each to Claude via
`.claude/skills/<name> -> ../../.agents/skills/<name>`; edit canonical files.
Preserve any separate tool-managed `graft` installation.

- `design-anonymizer-contract`: finding, arbitration, plan, and manifest decisions.
- `implement-anonymizer-pipeline`: scoped Rust scaffold or pipeline implementation.
- `verify-span-safety`: adversarial Unicode, span, resource, property/fuzz coverage.
- `integrate-vault-capture`: reversible capture boundary and failure conformance.
- `qualify-anonymizer-performance`: benchmarks, feature size, and release evidence.

Use RTK for shell commands when available. If `~/.codex/RTK.md` exists, read and
follow it; its absence must not block work or be treated as loaded instructions.
Start with `rtk proxy graft map` when available. Verify that the graph covers this
repository before using `graft ask --source`, `graft grep`, or `graft callers`.
A fallback graph for another repository is not evidence about this one. Read
canonical documents and use `rg` when coverage is absent. Open truncated spans
before editing. Do not install or upgrade tooling for documentation edits.

## Verification before finishing

Review tracked diffs and new/untracked files for ownership, explicit contracts,
security claims, synthetic examples, local references, and unrelated changes.
Check skill frontmatter and every Claude symlink's canonical destination.
Run `rtk git diff --check` and check new files for whitespace too.

Documentation-only work does not have runnable Rust checks in the current tree;
report them as not assessable rather than claiming success. Once Cargo/CI exists,
run the actual relevant checks, including the conventions' standard commands:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Also check the default/lightweight build and affected feature combinations.
Behavior changes require meaningful tests. Run applicable property/fuzz,
conformance, and performance checks for their changed contracts. Report measured
results, skipped/unavailable checks, remaining qualification gaps, and completed
edits concisely; do not claim release readiness without reproducible evidence.
