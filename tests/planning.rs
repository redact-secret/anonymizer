use anonymizer::{
    plan_irreversible, AnonymizeError, Confidence, FindingAction, FindingKind, FindingSource,
    PlanLimits, SourceFinding, Span,
};
fn finding(start: usize, end: usize) -> SourceFinding {
    SourceFinding {
        span: Span { start, end },
        kind: FindingKind::Person,
        source: FindingSource::Caller(1),
        confidence: Confidence::Unknown,
        action: FindingAction::Redact,
    }
}
#[test]
fn immutable_plan_is_deterministic_text_free_and_exactly_sized() {
    let input = "synthetic-name🧪abc";
    let a = finding(0, 14);
    let b = finding(18, 21);
    let first = plan_irreversible(input, &[a, b], PlanLimits::default()).unwrap();
    let second = plan_irreversible(input, &[b, a, a], PlanLimits::default()).unwrap();
    assert_eq!(first.manifest(), second.manifest());
    assert_eq!(first.output_bytes(), "<PERSON_1>🧪<PERSON_2>".len());
    let rows = first.manifest().replacements();
    assert_eq!(rows[0].identity().number(), 1);
    assert_eq!(rows[1].identity().number(), 2);
    assert_eq!(first.manifest().input_bytes(), input.len());
    for diagnostic in [format!("{first:?}"), format!("{:?}", first.manifest())] {
        assert!(!diagnostic.contains("synthetic-name"));
        assert!(!diagnostic.contains('🧪'));
    }
}
#[test]
fn planning_failures_return_only_fixed_error_without_any_partial_output() {
    let input = "abc";
    let fs = [finding(0, 1), finding(1, 2)];
    for (limits, expected) in [
        (
            PlanLimits {
                max_replacements: 1,
                ..Default::default()
            },
            AnonymizeError::ReplacementLimit,
        ),
        (
            PlanLimits {
                max_placeholder_bytes: 9,
                ..Default::default()
            },
            AnonymizeError::PlaceholderLimit,
        ),
        (
            PlanLimits {
                max_output_bytes: 10,
                ..Default::default()
            },
            AnonymizeError::OutputLimit,
        ),
        (
            PlanLimits {
                max_output_growth: 0,
                ..Default::default()
            },
            AnonymizeError::OutputLimit,
        ),
    ] {
        assert_eq!(plan_irreversible(input, &fs, limits).unwrap_err(), expected);
    }
    assert_eq!(input, "abc");
    assert_eq!(
        plan_irreversible(input, &[finding(0, 99)], PlanLimits::default()).unwrap_err(),
        AnonymizeError::InvalidSpan
    );
}
#[test]
fn reserves_placeholder_namespace_and_rejects_source_equal_replacement() {
    for input in ["<PERSON_1>", "abc <PERSON_literal", "<NETWORK_ADDRESS_2>"] {
        assert_eq!(
            plan_irreversible(input, &[finding(0, input.len())], PlanLimits::default())
                .unwrap_err(),
            AnonymizeError::PlaceholderCollision
        );
    }
    assert!(plan_irreversible("<PERSON_1>", &[], PlanLimits::default()).is_ok());
    assert!(plan_irreversible("<custom>", &[finding(0, 8)], PlanLimits::default()).is_ok());
}
#[test]
fn handles_shrinkage_empty_plans_and_multi_digit_numbering() {
    let input = "x".repeat(100);
    let plan = plan_irreversible(
        &input,
        &[finding(0, 100)],
        PlanLimits {
            max_output_growth: 0,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(plan.output_bytes(), 10);
    let fs: Vec<_> = (0..10).map(|i| finding(i, i + 1)).collect();
    let plan = plan_irreversible(&input, &fs, PlanLimits::default()).unwrap();
    assert_eq!(plan.manifest().replacements()[9].identity().number(), 10);
    assert_eq!(plan.output_bytes(), 191);
    let empty = plan_irreversible("", &[], PlanLimits::default()).unwrap();
    assert_eq!(empty.output_bytes(), 0);
}
