//! Bounded deterministic mutation/replay runner; not coverage-guided libFuzzer.
#[path = "../fuzz/targets/span_pipeline.rs"]
mod span_pipeline;
use std::time::Instant;
fn decode(hex: &str) -> Vec<u8> {
    hex.split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).expect("invalid synthetic corpus hex"))
        .collect()
}
fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|a| {
            a.parse::<usize>()
                .expect("integer iteration count required")
        })
        .unwrap_or(20_000);
    assert!(iterations <= 1_000_000, "bounded runner iteration limit");
    let corpus: Vec<_> = [
        include_str!("../fuzz/corpus/ascii.hex"),
        include_str!("../fuzz/corpus/unicode.hex"),
        include_str!("../fuzz/corpus/invalid-utf8.hex"),
        include_str!("../fuzz/corpus/extreme.hex"),
        include_str!("../fuzz/corpus/literal.hex"),
    ]
    .iter()
    .map(|s| decode(s))
    .collect();
    let start = Instant::now();
    for seed in &corpus {
        span_pipeline::fuzz_target(seed);
    }
    let mut state = 0x5eed_cafe_1234_5678u64;
    for iteration in 0..iterations {
        let mut data = corpus[iteration % corpus.len()].clone();
        for _ in 0..1 + iteration % 5 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let position = state as usize % data.len();
            data[position] = (state >> 32) as u8;
        }
        span_pipeline::fuzz_target(&data);
    }
    println!(
        "bounded fuzz replay passed: seeds={}, mutations={}, rng=5eedcafe12345678, elapsed_ms={}",
        corpus.len(),
        iterations,
        start.elapsed().as_millis()
    );
}
