# v0.1 foundation qualification

This is reproducible local foundation evidence, not production/public-release
approval. `baseline.csv` contains three raw synthetic workload repetitions;
`baseline.json` records host/toolchain/build, medians, RSS, comparable binary
sizes, source digest, and reviewable budgets. No text, tokens, mappings, private
paths, or account identifiers enter these reports.

## Reproduction

Use the installed current stable Rust with one job and one test thread:

```sh
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 cargo test --locked --all-features
CARGO_BUILD_JOBS=1 cargo bench --locked --bench qualification --no-default-features
CARGO_BUILD_JOBS=1 cargo bench --locked --bench qualification --features reversible
python3 scripts/qualify.py
python3 scripts/qualify.py --check-baseline
python3 scripts/release_gate.py
```

The stdlib-only Python runner builds the instrumented harness once, then runs it
three times and builds/runs identical-profile default/reversible `size_probe`
executables in separate target directories. With no flag, generated reports go
under ignored `target/qualification`. `--record` intentionally replaces reviewed
checked-in reports and cannot be combined with `--check-baseline`. Recording uses
actual UTC date. Baseline comparison requires the same OS release, architecture,
CPU/RAM information where available, and exact Rust/Cargo versions. Source digest
is provenance, not a condition preventing comparisons after code changes.
New hosts/toolchains need their own reviewed baselines; never compare macOS
absolute timing/size numbers directly with Linux/Windows.

## Workloads and instrumentation

The eight workload matrix includes empty input, repeated 256-byte small messages
with one and three sources, 64 one-byte expansion spans, 64 KiB shrinkage, 1 MiB
sparse findings, 100,000 adjacent findings, and 50,000 dense overlapping findings.
Each generated fixture uses only `x` input and bounded metadata. CSV records bytes,
submitted findings, source count, overlap profile, output bytes, iteration count,
mode, stage, allocation calls/traffic, and time. Detection/model costs are absent:
findings are precomputed. The reversible sink is a deterministic synthetic dummy,
with no cryptography, authority, persistence, mappings, or network. Those production
costs must be measured in a future qualified adapter.

Planner and constructor are timed separately; constructor planning is outside its
measured interval. End-to-end measurement remains the architectural decision
metric: separate medians must not be summed or used to hide end-to-end regressions.
Timing uses Instant plus black_box, repeated short messages and bounded heavy
iterations. Timed end-to-end includes result cleanup; constructor timing ends
before cleanup. This distinction and timer overhead dominate very small inputs.

`benches/support/counting_allocator.rs` is the only unsafe instrumentation surface.
It forwards untouched pointer/layout/size to System, performs no dereference or
ownership change, and records successful alloc/zeroed/realloc calls via atomics.
Deallocations forward unchanged and are not counted as allocation. Single-threaded
smoke checks verify alloc, zeroed alloc, realloc, and dealloc behavior before every
run. The runtime still forbids unsafe code. Requested bytes measure allocation
traffic, not retained bytes, allocator usable sizes, zeroization, or peak heap.
Fixture generation/reporting are outside counted operations. Timing uses the same
proxy allocator with tracking off, so its branch/atomic-load overhead is present;
these are comparable engine measurements, not an uninstrumented speed promise.
System maximum RSS is measured by `/usr/bin/time -l` for each already-built
benchmark subprocess on macOS. It includes fixtures, allocator, code, and runtime,
not just output; non-macOS RSS is unavailable in this runner and reported null.

## Baseline and regression gates

| Workload | Input bytes / findings | Irreversible median µs | Dummy reversible median µs | E2E allocations I/R | Irreversible MiB/s |
| --- | --- | ---: | ---: | ---: | ---: |
| empty | 0 / 0 | 0.049 | 0.051 | 0/0 | — |
| small-log-core | 256 / 4 | 1.565 | 5.600 | 4/11 | 156.0 |
| small-log | 256 / 4 | 1.982 | 6.132 | 4/11 | 123.2 |
| expansion | 256 / 64 | 10.016 | 99.712 | 4/71 | 24.4 |
| shrinkage | 65,536 / 16 | 75.058 | 199.830 | 4/23 | 832.7 |
| large-input | 1,048,576 / 128 | 1479.927 | 10750.154 | 4/135 | 675.7 |
| high-count | 100,000 / 100,000 | 76732.915 | 280579.247 | 4/100007 | 1.2 |
| dense-overlap | 65,536 / 50,000 | 2133.148 | 2047.117 | 4/8 | 29.3 |

Recorded three process RSS values: 45.07 MiB, 44.56 MiB, 45.36 MiB. The 100,000-finding irreversible operation requests 19,688,895 allocation-traffic bytes (four calls); the synthetic reversible operation requests 36,800,000 bytes (100,007 calls, including 100,000 dummy token strings). Retained heap and production vault costs are not inferred from these traffic counts.

Release probes use Rust 1.99.0, LTO enabled, one codegen unit, no stripping, and
runtime argument input plus black_box so actual feature code executes and stays
linked. The comparable default executable is 368,352 bytes; reversible is 386,872
bytes (+18,520, about 5.0%). This measures these concrete probe artifacts, not all
consumer binaries. Platform linker/std/toolchain differences require new baselines.

Portable deterministic gates run inside both default and reversible harnesses:
planner at most three allocations, constructor exactly one nonempty output
allocation requesting planned bytes, irreversible end-to-end at most four, and
this synthetic reversible sink at most accepted captures plus seven. Empty input
needs zero allocations. These bounds are observed and reflect the present storage
contract; a justified architecture change requires explicit gate/baseline review,
not silent removal. Output capacities are asserted on every measured case.

This is the first measured engine baseline; no speedup over an earlier runtime
is claimed. Three repetitions expose local timing noise but do not establish
tight statistical confidence. The recorded expansion-constructor samples have
a maximum/minimum ratio of about 51.7x; its cause was not established. Across
local recording trials the 100,000-finding irreversible end-to-end median ranged
from roughly 10.4 ms to 76.7 ms (about 7.4x). Median filtering cannot establish a
latency tail or stable host performance. A final immediate same-host comparison
passed, but the provisional 3x budget can still flag future noisy runs; investigate
and repeat comparable measurements before attributing a regression to code. Provisional same-host budgets are median runtime at most 3x baseline
with a 1,000 ns noise floor; comparable probe growth at most 10% or 16 KiB (whichever
is larger). These intentionally coarse review triggers reflect observed variability
and small-toolchain/linker drift, not universal SLAs. Read budget values from the
baseline JSON. Comparison also requires the complete workload/stage key set and
three samples per group; deleting a measured workload cannot silently remove its
regression gate. A negative missing-workload check failed as expected. Same-host
checking passed after recording. CI runs portable
allocation/capacity gates and fuzz smoke across the feature/toolchain matrix; it
does not enforce macOS timings or claim results while billing blocks execution.

## Supported scope and remaining gates

Declared conservative supported Rust minimum: 1.98.1, locally tested with fmt,
all-target/all-feature Clippy, default/reversible tests; current stable 1.99.0 also
passes. This is the lowest *qualified* toolchain, not a claim that older compilers
cannot build the source. No dependencies require sibling versions: pinned public
getter mapping for core 0.1.0-beta.14 / Fastner unpublished 0.0.0 and the vault
0.1.0-beta.5 token profile are source evidence, not qualified production adapters.
Features are default-empty and optional dependency-free reversible only. CI is
configured for stable and 1.98.1 on Linux/macOS/Windows sequentially; Linux/Windows
execution is not qualified because Actions billing/spending blocks it. WASM, FFI,
streaming, production vault recovery/authority, and full token marker parity are
not supported claims.

Foundation tests include 512 independent bitmap-oracle cases, 128 construction
cases, real resource boundaries, 20,000 bounded fuzz mutations, and dummy transaction
failure conformance. Coverage-guided/sanitizer campaigns and allocation-failure
injection remain gaps. `release-status.json` records passed versus blocked gates;
`release_gate.py` deliberately exits 1 and reports PUBLIC RELEASE NOT READY while
any gate is blocked. This snapshot does not rerun tests or override maintainer
publication review. Current blockers include platform CI, production vault adapter
and marker parity, extended campaigns, LICENSE/public release governance, and
private vulnerability reporting setup. Completing the foundation epic does not
remove these public-release blockers or authorize publishing a crate.
