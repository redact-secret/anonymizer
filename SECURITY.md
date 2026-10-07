# Security Policy

## Status

This repository is currently private and under active development.

Security guarantees described here are design goals unless a release explicitly states that they are implemented and qualified.

## Reporting a vulnerability

Do not open a public issue containing:

- live credentials,
- private personal data,
- reversible mappings,
- vault tokens,
- encryption material,
- exploit payloads containing sensitive customer data.

Before this repository becomes public, the maintainers should enable GitHub private vulnerability reporting / Security Advisories and publish the preferred contact path.

## Security boundary

`anonymizer` transforms findings into anonymized output.

It does **not** guarantee that:

- every secret or PII value was detected,
- anonymized output is universally safe,
- a statistical entity prediction is correct,
- a reversible token is non-sensitive,
- downstream applications will handle output safely.

Detection quality remains the responsibility of the configured recognizers.

## Sensitive-data handling

The implementation must avoid copying matched values unless necessary.

Preferred hot-path representation:

```text
borrowed input + byte ranges
```

instead of owned matched strings.

No raw matched value may be included in:

- errors,
- debug output,
- logs,
- metrics,
- traces,
- panic messages,
- snapshot fixtures,
- benchmark reports.

## Reversible mode

When reversible mode is enabled:

- mapping ownership belongs to `redact-secret-vault`,
- token issuance must use the vault contract,
- authorization belongs to the vault/server authority layer,
- `anonymizer` must not invent a second mapping store,
- `anonymizer` must not treat possession of a token as authorization,
- failures must not leave partially committed mappings without a defined rollback/abort contract.

## Untrusted input

All source text must be treated as untrusted.

The implementation must safely handle:

- malformed UTF-8 at FFI boundaries,
- invisible Unicode characters,
- bidi/control characters,
- overlapping spans,
- duplicate spans,
- out-of-range spans,
- zero-length spans,
- very large inputs,
- adversarial finding counts,
- placeholder-like literals already present in source text.

## Denial-of-service controls

Public APIs should support bounded operation.

At minimum, limits should exist or be inherited for:

- input size,
- number of findings,
- number of replacement spans,
- placeholder length,
- reversible captures,
- output growth.

Avoid algorithms with accidental quadratic behavior on attacker-controlled input.

## Unsafe Rust

Prefer safe Rust.

Any future use of `unsafe` requires:

- an explicit architectural justification,
- a narrowly scoped module,
- safety invariants in code comments,
- dedicated tests,
- benchmark evidence that the unsafe path is materially justified.

## Dependency policy

Prefer a small dependency surface.

New runtime dependencies should be reviewed for:

- transitive size,
- unsafe code,
- build scripts,
- network behavior,
- filesystem behavior,
- cryptographic implications,
- MSRV impact,
- WASM compatibility where relevant.

## Security testing

Before public release, include:

- fuzz tests for span merging and output construction,
- property tests for non-overlapping output plans,
- tests proving no source bytes outside selected spans are altered,
- Unicode adversarial corpus,
- token-literal collision tests,
- reversible abort/rollback tests,
- large-input resource-bound tests,
- no-secret-in-error tests.

## Disclosure discipline

Security documentation must distinguish:

- implemented behavior,
- tested behavior,
- qualified behavior,
- planned behavior.

Do not describe a planned control as a current guarantee.

## Reversible qualification limits

The optional local TokenSink contract requires trusted host authority wiring and
atomic commit/compensating abort. Dummy conformance does not qualify any production
vault. Cleanup failure may leave externally retained mappings; hosts must reconcile
without releasing partial output. Panics, cancellation, process loss, persistence,
zeroization, and authority remain outside this engine. The minimal contiguous token
marker preflight does not match vault's Unicode-Cf spoof detection; production
adapters must strengthen this before claiming vault output-binding parity.
