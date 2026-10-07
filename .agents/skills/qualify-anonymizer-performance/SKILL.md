---
name: qualify-anonymizer-performance
description: "Measure anonymizer planner/constructor cost, allocations, feature binary size, and v0.1 release evidence; use for issue #9 or performance-sensitive architecture changes."
---

# Qualify anonymizer performance

Read `AGENTS.md`, authoritative documents, issue #9 and epic #1, actual Cargo/features/CI, and existing baselines. Do not create executable benchmark tooling unless implementation or benchmark work is in scope.

Use synthetic reproducible workloads varying input bytes, finding count, overlap density, source count, irreversible/reversible mode, and replacement expansion. Include zero findings, high-count worst cases, and repeated small messages. Measure planner and constructor separately from recognizer costs and also examine end-to-end composition.

Record toolchain, supported sibling versions, feature set, workload generation, machine/build settings including LTO, commands, and measurement method. Measure time/throughput, allocation count/bytes and peak memory where supported. Compare default vs optional integrations and report binary-size deltas using comparable artifacts. Never put matched values, real tokens, or mappings in reports.

For architecture changes, compare before/after under equivalent settings and describe uncertainty or noise. Do not accept a microbenchmark improvement that hides an end-to-end regression. Set regression thresholds from evidence and product needs, not invented universal numbers.

For release qualification, check reproducibility in CI or a documented workflow, runtime/toolchain and dependency support, feature matrix, performance/size baselines, property/fuzz status, vault conformance, and non-goals such as streaming. Mark missing or unexecuted gates explicitly; a planned benchmark or single passing test is not v0.1 readiness.
