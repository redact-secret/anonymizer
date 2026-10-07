//! Comparable release executable exercising the public engine for feature size.
use anonymizer::{
    anonymize, Confidence, FindingAction, FindingKind, FindingSource, SourceFinding, Span,
};
fn main() {
    let input = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "synthetic-value".into());
    if input.is_empty() {
        return;
    }
    let finding = SourceFinding {
        span: Span {
            start: 0,
            end: input.len(),
        },
        kind: FindingKind::Other,
        source: FindingSource::Caller(1),
        confidence: Confidence::Unknown,
        action: FindingAction::Redact,
    };
    let output = anonymize(&input, &[finding]).expect("synthetic probe failed");
    std::hint::black_box(output.text().len());
    #[cfg(feature = "reversible")]
    {
        struct Dummy;
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
                Ok(captures
                    .iter()
                    .map(|_| "<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>".into())
                    .collect())
            }
            fn commit(&mut self) -> Result<(), ()> {
                Ok(())
            }
            fn abort(&mut self) -> Result<(), ()> {
                Ok(())
            }
        }
        let output = anonymizer::anonymize_reversible(
            &input,
            &[finding],
            &mut Dummy,
            anonymizer::CaptureLimits::default(),
        )
        .expect("synthetic reversible probe failed");
        std::hint::black_box(output.text().len());
    }
}
