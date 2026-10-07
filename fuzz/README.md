# Bounded span-pipeline fuzz qualification

`targets/span_pipeline.rs` is a dependency-free byte-entry fuzz target shared by
integration properties and `examples/fuzz_replay.rs`. Its wire format is one text
length byte, up to 96 text bytes, then three-byte start/end/action records (at
most 64 findings). Offset 255 selects `usize::MAX`; other offsets include values
outside text. Fallible `str::from_utf8` rejects malformed external bytes before
calling the `&str` API. This is a simulated external adapter boundary, not a
shipped FFI or byte-input API.

The five inspectable hex seeds cover ASCII overlap/duplicate/adjacency, Unicode,
malformed UTF-8, extreme/reversed/empty spans, and reserved placeholder literals.
Every fixture is synthetic. Assertions use fixed diagnostics and do not print
input values. Generated valid-input properties add 512 deterministic cases with
Unicode, core/caller/NER metadata, duplicates, and mixed actions. An independent
byte bitmap computes all Redact coverage rather than reproducing the interval
sweep. Tests check accepted union coverage, ordering/non-overlap, permutation
stability of output/manifest, exact untouched bytes, and output capacity.

Reproduce the bounded mutation smoke with:

```sh
CARGO_BUILD_JOBS=1 cargo run --locked --release --example fuzz_replay -- 20000
```

The runner first replays five seeds, then performs 20,000 deterministic mutations
(1–5 overwritten bytes each) using xorshift state `0x5eedcafe12345678`. It caps
iterations at 1,000,000 and uses only a small cloned seed buffer per iteration.
On 2026-10-07, macOS x86_64 / Rust 1.99.0, this smoke passed in 12 ms excluding
compilation; elapsed time is informational, not a performance gate. CI runs the
same smoke with one build job and one test thread per sequential OS entry.
Execution on GitHub remains blocked by account billing/spending limits.

This runner is mutation/replay fuzzing, not coverage-guided libFuzzer, sanitizers,
a coverage measurement, or an exhaustive safety proof. `cargo-fuzz` is unavailable
on this host; no new tooling or heavy development dependencies were installed.
A future coverage-guided harness can call this same bounded target. Corpus
expansion, extended campaigns, WASM/FFI qualification, allocation-failure injection,
and production vault conformance remain release evidence gaps.

Resource-bound tests include the real default 100,000 submitted-finding boundary
and one excess finding, the 16 MiB input boundary and one excess byte, plus small
exact growth/placeholder/capture/replacement/output limits. Reversible conformance
covers failures separately and explicitly records the unchanged Unicode-Cf token
marker detection gap; these tests do not imply marker parity with the vault.
