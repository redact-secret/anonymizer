#![cfg(feature = "reversible")]
use anonymizer::{
    anonymize_reversible, AnonymizeError, Capture, CaptureLimits, Confidence, FindingAction,
    FindingKind, FindingSource, ReplacementMode, SourceFinding, Span, TokenSink,
};
const TOKEN_A: &str = "<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>";
const TOKEN_B: &str = "<rsv_bbbbbbbbbbbbbbbbbbbbbbbbbb>";
#[derive(Default)]
struct Sink {
    fail: &'static str,
    calls: Vec<&'static str>,
    committed: bool,
    staged: bool,
    abort_fails: bool,
}
impl TokenSink for Sink {
    type Error = &'static str;
    fn begin(&mut self) -> Result<(), Self::Error> {
        self.calls.push("begin");
        self.staged = true;
        if self.fail == "begin" {
            Err("synthetic-sensitive-error")
        } else {
            Ok(())
        }
    }
    fn stage(
        &mut self,
        captures: &[Capture<'_>],
        _: CaptureLimits,
    ) -> Result<Vec<String>, Self::Error> {
        self.calls.push("stage");
        assert_eq!(captures[0].value(), "synthetic-name");
        assert!(!format!("{:?}", captures[0]).contains("synthetic-name"));
        match self.fail {
            "stage" => Err("synthetic-sensitive-error"),
            "count" => Ok(vec![]),
            "grammar" => Ok(vec!["synthetic-name".into(), TOKEN_B.into()]),
            "duplicate" => Ok(vec![TOKEN_A.into(), TOKEN_A.into()]),
            "length" => Ok(vec![format!("{TOKEN_A}x"), TOKEN_B.into()]),
            _ => Ok(vec![TOKEN_A.into(), TOKEN_B.into()]),
        }
    }
    fn commit(&mut self) -> Result<(), Self::Error> {
        self.calls.push("commit");
        self.committed = true;
        if self.fail == "commit" {
            Err("synthetic-sensitive-error")
        } else {
            self.staged = false;
            Ok(())
        }
    }
    fn abort(&mut self) -> Result<(), Self::Error> {
        self.calls.push("abort");
        if self.abort_fails {
            Err("synthetic-sensitive-error")
        } else {
            self.committed = false;
            self.staged = false;
            Ok(())
        }
    }
}
fn finding(start: usize, end: usize) -> SourceFinding {
    SourceFinding {
        span: Span { start, end },
        kind: FindingKind::Person,
        source: FindingSource::Caller(1),
        confidence: Confidence::Unknown,
        action: FindingAction::Redact,
    }
}
const INPUT: &str = "synthetic-name and abc!";
fn fs() -> [SourceFinding; 2] {
    [finding(0, 14), finding(19, 22)]
}
#[test]
fn bulk_success_commits_before_return_without_token_diagnostics() {
    let mut sink = Sink::default();
    let output = anonymize_reversible(INPUT, &fs(), &mut sink, CaptureLimits::default()).unwrap();
    assert_eq!(sink.calls, ["begin", "stage", "commit"]);
    assert!(sink.committed);
    assert_eq!(output.text(), format!("{TOKEN_A} and {TOKEN_B}!"));
    assert_eq!(output.text().len(), output.manifest().output_bytes());
    assert_eq!(
        output.manifest().replacements()[0].mode(),
        ReplacementMode::Reversible
    );
    assert!(!format!("{output:?}").contains(TOKEN_A));
    assert!(!format!("{:?}", output.manifest()).contains("synthetic-name"));
}
#[test]
fn every_external_failure_aborts_and_compensates_partial_commit() {
    for phase in [
        "begin",
        "stage",
        "commit",
        "count",
        "grammar",
        "duplicate",
        "length",
    ] {
        let mut sink = Sink {
            fail: phase,
            ..Default::default()
        };
        let error =
            anonymize_reversible(INPUT, &fs(), &mut sink, CaptureLimits::default()).unwrap_err();
        assert!(matches!(
            error,
            AnonymizeError::CaptureFailed | AnonymizeError::InvalidToken
        ));
        assert_eq!(sink.calls.last(), Some(&"abort"));
        assert!(!sink.staged && !sink.committed);
        assert!(!error.to_string().contains("synthetic-sensitive-error"));
    }
}
#[test]
fn cleanup_failure_reports_residual_state_without_sink_details() {
    let mut sink = Sink {
        fail: "commit",
        abort_fails: true,
        ..Default::default()
    };
    assert_eq!(
        anonymize_reversible(INPUT, &fs(), &mut sink, CaptureLimits::default()).unwrap_err(),
        AnonymizeError::CleanupFailed
    );
    assert!(sink.committed);
}
#[test]
fn all_local_preflight_failures_and_empty_calls_avoid_sink() {
    let mut sink = Sink::default();
    let limits = CaptureLimits {
        max_captures: 1,
        ..Default::default()
    };
    assert_eq!(
        anonymize_reversible(INPUT, &fs(), &mut sink, limits).unwrap_err(),
        AnonymizeError::CaptureLimit
    );
    let limits = CaptureLimits {
        plan: anonymizer::PlanLimits {
            max_output_bytes: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        anonymize_reversible(INPUT, &fs(), &mut sink, limits).unwrap_err(),
        AnonymizeError::OutputLimit
    );
    let limits = CaptureLimits {
        max_token_bytes: 31,
        ..Default::default()
    };
    assert_eq!(
        anonymize_reversible(INPUT, &fs(), &mut sink, limits).unwrap_err(),
        AnonymizeError::InvalidToken
    );
    assert_eq!(
        anonymize_reversible(
            "RSV_literal",
            &[finding(0, 3)],
            &mut sink,
            CaptureLimits::default()
        )
        .unwrap_err(),
        AnonymizeError::PlaceholderCollision
    );
    assert_eq!(
        anonymize_reversible(
            TOKEN_A,
            &[finding(0, 32)],
            &mut sink,
            CaptureLimits::default()
        )
        .unwrap_err(),
        AnonymizeError::PlaceholderCollision
    );
    assert_eq!(
        anonymize_reversible(
            INPUT,
            &[finding(0, 99)],
            &mut sink,
            CaptureLimits::default()
        )
        .unwrap_err(),
        AnonymizeError::InvalidSpan
    );
    let mut blocked = finding(0, 1);
    blocked.action = FindingAction::Block;
    assert_eq!(
        anonymize_reversible(INPUT, &[blocked], &mut sink, CaptureLimits::default()).unwrap_err(),
        AnonymizeError::Blocked
    );
    let output = anonymize_reversible(TOKEN_A, &[], &mut sink, CaptureLimits::default()).unwrap();
    assert_eq!(output.text(), TOKEN_A);
    assert!(sink.calls.is_empty());
}
