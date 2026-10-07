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

## Implemented scaffold

The Rust crate currently exposes `anonymize(&str) -> Result<String, AnonymizeError>`.
It preserves input exactly and rejects input above 16 MiB. It performs no
detection or replacement and must not be treated as sanitized output. Private
normalization, arbitration, planning, and construction modules establish the
pipeline boundaries. `normalize_findings(input, findings, limits)` now validates
and canonicalizes compact caller-supplied metadata (see ARCHITECTURE.md); the
identity `anonymize` scaffold does not yet consume it or apply replacements.
`arbitrate_findings` returns immutable ordered overlap unions and safe dominant
metadata, retaining all redaction coverage and rejecting upstream Block actions.
Its explicit policy and decision table are documented in ARCHITECTURE.md.
`plan_irreversible` adds an immutable input-bound plan, safe manifest, fixed
placeholder identities, collision rejection, and checked output capacity; final
replacement construction follows separately.

The default dependency graph contains only this crate and the Rust standard
library. There are no runtime or development dependencies, sibling imports,
build scripts, serialization, network services, or optional integration features
yet. `default = []`; `--no-default-features` and `--all-features` are currently
equivalent. Integration flags will be added only with implemented capabilities.

CI tests the current stable toolchain on Linux, macOS, and Windows with one build
job and one matrix target at a time. Local scaffold checks ran on macOS with
Rust 1.99.0; Linux/Windows remain unqualified: GitHub Actions execution is currently blocked
by account billing/spending limits. CI configuration is not a passing result.
MSRV is pending qualification: no `rust-version` or older-toolchain support is
claimed. Before release, choose and test an MSRV against every enabled feature.
`Cargo.lock` is committed for reproducible repository checks; this is not a
published crate. Release builds enable LTO with one codegen unit.

Run the checks in CONVENTIONS.md plus `cargo test --locked --no-default-features`
and `cargo build --locked --release --no-default-features`. On small devices set
`CARGO_BUILD_JOBS=1`. Performance, WASM, and sibling-version qualification remain
pending; the scaffold does not imply v0.1 release readiness.

## Conceptual API

The exact public API is intentionally not frozen yet. The preferred shape is bulk-oriented and allocation-conscious.

```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub struct SourceFinding<'a> {
    pub span: Span,
    pub kind: &'a str,
    pub confidence: u16,
    pub source: FindingSource,
}

pub enum AnonymizationMode<'a> {
    Irreversible,
    Reversible(&'a dyn TokenSink),
}

pub fn anonymize<'a>(
    input: &'a str,
    findings: &[SourceFinding<'a>],
    mode: AnonymizationMode<'_>,
) -> Result<AnonymizedOutput, AnonymizeError>;
```

The concrete API may differ after benchmarking and integration work.

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
