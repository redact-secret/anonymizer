# Conventions

## 1. General principles

This repository favors:

- explicit contracts,
- small public APIs,
- deterministic behavior,
- benchmark-backed optimization,
- safe Rust,
- low allocation,
- dependency restraint.

Architecture should optimize both maintainability and hot-path performance.

## 2. Language

Primary implementation language: Rust.

Documentation, issue titles, commit messages, and public API comments should be in English.

## 3. Formatting and linting

Use standard Rust tooling:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Exact CI commands may evolve as the workspace is scaffolded.

## 4. Public API discipline

Do not expose a public abstraction only because it is convenient internally.

Public additions should answer:

- Is this required by an external consumer?
- Can it be kept private until the behavior is proven?
- Does it force allocation or dynamic dispatch on hot paths?
- Does it couple this repository to an implementation detail in another repository?

Prefer private implementation first, public contract later.

## 5. Performance-sensitive code

Hot-path code should prefer:

- borrowed `&str`,
- slices,
- compact enums,
- integer/range metadata,
- pre-allocation,
- single-pass output construction,
- static dispatch where practical.

Avoid by default:

- cloned matched substrings,
- `serde_json::Value`,
- per-finding heap maps,
- repeated regex compilation,
- repeated normalization,
- repeated scanning,
- mandatory trait-object dispatch where generic/static dispatch is viable.

## 6. Cross-repository contracts

Depend only on documented public APIs.

Never import:

- another repository's private module,
- generated build artifacts that are not a supported contract,
- test-only internals,
- unpublished package paths.

If a required contract is missing, create an explicit upstream design issue instead of reaching into internals.

## 7. Commit messages

Prefer concise imperative messages, for example:

```text
Add deterministic overlap planner
Avoid substring allocation in replacement pass
Define reversible capture boundary
```

A commit should represent one coherent change.

## 8. Tests

Every behavior change should include tests.

Important categories:

- unit tests,
- property tests,
- fuzz targets,
- integration tests,
- conformance tests,
- benchmark regression cases.

Performance-sensitive changes should include before/after measurements when practical.

## 9. Benchmarks

Benchmarks should record:

- input size,
- finding count,
- runtime,
- allocation behavior where measurable,
- binary-size impact for feature changes.

Do not optimize solely from microbenchmarks if the change makes end-to-end composition slower.

## 10. Error messages

Errors must not include matched values.

Use fixed/bounded error vocabulary suitable for logs without exposing source text.

## 11. Feature flags

Features must represent optional capabilities, not arbitrary build modes.

Optional integrations should not inflate the default build.

Feature combinations must be tested.

## 12. Documentation

Every architectural decision that affects:

- repository boundaries,
- security ownership,
- binary size,
- runtime dependencies,
- public contracts,

should be documented before implementation or in the same change.

## 13. Public-readiness

Before making the repository public:

- remove internal-only references that cannot be understood externally,
- add license information,
- add contribution instructions,
- enable vulnerability reporting,
- verify examples use synthetic data only,
- ensure issue templates do not invite submission of live secrets.
