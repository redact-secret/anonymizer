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

The internal finding representation should remain compact.

Preferred characteristics:

- UTF-8 byte offsets for Rust-native paths.
- No matched substring ownership.
- Stable source identity.
- Bounded type identifiers.
- Confidence represented compactly.
- Optional metadata only when it has a demonstrated runtime use.

Avoid open-ended hot-path structures such as:

```rust
HashMap<String, serde_json::Value>
```

unless isolated from the core path.

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
