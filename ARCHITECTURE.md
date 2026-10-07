# Architecture

## 1. Scope

`anonymizer` is the composition layer that transforms a source document using findings produced by external recognizers.

It owns:

- finding normalization,
- overlap arbitration across sources,
- replacement planning,
- irreversible placeholder generation,
- optional reversible-token orchestration,
- final output construction.

It does not own:

- canonical credential detection,
- statistical NER,
- original-value storage,
- authorization,
- persistence,
- cryptography,
- restoration.

## 2. Ecosystem boundary

```text
                       +-------------------+
                       |      fastner      |
                       | independent NER   |
                       +---------+---------+
                                 |
                                 | Entity spans
                                 v
+-------------------+     +------+-------+
|   redact-secret   |---->|  anonymizer  |
| canonical core    |     | orchestration|
+-------------------+     +------+-------+
                                 |
                                 | optional reversible capture
                                 v
                       +---------+----------+
                       | redact-secret-vault|
                       | mapping + authority|
                       +--------------------+
```

Dependency direction must remain one-way.

`redact-secret` and `fastner` must remain usable without this repository.

## 3. Runtime boundary is not repository boundary

The project must support a zero-serialization native Rust path.

Repository separation exists for:

- ownership,
- release cadence,
- independent benchmarking,
- independent public API evolution,
- clearer security responsibility.

It must not imply mandatory:

- subprocess execution,
- HTTP,
- JSON,
- Serde,
- IPC,
- database access.

## 4. Internal pipeline

Preferred whole-input flow:

```text
borrowed input
    |
    +--> deterministic findings
    |
    +--> NER findings
    |
    +--> caller findings
            |
            v
     normalize spans
            |
            v
     classify source/type
            |
            v
      overlap arbitration
            |
            v
      replacement plan
            |
            v
     capacity calculation
            |
            v
      allocate output once
            |
            v
        ordered write
```

The anonymizer should not rescan text to rediscover findings already supplied by recognizers.

## 5. Finding model

The implemented public `SourceFinding` contains only `Span`, `FindingKind`,
`FindingSource`, `Confidence`, and `FindingAction`. No matched substring or
open-ended metadata is retained. `Span` is `[start, end)` in UTF-8 bytes against
the exact original input, with no Unicode normalization or offset translation.
`normalize_findings` rejects zero-length, reversed, out-of-range, and non-character
boundary spans before allocation, sorting, or slicing. Any invalid submission
fails the whole call with a fixed source-free error.

Sources are `Core`, `Fastner`, or host-registered `Caller(u16)`. IDs are stable
configuration ordinals, never account identifiers or value-derived labels. They
are metadata, not authorization or proof of recognizer identity; the trusted
host must assign sources. Kinds are a closed category enum; no arbitrary type
name can enter diagnostics. `Other` is only for explicitly accepted unmapped
sensitive categories. Detector-specific placeholder fidelity remains a future
bounded-kind extension, not an arbitrary-string escape hatch.

Confidence preserves interpretation: `Unknown`, `Ordinal(Low|Medium|High)`, or
`Calibrated(u16)` mapping `0..=65535` to `0..=1`. Ordinal evidence is not converted
into invented probabilities. Values across confidence variants or model versions
are not inherently comparable. `FindingAction` preserves `Redact`, `Block`,
`Warn`, and `Allow`; adapters must never reinterpret block as successful output
or report-only actions as mandatory replacement.

Normalization sorts lexicographically by declaration order: span start/end,
kind, source, confidence, action. It removes only exact full-metadata duplicates;
same-span findings with different source, confidence, action, or kind remain.
Overlapping and adjacent findings remain for arbitration. Sorting order is a
canonical representation, not an accepted overlap precedence. Complexity is
O(n log n) time and O(n) metadata space, without substring allocations.
`Limits` defaults to 16 MiB input and 100,000 supplied findings, checked before
allocation and deduplication. Hosts can lower or explicitly raise these bounds;
raising them accepts greater resource exposure. Replacement/capture/output
bounds belong to later stages.

### Public sibling mapping evidence

Observed 2026-10-07 by source inspection; no runtime adapter or version support
qualification is claimed. No sibling dependency is added.

- [`redact-secret` types at efe7149](https://github.com/redact-secret/redact-secret/blob/efe714968c0e8865df936de9ffa3e095aa771b5b/crates/secret-scan-core/src/types.rs),
  package `redact-secret` version `0.1.0-beta.14`: map public `Finding::range()`
  getters `start()`/`end()` to `Span`, `confidence()` variants to ordinal evidence,
  and `action()` exactly to `FindingAction`, with source `Core`. Hosts map
  `type_name()` through an explicit reviewed category table; do not assume every
  core finding is a credential. Unknown names require an explicit `Other` mapping
  or a safe adapter error, never dropping a finding silently. Core scanning has
  already resolved its own detector competition; consume its public policy-applied
  results without importing candidates or private arbitration. Bounded category
  mapping loses detector-specific names and obfuscation metadata; it does not
  claim feature parity with standalone core redaction.
- [`fastner` types at f4a6a56](https://github.com/redact-secret/fastner/blob/f4a6a567e6255490ee35229f8fed95e09e644c94/crates/fastner/src/types.rs),
  workspace version `0.0.0`, unpublished: map `Entity::range().start()/end()`,
  `Entity::kind()` (`Person` at this revision), and `confidence().raw()` directly,
  with source `Fastner`, kind `Person`, and explicit host-selected `Redact` intent.
  `EntityKind` is non-exhaustive: future variants require reviewed mapping or safe
  failure. Do not call `ByteRange::slice` on unvalidated caller input. A published
  runtime contract and integration tests are required before production support.

Confidence: high for these pinned public getters; package publication and future
API compatibility remain unqualified. Neither source requires recognizers to
reference anonymizer or exposes mapping/restore authority.

## 6. Overlap arbitration

Overlap handling is a product contract and must be explicit.

Candidate policy axes may include:

1. explicit security priority,
2. source precedence,
3. specificity,
4. confidence,
5. narrower span,
6. deterministic registration order.

The final rule must be deterministic and testable.

The project must not silently alter the meaning of a `redact-secret` finding without an explicit policy decision.

## 7. Irreversible mode

Irreversible mode replaces accepted spans with placeholders.

Examples:

```text
Sarah Kim -> <PERSON_1>
ghp_...   -> <GITHUB_TOKEN_1>
```

Requirements:

- deterministic numbering for deterministic inputs,
- no matched value in errors,
- placeholder validation,
- no overlapping writes,
- one ordered output pass.

## 8. Reversible mode

Reversible mode delegates token issuance and retention to a vault-facing capability.

Conceptually:

```text
finding span
   |
   v
capture eligible original value
   |
   v
vault issues unpredictable token
   |
   v
anonymizer places token into output
```

The anonymizer must not retain an independent copy of the vault mapping after the operation.

A reversible token must not be treated as a plaintext identifier or as authorization.

## 9. Performance rules

The following are architectural requirements:

1. Repository boundaries must not require process boundaries.
2. Native Rust composition must not require serialization.
3. Borrow source text whenever possible.
4. Represent matched locations as ranges.
5. Avoid per-finding substring allocation.
6. Do not repeat Unicode normalization if upstream results already guarantee the needed invariant.
7. Prefer bulk APIs.
8. Sort/merge findings once.
9. Construct final output in one ordered pass.
10. Optional integrations must be feature-gated or otherwise removable from small builds.
11. Preserve static-linking and LTO opportunities.
12. Benchmark architecture changes before accepting abstraction overhead.

## 10. Binary-size strategy

The default crate should not automatically include:

- `fastner` model data,
- vault persistence backends,
- database drivers,
- crypto SDKs,
- network stacks.

Possible feature shape:

```toml
[features]
default = []
core = ["dep:redact-secret"]
fastner = ["dep:fastner"]
reversible = ["dep:redact-secret-vault-contract"]
full = ["core", "fastner", "reversible"]
```

Exact feature names are not yet fixed.

## 11. Error boundary

Errors must be safe by construction.

Errors may contain:

- fixed error codes,
- bounded counts,
- safe configuration identifiers.

Errors must not contain:

- matched values,
- source fragments,
- original reversible values,
- vault secrets,
- authorization details that should not leave the trusted boundary.

## 12. Streaming

Streaming is not implied by whole-input support.

Streaming introduces:

- incomplete entities,
- unresolved overlap windows,
- delayed NER decisions,
- token issuance before finality,
- cancellation semantics,
- rollback concerns.

Streaming should be designed separately and only after whole-input behavior is stable and benchmarked.

## 13. Public-release gate

Before public release:

- API stability policy must exist.
- Supported dependency versions must be documented.
- Performance baselines must be reproducible.
- Binary-size baselines must be recorded.
- Adversarial Unicode and overlap suites must pass.
- Reversible mode must pass vault conformance tests.
- No private/internal dependency path may be required from sibling repositories.
