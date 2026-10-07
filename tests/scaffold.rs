use anonymizer::{anonymize, AnonymizeError};

#[test]
fn preserves_empty_and_unicode_input_exactly() {
    for input in [
        "",
        "synthetic text",
        "가상 🧪 e\u{301}\r\n<PERSON_1>\u{202e}",
    ] {
        assert_eq!(anonymize(input).unwrap().as_bytes(), input.as_bytes());
    }
}

#[test]
fn input_bound_is_enforced_without_source_diagnostics() {
    let at_limit = "x".repeat(16 * 1024 * 1024);
    assert_eq!(anonymize(&at_limit).unwrap().len(), at_limit.len());
    let too_large = "x".repeat(16 * 1024 * 1024 + 1);
    let error = anonymize(&too_large).unwrap_err();
    assert_eq!(error, AnonymizeError::InputLimit);
    assert_eq!(error.to_string(), "input limit exceeded");
    assert_eq!(format!("{error:?}"), "InputLimit");
}
