//! Isolated allocation instrumentation; runtime library remains forbid(unsafe_code).
use anonymizer::{
    anonymize, construct, plan_irreversible, Confidence, FindingAction, FindingKind, FindingSource,
    PlanLimits, SourceFinding, Span,
};
use std::{hint::black_box, time::Instant};
#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::{allocator_smoke, measured};
struct Workload {
    name: &'static str,
    input: String,
    findings: Vec<SourceFinding>,
    sources: usize,
    overlap: &'static str,
    iterations: usize,
}
fn fixture(
    name: &'static str,
    bytes: usize,
    count: usize,
    width: usize,
    sources: usize,
    overlap: &'static str,
    iterations: usize,
) -> Workload {
    let input = "x".repeat(bytes);
    let findings = (0..count)
        .map(|i| {
            let start = if overlap == "dense" {
                i % (bytes - width)
            } else {
                i * (bytes / count.max(1))
            };
            SourceFinding {
                span: Span {
                    start,
                    end: (start + width).min(bytes),
                },
                kind: FindingKind::Other,
                source: match i % sources {
                    0 => FindingSource::Core,
                    1 => FindingSource::Fastner,
                    n => FindingSource::Caller(n as u16),
                },
                confidence: Confidence::Unknown,
                action: FindingAction::Redact,
            }
        })
        .collect();
    Workload {
        name,
        input,
        findings,
        sources,
        overlap,
        iterations,
    }
}
fn row(w: &Workload, mode: &str, stage: &str, ns: u128, calls: usize, bytes: usize, output: usize) {
    println!(
        "{},{},{},{},{},{},{},{},{},{},{},{}",
        w.name,
        mode,
        stage,
        w.input.len(),
        w.findings.len(),
        w.sources,
        w.overlap,
        w.iterations,
        ns / w.iterations as u128,
        calls,
        bytes,
        output
    );
}
#[cfg(feature = "reversible")]
#[derive(Default)]
struct Dummy;
#[cfg(feature = "reversible")]
impl anonymizer::TokenSink for Dummy {
    type Error = ();
    fn begin(&mut self) -> Result<(), ()> {
        Ok(())
    }
    fn stage(
        &mut self,
        captures: &[anonymizer::Capture<'_>],
        _: anonymizer::CaptureLimits,
    ) -> Result<Vec<String>, ()> {
        // Synthetic deterministic conformance tokens only. This benchmark sink
        // has no mapping store, authority, cryptography, or production identity.
        let mut tokens = Vec::new();
        tokens.try_reserve_exact(captures.len()).map_err(|_| ())?;
        for index in 0..captures.len() {
            let mut token = String::new();
            token.try_reserve_exact(32).map_err(|_| ())?;
            token.push_str("<rsv_");
            let mut ordinal = index;
            for _ in 0..26 {
                token.push(char::from(b'a' + (ordinal % 26) as u8));
                ordinal /= 26;
            }
            token.push('>');
            tokens.push(token);
        }
        Ok(tokens)
    }
    fn commit(&mut self) -> Result<(), ()> {
        Ok(())
    }
    fn abort(&mut self) -> Result<(), ()> {
        Ok(())
    }
}
fn main() {
    allocator_smoke();
    println!("workload,mode,stage,input_bytes,findings,sources,overlap,iterations,ns_per_op,allocation_calls,requested_bytes,output_bytes");
    let workloads = [
        fixture("empty", 0, 0, 0, 1, "none", 1000),
        fixture("small-log-core", 256, 4, 8, 1, "none", 2000),
        fixture("small-log", 256, 4, 8, 3, "none", 2000),
        fixture("expansion", 256, 64, 1, 3, "none", 500),
        fixture("shrinkage", 65536, 16, 1024, 3, "none", 100),
        fixture("large-input", 1_048_576, 128, 16, 3, "none", 20),
        fixture("high-count", 100_000, 100_000, 1, 3, "none", 3),
        fixture("dense-overlap", 65536, 50_000, 1024, 3, "dense", 5),
    ];
    for w in &workloads {
        let (plan, calls, bytes) =
            measured(|| plan_irreversible(&w.input, &w.findings, PlanLimits::default()).unwrap());
        let output_len = plan.output_bytes();
        assert!(calls <= 3, "planner allocation gate");
        let start = Instant::now();
        for _ in 0..w.iterations {
            black_box(
                plan_irreversible(
                    black_box(&w.input),
                    black_box(&w.findings),
                    PlanLimits::default(),
                )
                .unwrap(),
            );
        }
        row(
            w,
            "irreversible",
            "planner",
            start.elapsed().as_nanos(),
            calls,
            bytes,
            output_len,
        );
        let (output, calls, bytes) = measured(|| construct(plan).unwrap());
        assert!(
            calls == usize::from(output_len > 0) && bytes == output_len,
            "constructor one-allocation gate"
        );
        assert!(output.text().len() == output_len, "output capacity gate");
        drop(output);
        let mut elapsed = 0;
        for _ in 0..w.iterations {
            let plan = plan_irreversible(&w.input, &w.findings, PlanLimits::default()).unwrap();
            let start = Instant::now();
            let output = black_box(construct(black_box(plan)).unwrap());
            elapsed += start.elapsed().as_nanos();
            black_box(output);
        }
        row(
            w,
            "irreversible",
            "constructor",
            elapsed,
            calls,
            bytes,
            output_len,
        );
        let (output, calls, bytes) = measured(|| anonymize(&w.input, &w.findings).unwrap());
        drop(output);
        assert!(calls <= 4, "end-to-end allocation gate");
        let start = Instant::now();
        for _ in 0..w.iterations {
            black_box(anonymize(black_box(&w.input), black_box(&w.findings)).unwrap());
        }
        row(
            w,
            "irreversible",
            "end-to-end",
            start.elapsed().as_nanos(),
            calls,
            bytes,
            output_len,
        );
        #[cfg(feature = "reversible")]
        {
            let mut sink = Dummy;
            let limits = anonymizer::CaptureLimits::default();
            let (output, calls, bytes) = measured(|| {
                anonymizer::anonymize_reversible(&w.input, &w.findings, &mut sink, limits).unwrap()
            });
            let capture_count = output.manifest().replacements().len();
            let output_len = output.text().len();
            drop(output);
            assert!(
                calls <= capture_count + 7,
                "dummy reversible allocation gate"
            );
            let start = Instant::now();
            for _ in 0..w.iterations {
                black_box(
                    anonymizer::anonymize_reversible(
                        black_box(&w.input),
                        black_box(&w.findings),
                        &mut sink,
                        limits,
                    )
                    .unwrap(),
                );
            }
            row(
                w,
                "reversible-dummy",
                "end-to-end",
                start.elapsed().as_nanos(),
                calls,
                bytes,
                output_len,
            );
        }
    }
}
