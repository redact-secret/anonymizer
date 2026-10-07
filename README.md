# anonymizer

High-performance anonymization orchestration for the Redact Secret ecosystem.

> **Status:** Private development repository. This project is not yet considered production-ready or publicly supported. The repository is expected to become public after its architecture, conformance, security, and performance contracts are sufficiently validated.

## Purpose

`anonymizer` turns findings from one or more detection engines into a single anonymized output.

It is designed to compose with:

- [`redact-secret/redact-secret`](https://github.com/redact-secret/redact-secret) for canonical deterministic credential and PII detection.
- [`redact-secret/fastner`](https://github.com/redact-secret/fastner) for optional statistical/contextual named-entity recognition.
- [`redact-secret/redact-secret-vault`](https://github.com/redact-secret/redact-secret-vault) for optional reversible tokenization and mapping lifecycle.
- [`redact-secret/restore`](https://github.com/redact-secret/restore) for controlled reconstruction of reversible output.

`anonymizer` is **not** a detector implementation and is **not** a vault.

Its primary responsibility is orchestration:

```text
input
 ├─> redact-secret findings
 ├─> fastner entities
 └─> optional caller-provided findings
          |
          v
   normalize / arbitrate
          |
          v
   replacement planning
          |
          v
   one-pass output construction
          |
          +--> irreversible anonymized text
          |
          └--> reversible tokenized text
                  |
                  v
           redact-secret-vault
```

## Design goals

- Fast native Rust composition.
- No mandatory serialization between ecosystem components.
- Borrowed input and byte-range based processing on hot paths.
- One merge phase and one output-construction pass where possible.
- No detector reimplementation.
- No ownership of authorization, persistence, cryptography, or restoration policy.
- Optional integrations must not inflate the default binary.
- Deterministic output for deterministic inputs and configuration.
- Clear separation between irreversible anonymization and reversible tokenization.

## Non-goals

`anonymizer` does **not**:

- Reimplement credential or PII detectors from `redact-secret`.
- Reimplement NER models from `fastner`.
- Store original values.
- Decide whether a restore request is authorized.
- Own persistent storage, encryption keys, or tenant policy.
- Require a network service boundary.
- Require JSON, Serde, or IPC in native Rust usage.

## Implemented whole-input API

The dependency-free Rust crate validates caller findings, unions overlapping
redactions, builds an immutable input-bound plan, and constructs irreversible
output with deterministic placeholders. It performs no detection itself.

```rust
use anonymizer::{anonymize, Confidence, FindingAction, FindingKind,
    FindingSource, SourceFinding, Span};

let finding = SourceFinding {
    span: Span { start: 0, end: 14 },
    kind: FindingKind::Person,
    source: FindingSource::Caller(1),
    confidence: Confidence::Unknown,
    action: FindingAction::Redact,
};
let output = anonymize("synthetic-name!", &[finding]).unwrap();
assert_eq!(output.text(), "<PERSON_1>!");
```

`anonymize(input, findings)` uses default limits. For custom bounds or planning
inspection, call `plan_irreversible(input, findings, limits)` then `construct(plan)`.
The constructor takes its original input only from the privately bound plan,
reserves final capacity once, and writes untouched slices and fixed labels in
order. `normalize_findings` and `arbitrate_findings` are also available for callers
that need validated metadata separately; do not call them redundantly before the
planner. `output.into_parts()` moves text and safe manifest without cloning.
Debug on output and plan excludes text. Undetected bytes remain unchanged;
placeholder labels neither prove complete detection nor confer authority.

Contracts, decision tables, limits, and reserved `<KIND_` collision rejection are
in ARCHITECTURE.md. Optional `reversible` enables `anonymize_reversible` with a trusted generic
transaction `TokenSink` and dummy conformance coverage. No production vault or
recognizer adapter is implemented; see the documented upstream contract gap. Public APIs remain developmental until release policy
and integration qualification are established.

The default dependency graph contains only this crate and Rust's standard
library. There are no runtime/development dependencies, sibling imports, build
scripts, serialization, or services. `default = []`; optional `reversible` adds local Rust transaction orchestration
without runtime dependencies. Future integration flags require
implemented capabilities, not empty promises.

CI is configured for stable Linux/macOS/Windows, one build job and one matrix
entry at a time. Local checks passed on macOS with Rust 1.99.0. Linux/Windows
remain unqualified: GitHub Actions execution is currently blocked by account
billing/spending limits. MSRV and WASM are pending qualification; no older
toolchain support is claimed. Cargo.lock is committed for repository checks;
the crate is unpublished. Release builds enable LTO and one codegen unit.

Run the checks in CONVENTIONS.md plus `cargo test --locked --no-default-features`
and `cargo build --locked --release --no-default-features`. Small devices should
set `CARGO_BUILD_JOBS=1`. Performance and sibling integration qualification remain
pending; implemented irreversible behavior does not imply release readiness.

## Performance contract

Repository boundaries are not runtime boundaries.

Native Rust integrations should support:

- direct in-process calls,
- static linking,
- cross-crate optimization,
- LTO-friendly builds,
- borrowed text,
- byte ranges instead of copied matched strings,
- bulk submission of findings,
- pre-sized output buffers,
- no repeated tokenization or normalization unless explicitly required.

The project should avoid architectural choices that force:

```text
crate -> JSON -> process -> JSON -> crate
```

for local Rust use.

Service or IPC integrations may be added separately, but they must remain optional.

## Relationship to `redact-secret`

`redact-secret` remains a complete standalone library and keeps its own basic redaction APIs.

`anonymizer` exists for higher-order workflows:

- merge findings from multiple engines,
- resolve cross-source overlaps,
- apply richer replacement strategies,
- coordinate deterministic and statistical findings,
- optionally issue reversible tokens.

This project must not turn `redact-secret` into a scan-only dependency.

## Relationship to `fastner`

`fastner` is an independent NER engine.

`anonymizer` may consume its entity spans, but:

- `fastner` must not depend on `anonymizer`,
- `fastner` must not depend on `redact-secret`,
- `anonymizer` must not duplicate NER logic.

## Relationship to `redact-secret-vault`

In reversible mode, `anonymizer` may request token issuance and capture through a small vault-facing contract.

The vault owns:

- token identity,
- original-value retention,
- TTL and revocation,
- usage budgets,
- capture lifecycle,
- tenant/principal/source/sink/purpose policy,
- persistence,
- cryptography,
- key providers.

`anonymizer` owns none of those concerns.

## Security model

Anonymization reduces exposure of known findings. It does not prove that output is safe or that every sensitive value was detected.

See [SECURITY.md](SECURITY.md) and [ARCHITECTURE.md](ARCHITECTURE.md).

## Repository status

Before public release, this repository should have at minimum:

- architecture decisions for overlap arbitration,
- compatibility contracts with supported `redact-secret` and `fastner` versions,
- adversarial Unicode tests,
- deterministic-output tests,
- large-input and high-finding-count benchmarks,
- fuzz/property tests for span safety,
- reversible-mode conformance against the vault contract,
- documentation of supported runtimes and binary-size impact.

## License

A license should be added before the repository becomes public.
