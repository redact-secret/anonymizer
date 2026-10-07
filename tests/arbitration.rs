use anonymizer::{
    arbitrate_findings, AnonymizeError, ArbitrationReason, Confidence, FindingAction, FindingKind,
    FindingSource, Limits, SourceFinding, Span,
};
fn finding(start: usize, end: usize, source: FindingSource, kind: FindingKind) -> SourceFinding {
    SourceFinding {
        span: Span { start, end },
        source,
        kind,
        confidence: Confidence::Unknown,
        action: FindingAction::Redact,
    }
}
fn run(findings: &[SourceFinding]) -> Vec<anonymizer::AcceptedSpan> {
    arbitrate_findings("0123456789", findings, Limits::default()).unwrap()
}
#[test]
fn unions_contained_partial_and_transitive_overlaps_without_losing_coverage() {
    let core = finding(1, 6, FindingSource::Core, FindingKind::Credential);
    let mut narrow = finding(2, 3, FindingSource::Fastner, FindingKind::Person);
    narrow.confidence = Confidence::Calibrated(u16::MAX);
    let a = finding(5, 8, FindingSource::Caller(1), FindingKind::Email);
    let b = finding(7, 9, FindingSource::Fastner, FindingKind::Person);
    for fs in [
        [core, narrow, a, b],
        [b, narrow, core, a],
        [a, b, narrow, core],
    ] {
        let accepted = run(&fs);
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].span(), Span { start: 1, end: 9 });
        assert_eq!(accepted[0].dominant(), core);
        assert_eq!(accepted[0].contributors(), 4);
        assert_eq!(accepted[0].reason(), ArbitrationReason::OverlapUnion);
    }
}
#[test]
fn exact_agreement_is_distinguished_from_duplicates_and_adjacency() {
    let a = finding(0, 2, FindingSource::Fastner, FindingKind::Person);
    let b = finding(0, 2, FindingSource::Caller(3), FindingKind::Person);
    let c = finding(2, 4, FindingSource::Core, FindingKind::Credential);
    let accepted = run(&[a, a, b, c]);
    assert_eq!(accepted.len(), 2);
    assert_eq!(accepted[0].contributors(), 2);
    assert_eq!(accepted[0].reason(), ArbitrationReason::ExactAgreement);
    assert_eq!(accepted[0].dominant(), b);
    assert_eq!(accepted[1].reason(), ArbitrationReason::Single);
}
#[test]
fn block_fails_globally_and_report_actions_never_suppress_redaction() {
    let a = finding(1, 3, FindingSource::Fastner, FindingKind::Person);
    let mut report = finding(0, 9, FindingSource::Core, FindingKind::Credential);
    for action in [FindingAction::Warn, FindingAction::Allow] {
        report.action = action;
        let accepted = run(&[a, report]);
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].span(), a.span);
    }
    report.action = FindingAction::Block;
    assert_eq!(
        arbitrate_findings("0123456789", &[a, report], Limits::default()),
        Err(AnonymizeError::Blocked)
    );
}
#[test]
fn equal_source_labels_and_unicode_are_deterministic() {
    let a = finding(0, 3, FindingSource::Caller(2), FindingKind::Person);
    let b = finding(0, 3, FindingSource::Caller(2), FindingKind::Credential);
    let c = finding(3, 7, FindingSource::Fastner, FindingKind::Person);
    let expected = arbitrate_findings("가🧪", &[a, b, c], Limits::default()).unwrap();
    assert_eq!(expected[0].dominant(), b);
    assert_eq!(
        arbitrate_findings("가🧪", &[c, b, a, b], Limits::default()).unwrap(),
        expected
    );
    assert!(arbitrate_findings("", &[], Limits::default())
        .unwrap()
        .is_empty());
}
#[test]
fn long_overlap_chain_has_one_bounded_union() {
    let input = "x".repeat(20_002);
    let fs: Vec<_> = (0..20_000)
        .map(|i| finding(i, i + 2, FindingSource::Fastner, FindingKind::Person))
        .collect();
    let accepted = arbitrate_findings(&input, &fs, Limits::default()).unwrap();
    assert_eq!(accepted.len(), 1);
    assert_eq!(
        accepted[0].span(),
        Span {
            start: 0,
            end: 20_001
        }
    );
    assert_eq!(accepted[0].contributors(), fs.len());
}
