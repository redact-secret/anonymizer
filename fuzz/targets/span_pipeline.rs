//! Dependency-free bounded byte-entry fuzz target, reusable by mutation engines.
use anonymizer::{
    anonymize, arbitrate_findings, AnonymizeError, Confidence, FindingAction, FindingKind,
    FindingSource, Limits, SourceFinding, Span,
};

/// Exercise the public pipeline after a fallible UTF-8 external-input boundary.
/// Wire: u8 text length, raw text, then [start, end, action] records. Offsets 255
/// mean usize::MAX. Text <=96 bytes, findings <=64. Invalid UTF-8 never becomes &str.
pub fn fuzz_target(data: &[u8]) {
    if data.is_empty() {
        return;
    }
    let length = usize::from(data[0]).min(96).min(data.len() - 1);
    let Ok(input) = std::str::from_utf8(&data[1..1 + length]) else {
        return;
    };
    let findings: Vec<_> = data[1 + length..]
        .as_chunks::<3>()
        .0
        .iter()
        .take(64)
        .map(|r| {
            let offset = |b| {
                if b == 255 {
                    usize::MAX
                } else {
                    usize::from(b) % (input.len() + 2)
                }
            };
            SourceFinding {
                span: Span {
                    start: offset(r[0]),
                    end: offset(r[1]),
                },
                kind: FindingKind::Other,
                source: FindingSource::Caller(1),
                confidence: Confidence::Unknown,
                action: match r[2] % 4 {
                    0 => FindingAction::Redact,
                    1 => FindingAction::Warn,
                    2 => FindingAction::Allow,
                    _ => FindingAction::Block,
                },
            }
        })
        .collect();
    check_case(input, &findings);
}

/// Independent byte-coverage oracle, not the production overlap sweep.
pub fn check_case(input: &str, findings: &[SourceFinding]) {
    let result = anonymize(input, findings);
    let invalid = findings.iter().any(|f| {
        f.span.start >= f.span.end
            || f.span.end > input.len()
            || !input.is_char_boundary(f.span.start)
            || !input.is_char_boundary(f.span.end)
    });
    if invalid {
        assert!(
            matches!(
                result,
                Err(AnonymizeError::InvalidSpan | AnonymizeError::InvalidBoundary)
            ),
            "invalid span must fail safely"
        );
        return;
    }
    if findings.iter().any(|f| f.action == FindingAction::Block) {
        assert!(
            matches!(result, Err(AnonymizeError::Blocked)),
            "block must fail globally"
        );
        return;
    }
    // Collision rejection is part of the selected placeholder grammar.
    let Ok(output) = result else {
        assert!(
            matches!(
                anonymize(input, findings),
                Err(AnonymizeError::PlaceholderCollision)
            ),
            "unexpected bounded-case failure"
        );
        return;
    };
    let accepted = arbitrate_findings(input, findings, Limits::default()).unwrap();
    let mut expected = vec![false; input.len()];
    for finding in findings
        .iter()
        .filter(|f| f.action == FindingAction::Redact)
    {
        expected[finding.span.start..finding.span.end].fill(true);
    }
    let mut actual = vec![false; input.len()];
    let mut cursor = 0;
    let mut output_cursor = 0;
    for (index, accepted) in accepted.iter().enumerate() {
        let span = accepted.span();
        assert!(
            span.start >= cursor && span.start < span.end && span.end <= input.len(),
            "sorted nonoverlap invariant"
        );
        actual[span.start..span.end].fill(true);
        let untouched = &input.as_bytes()[cursor..span.start];
        assert!(
            output.text().as_bytes()[output_cursor..output_cursor + untouched.len()] == *untouched,
            "untouched bytes changed"
        );
        output_cursor += untouched.len();
        let kind = match accepted.dominant().kind {
            FindingKind::Credential => "CREDENTIAL",
            FindingKind::Person => "PERSON",
            FindingKind::Email => "EMAIL",
            FindingKind::Phone => "PHONE",
            FindingKind::NetworkAddress => "NETWORK_ADDRESS",
            FindingKind::Address => "ADDRESS",
            FindingKind::Other => "OTHER",
        };
        let label = format!("<{kind}_{}>", index + 1);
        assert!(
            output.text()[output_cursor..].starts_with(&label),
            "placeholder identity mismatch"
        );
        output_cursor += label.len();
        cursor = span.end;
    }
    assert!(actual == expected, "redaction coverage mismatch");
    assert!(
        output.text().as_bytes()[output_cursor..] == input.as_bytes()[cursor..],
        "final untouched bytes changed"
    );
    assert!(
        output.text().len() == output.manifest().output_bytes(),
        "capacity mismatch"
    );
    let mut reversed = findings.to_vec();
    reversed.reverse();
    let repeated = anonymize(input, &reversed).unwrap();
    assert!(
        output.text() == repeated.text() && output.manifest() == repeated.manifest(),
        "permutation changed result"
    );
}
