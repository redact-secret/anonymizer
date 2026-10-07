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

### Accepted whole-input policy: conservative union

`arbitrate_findings` normalizes once and performs one start-ordered sweep. Any
valid `Block` finding fails the whole operation, even when it does not overlap a
redaction. Span/limit validation takes precedence over policy failure. `Warn`
and `Allow` create no replacement and never suppress a `Redact`. Report-only
findings are not retained in accepted replacement metadata; callers already own
these findings and may report their safe metadata separately.

Each connected component of strictly overlapping `Redact` ranges becomes its
union. Adjacent ranges remain separate. This preserves all submitted redaction
coverage, including every byte of core credentials; a narrow high-confidence
NER span cannot expose the remainder. Connected interval unions contain no gap,
but mixed recognizers can extend replacement beyond core spans. No source wins
exclusive coverage. Dominant metadata chooses a label for the entire union,
not permission to drop other finding bytes. This can conservatively redact more
text and lose per-detector replacement granularity compared with standalone core.
It does not change core policy actions or claim exact core placeholder parity.

| Case | Accepted result | Safe reasoning |
| --- | --- | --- |
| Exact full-metadata repetition | Deduplicate before sweep | One contributor |
| Same range, different source/kind/confidence | One range | `ExactAgreement`, dominant label, contributor count |
| Contained redaction | Union preserves wider range | `OverlapUnion` |
| Partial or transitive overlap | Entire connected union | `OverlapUnion` |
| Credential versus PII / core versus NER | Union all redaction coverage | Dominant label per total key below |
| Equal priority/source/confidence | Same union regardless of submission order | Canonical metadata tie |
| Adjacent redactions | Separate ordered ranges | Separate labels and numbering later |
| Warn/Allow overlaps Redact | Redact unchanged | Report-only actions do not arbitrate coverage |
| Block anywhere | Fixed `Blocked` error, no accepted result | No source text or partial output |

Label selection uses this ascending total key: source class (`Core`, `Caller`,
`Fastner`), credential-category preference within that class, caller registration
number, then the full normalized `SourceFinding` lexicographic order. Registration
IDs come from host configuration, never arrival order. Core labels take priority
for recognizable canonical semantics; explicit host recognizers precede
statistical labels. Credential preference labels an already-covered union; it
is not a detection-quality guarantee. Remaining kind and span ties follow their
bounded declared ordering. Confidence is only a final canonical metadata tie,
never a confidence threshold or cross-model evidence ranking. Different
confidence variants and model calibrations are not meaningfully interchangeable.

Specificity is absent from the submitted contract and cannot be guessed. Narrower
versus wider exclusive winners were rejected because they can expose portions of
another finding. Global confidence-based precedence was rejected because it would
invent cross-model comparability. Configurable security priority is deferred until
a concrete consumer contract exists; v0.1 uses this single explicit policy.

`AcceptedSpan` has private fields and read-only getters: union `span`, original
`dominant` finding, unique redaction `contributors` count, and `ArbitrationReason`.
It carries no plaintext or arbitrary labels. Together with caller-owned normalized
findings, the union bounds identify every contributing redaction by intersection;
individual suppressed-label rows need not be duplicated in hot-path storage.
Counts and metadata are permutation-independent. Public construction only returns
sorted, non-overlapping spans validated against input. Metadata is suitable for
later safe manifests, not an authorization record or secret-free output proof.

Cost: one O(n log n) normalization sort, one O(n) sweep, and O(n) metadata storage;
no rescans, pairwise overlap graph, repeated interval insertion, or substring
allocation. Limits bound submitted counts before sorting. Tests cover transitive
chains, Unicode boundaries, duplicates, exact agreement, containment, partial
intersection, adjacency, source/kind ties, report-only actions, and global block.

### Immutable planning and safe manifest

`plan_irreversible(input, findings, PlanLimits)` invokes the single normalization /
arbitration pass and returns a `ReplacementPlan` borrowing the exact original
`&str`. Private fields prevent mutation or rebinding; output construction must use
this original input, never accept an unrelated same-length string. No original
substring is owned by a plan. Custom Debug hides borrowed input and prints only
safe manifest metadata. Holding a plan extends the lifetime of sensitive input;
hosts remain responsible for source storage and memory lifecycle.

`TransformationManifest` exposes only input/output byte counts and ordered
`PlannedReplacement` rows: accepted range, dominant bounded metadata, contributor
count/reason, mode, and a `ReplacementIdentity` of category plus ordinal. It owns
no plaintext, source fragments, raw reversible tokens, mappings, authority state,
or input fingerprint. Read-only accessors prevent mutation. Location/category/
length metadata can still reveal context and needs destination review; safe here
means structurally excludes values, not universally publishable.

Irreversible identities are global one-based source-order numbers, not per-value
identities: repeated values are numbered independently, and duplicate findings
are deduplicated before numbering. Fixed grammar is `<KIND_N>` with categories
`CREDENTIAL`, `PERSON`, `EMAIL`, `PHONE`, `NETWORK_ADDRESS`, `ADDRESS`, `OTHER`.
The planner computes exact decimal digit widths without building strings.
When a plan contains replacements, one bounded linear input scan rejects any
reserved `<KIND_` prefix anywhere, even malformed placeholder-like literals or
inside accepted spans. This conservative policy prevents generated-label
ambiguity and replacing matched plaintext with an identical placeholder. Empty
plans preserve placeholder-like source text because they generate no labels.
Escaping and per-input namespace searching were rejected to preserve untouched
bytes and avoid repeated scans. Reversible token collisions require their own
vault-facing policy; this contract does not invent tokens.

`PlanLimits` includes input/finding bounds plus defaults of 100,000 replacements,
64 bytes per placeholder, 32 MiB absolute output, and 16 MiB positive growth.
Hosts may change these explicitly. Capture count will be bounded with reversible
support. Allocation is fallible; checked arithmetic sums removed and added bytes
then calculates final capacity, rejecting overflow, output limits, or collisions
before any final output exists. Shrinkage does not count as positive growth.
All failure returns are fixed source-free errors; no partial plan/output escapes.
Plan and manifest share one row vector rather than duplicating metadata. No final
write or vault capture is performed by planning.

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

### Implemented constructor

`construct(ReplacementPlan)` consumes the immutable bound plan; there is no
separate caller-supplied input that could accidentally substitute a different
same-length document. It fallibly reserves the exact planned output length once
and appends each untouched slice, fixed placeholder, and final suffix in order.
Numbers are formatted using a bounded stack digit array, without a per-finding
heap string, regex, recognizer rescan, or formatting callback. Construction is
O(input bytes + output bytes + replacement count) after planning. Empty output
needs no allocation. The standard allocator may round requested capacity up;
this contract means one reserve and no growth, not exact allocator size.

`AnonymizedOutput` privately owns output plus the moved manifest. Text access is
explicit via `text()`/`into_parts()`; custom Debug only exposes manifest metadata,
including when output retains undetected sensitive values. This is not memory
zeroization or a guarantee about caller logging. Planning/allocation errors return
no transformed partial output. Generated synthetic corpus tests compare every
untouched byte slice and planned capacity across 128 deterministic Unicode cases;
stronger fuzz/property qualification and performance evidence follow separately.

## 8. Reversible mode

The optional dependency-free `reversible` feature exposes statically dispatched
`TokenSink` and `anonymize_reversible`. This is a local candidate transaction
contract qualified by dummy-sink conformance tests, not an implemented production
vault adapter. The trusted host supplies capture authority context to its sink;
source IDs and model-provided claims never establish authority.

Preflight normalizes/arbitrates once, rejects Block/invalid spans, checks capture
and replacement counts, calculates exact output length for the selected token
profile, and reserves source-slice metadata, manifest rows, and final output
before side effects. Zero captures return unchanged text without touching the
sink. Defaults: 100,000 captures, 32-byte maximum token, shared 32 MiB output and
16 MiB positive growth. Hosts explicitly accept exposure when raising limits.

One transaction follows `begin → stage(all borrowed accepted slices) → validate
all returned tokens → construct private output → commit → return`. Failed begin,
stage, validation, scratch allocation, or commit calls abort. No partial output
is returned. Sink errors are never formatted; only fixed `CaptureFailed`,
`InvalidToken`, or `CleanupFailed` errors cross the boundary. Abort must work after
a failed begin and failed commit; successful abort means all attempted mappings
are removed, including compensating cleanup of external partial writes. Failed
abort reports residual-state risk, and the host must reconcile retained mappings.
Commit must publish all mappings atomically or keep them abortable. No local code
can prove a remote sink honors these rules; persistent authority qualification
requires its own fault tests. Panic, process loss, cancellation, zeroization, and
distributed recovery remain host responsibilities. Sink methods must not panic.

The selected profile matches observed issued vault tokens: `<rsv_` plus 26
lowercase RFC4648 base32 characters `[a-z2-7]` plus `>`, exactly 32 bytes. Anonymizer
validates count/order correspondence, grammar, length, and uniqueness using a
fallibly allocated sorted borrowed-reference vector. It also rejects tokens equal
to any entire accepted original value without copying those values or retaining
a mapping store. The sink must honor bounds before allocating its returned vector;
post-return validation cannot undo excessive trusted-sink allocation. Unpredictable
identity generation and freedom from arbitrary substring/semantic plaintext
encoding are trusted sink obligations, not locally provable properties.

Before any capture, input containing contiguous case-insensitive `rsv_` is
rejected when replacements exist, including source-equal tokens and malformed
markers. This deliberately conservative profile prevents ordinary exact-token
literal collisions. It does **not** implement vault's Unicode-Cf-separated spoof
marker detection or output-binding parity. Such source markers may remain in
untouched bytes; a production adapter must apply the vault's stronger marker
rules or reject unsupported inputs. Empty capture calls generate no tokens and
preserve source literals. Generated tokens are placed in one ordered pass and
only exist transiently for construction; manifests contain mode `Reversible`
and safe category/ordinal references, never raw tokens, mappings, or authority.
Output Debug hides text and tokens. Tokens themselves remain potentially sensitive.

### Pinned vault evidence and upstream gap

Observed 2026-10-07, high confidence in source inspection:
[`@redact-secret/vault` 0.1.0-beta.5 types](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/packages/vault/src/types.ts)
expose TypeScript `capture(input, CaptureOptions)` with release grants and policy;
[`vault.ts`](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/packages/vault/src/vault.ts)
uses a core-backed whole-input capture plan, rather than a generic public native
Rust bulk-slice transaction. The issued grammar and stronger marker detection are
observed in [`token.ts`](https://github.com/redact-secret/redact-secret-vault/blob/022972391314640e719233b3d9d1f3ad0acb802d/packages/vault/src/token.ts).

An upstream native public contract is needed for begin/bulk staging/atomic
commit/idempotent abort (including partial external commit cleanup), with trusted
host authority binding, bounds, token profile, and fault conformance. No upstream
issue is posted and no private/internal capture-plan import, detector rescan,
TS subprocess, network stack, mapping store, or token generator is introduced.
Dummy tests qualify this local contract's success, phase failures, compensating
abort, cleanup failure, invalid/duplicate/count-mismatched tokens, collision
preflight, limits, global Block, and zero-capture behavior. Real vault interoperability,
persistent failure recovery, and full marker parity remain unqualified.

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
