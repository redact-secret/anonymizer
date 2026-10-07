#[path = "../fuzz/targets/span_pipeline.rs"]
mod span_pipeline;
use anonymizer::{
    anonymize, normalize_findings, plan_irreversible, AnonymizeError, Confidence, FindingAction,
    FindingKind, FindingSource, Limits, PlanLimits, SourceFinding, Span,
};
fn finding(start: usize, end: usize) -> SourceFinding {
    SourceFinding {
        span: Span { start, end },
        kind: FindingKind::Other,
        source: FindingSource::Caller(1),
        confidence: Confidence::Unknown,
        action: FindingAction::Redact,
    }
}
#[test]
fn independent_generated_coverage_and_preservation_properties() {
    let alphabet = [
        "x", "가", "🧪", "e\u{301}", "\u{202e}", "\u{200b}", "\0", "\r\n",
    ];
    let mut state = 0x1428_5eed_u64;
    for case in 0..512usize {
        let mut input = String::new();
        let mut boundaries = vec![0];
        for index in 0..24 {
            input.push_str(alphabet[(case + index * 3) % alphabet.len()]);
            boundaries.push(input.len());
        }
        let mut findings = Vec::new();
        for index in 0..32 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let first = state as usize % 24;
            let last = first + 1 + (state >> 32) as usize % (24 - first);
            let mut f = finding(boundaries[first], boundaries[last]);
            f.source = match index % 3 {
                0 => FindingSource::Core,
                1 => FindingSource::Fastner,
                _ => FindingSource::Caller((index % 7) as u16),
            };
            f.kind = if index % 2 == 0 {
                FindingKind::Credential
            } else {
                FindingKind::Person
            };
            f.confidence = Confidence::Calibrated(state as u16);
            if index % 11 == 0 {
                f.action = FindingAction::Warn;
            }
            findings.push(f);
            if index % 9 == 0 {
                findings.push(f);
            }
        }
        span_pipeline::check_case(&input, &findings);
    }
}
#[test]
fn replay_external_utf8_and_extreme_span_seed_corpus() {
    for corpus in [
        include_str!("../fuzz/corpus/ascii.hex"),
        include_str!("../fuzz/corpus/unicode.hex"),
        include_str!("../fuzz/corpus/invalid-utf8.hex"),
        include_str!("../fuzz/corpus/extreme.hex"),
        include_str!("../fuzz/corpus/literal.hex"),
    ] {
        let data: Vec<_> = corpus
            .split_whitespace()
            .map(|h| u8::from_str_radix(h, 16).unwrap())
            .collect();
        span_pipeline::fuzz_target(&data);
    }
    for bytes in [
        &[0xff][..],
        &[0xc0, 0xaf],
        &[0xf0, 0x9f],
        &[0xed, 0xa0, 0x80],
    ] {
        assert!(std::str::from_utf8(bytes).is_err());
    }
}
#[test]
fn configured_and_default_count_limits_apply_before_duplicate_collapse() {
    let f = finding(0, 1);
    let at_limit = vec![f; 100_000];
    assert_eq!(
        normalize_findings("x", &at_limit, Limits::default())
            .unwrap()
            .len(),
        1
    );
    let mut excess = at_limit;
    excess.push(f);
    assert_eq!(
        normalize_findings("x", &excess, Limits::default()).unwrap_err(),
        AnonymizeError::FindingLimit
    );
    let limits = PlanLimits {
        max_placeholder_bytes: 8,
        ..Default::default()
    };
    assert_eq!(
        plan_irreversible("x", &[f], limits).unwrap_err(),
        AnonymizeError::PlaceholderLimit
    );
    let limits = PlanLimits {
        max_output_growth: 8,
        ..Default::default()
    };
    assert!(plan_irreversible("x", &[f], limits).is_ok());
    assert_eq!(
        plan_irreversible(
            "x",
            &[f],
            PlanLimits {
                max_output_growth: 7,
                ..limits
            }
        )
        .unwrap_err(),
        AnonymizeError::OutputLimit
    );
}
#[test]
fn fixed_errors_do_not_echo_synthetic_source_or_input_identifiers() {
    let input = "synthetic-secret-name";
    for span in [
        Span { start: 0, end: 0 },
        Span {
            start: usize::MAX,
            end: 1,
        },
        Span {
            start: 1,
            end: usize::MAX,
        },
    ] {
        let error = anonymize(input, &[finding(span.start, span.end)]).unwrap_err();
        assert!(!format!("{error:?}: {error}").contains(input));
    }
}
