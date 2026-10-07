use anonymizer::{
    normalize_findings, AnonymizeError, Confidence, Evidence, FindingAction, FindingKind,
    FindingSource, Limits, SourceFinding, Span,
};

fn finding(start: usize, end: usize) -> SourceFinding {
    SourceFinding {
        span: Span { start, end },
        kind: FindingKind::Person,
        source: FindingSource::Fastner,
        confidence: Confidence::Calibrated(42_000),
        action: FindingAction::Redact,
    }
}

#[test]
fn validates_before_sorting_and_preserves_unicode_offsets() {
    let input = "a🧪가e\u{301}";
    let findings = [finding(5, 8), finding(1, 5), finding(0, 1), finding(8, 11)];
    let output = normalize_findings(input, &findings, Limits::default()).unwrap();
    assert_eq!(
        output.iter().map(|f| f.span.start).collect::<Vec<_>>(),
        [0, 1, 5, 8]
    );
    for f in output {
        assert!(input.get(f.span.start..f.span.end).is_some());
    }
}

#[test]
fn rejects_all_invalid_span_classes_with_fixed_errors() {
    for (span, expected) in [
        ((0, 0), AnonymizeError::InvalidSpan),
        ((4, 1), AnonymizeError::InvalidSpan),
        ((0, 99), AnonymizeError::InvalidSpan),
        ((usize::MAX, usize::MAX), AnonymizeError::InvalidSpan),
        ((1, 3), AnonymizeError::InvalidBoundary),
        ((0, 2), AnonymizeError::InvalidBoundary),
    ] {
        let error =
            normalize_findings("가a", &[finding(span.0, span.1)], Limits::default()).unwrap_err();
        assert_eq!(error, expected);
        assert!(!error.to_string().contains('가'));
    }
}

#[test]
fn canonicalization_is_permutation_independent_and_deduplicates_exact_metadata_only() {
    let a = finding(0, 2);
    let b = finding(1, 3);
    let mut c = a;
    c.source = FindingSource::Caller(7);
    c.confidence = Confidence::Ordinal(Evidence::High);
    let mut d = a;
    d.action = FindingAction::Warn;
    let expected = normalize_findings("abcd", &[a, b, c, d], Limits::default()).unwrap();
    for submitted in [[a, b, c, d, a], [d, c, a, b, a], [b, a, d, a, c]] {
        assert_eq!(
            normalize_findings("abcd", &submitted, Limits::default()).unwrap(),
            expected
        );
    }
    assert_eq!(expected.len(), 4);
}

#[test]
fn checks_limits_before_deduplication_and_accepts_adjacent_spans() {
    let limits = Limits {
        max_input_bytes: 2,
        max_findings: 2,
    };
    assert_eq!(
        normalize_findings("abc", &[], limits),
        Err(AnonymizeError::InputLimit)
    );
    let a = finding(0, 1);
    assert_eq!(
        normalize_findings("ab", &[a, a, a], limits),
        Err(AnonymizeError::FindingLimit)
    );
    assert_eq!(
        normalize_findings("ab", &[a, finding(1, 2)], limits)
            .unwrap()
            .len(),
        2
    );
    assert!(normalize_findings(
        "",
        &[],
        Limits {
            max_input_bytes: 0,
            max_findings: 0
        }
    )
    .unwrap()
    .is_empty());
}
