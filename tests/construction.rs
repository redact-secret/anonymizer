use anonymizer::{
    anonymize, construct, plan_irreversible, AnonymizeError, Confidence, FindingAction,
    FindingKind, FindingSource, PlanLimits, SourceFinding, Span,
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
fn constructs_adjacent_unicode_expansion_and_shrinkage() {
    let input = "가🧪 -- synthetic-long-name!";
    let output = anonymize(input, &[finding(0, 3), finding(3, 7), finding(11, 30)]).unwrap();
    assert_eq!(output.text(), "<PERSON_1><PERSON_2> -- <PERSON_3>!");
    assert_eq!(output.text().len(), output.manifest().output_bytes());
    assert!(!format!("{output:?}").contains("synthetic-long-name"));
    assert!(!format!("{output:?}").contains(" -- "));
    let (text, manifest) = output.into_parts();
    assert_eq!(text.len(), manifest.output_bytes());
}
#[test]
fn constructor_respects_configured_limits_and_collision_failure() {
    let fs = [finding(0, 1), finding(1, 2)];
    let limits = PlanLimits {
        max_replacements: 2,
        ..Default::default()
    };
    let output = construct(plan_irreversible("ab", &fs, limits).unwrap()).unwrap();
    assert_eq!(output.text(), "<PERSON_1><PERSON_2>");
    assert_eq!(
        plan_irreversible(
            "ab",
            &fs,
            PlanLimits {
                max_replacements: 1,
                ..limits
            }
        )
        .unwrap_err(),
        AnonymizeError::ReplacementLimit
    );
    assert_eq!(
        anonymize("<PERSON_1>", &[finding(0, 10)]).unwrap_err(),
        AnonymizeError::PlaceholderCollision
    );
    assert_eq!(anonymize("<PERSON_1>", &[]).unwrap().text(), "<PERSON_1>");
}
#[test]
fn generated_property_preserves_every_unselected_byte_exactly() {
    let alphabet = ["a", "가", "🧪", "\r\n", "e\u{301}", "\u{202e}", " "];
    for seed in 0..128usize {
        let mut input = String::new();
        let mut fs = Vec::new();
        for i in 0..48usize {
            let start = input.len();
            input.push_str(alphabet[(seed + i * 5) % alphabet.len()]);
            if (seed + i * 3) % 7 < 2 {
                fs.push(finding(start, input.len()));
            }
        }
        let output = anonymize(&input, &fs).unwrap();
        let mut source_cursor = 0;
        let mut output_cursor = 0;
        for replacement in output.manifest().replacements() {
            let span = replacement.accepted().span();
            let untouched = &input.as_bytes()[source_cursor..span.start];
            assert_eq!(
                &output.text().as_bytes()[output_cursor..output_cursor + untouched.len()],
                untouched
            );
            output_cursor += untouched.len();
            let label = format!("<PERSON_{}>", replacement.identity().number());
            assert_eq!(
                &output.text()[output_cursor..output_cursor + label.len()],
                label
            );
            output_cursor += label.len();
            source_cursor = span.end;
        }
        assert_eq!(
            &output.text().as_bytes()[output_cursor..],
            &input.as_bytes()[source_cursor..]
        );
        assert_eq!(output.text().len(), output.manifest().output_bytes());
    }
}
