//! Forward transformation scaffolding for caller-supplied recognizer findings.
//!
//! This initial API only preserves input. Detection, authorization, mapping
//! persistence, cryptography, and restoration are outside this crate's ownership.
//! Finding submission and transformation contracts are introduced separately.
#![forbid(unsafe_code)]

mod arbitration;
mod construction;
mod normalization;
mod planning;

/// Fixed, source-free failures safe to format in diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnonymizeError {
    /// Input exceeds the initial 16 MiB whole-input bound.
    InputLimit,
    /// The output allocation could not be reserved.
    Allocation,
}

impl std::fmt::Display for AnonymizeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InputLimit => "input limit exceeded",
            Self::Allocation => "output allocation failed",
        })
    }
}

impl std::error::Error for AnonymizeError {}

/// Preserve input through the initial whole-input pipeline.
///
/// This scaffold performs no detection or replacement and provides no claim of
/// sanitized output. Input is borrowed; only the final output is allocated.
/// The API is provisional until the finding and replacement contracts land.
pub fn anonymize(input: &str) -> Result<String, AnonymizeError> {
    let normalized = normalization::normalize(input)?;
    let accepted = arbitration::arbitrate(normalized);
    let plan = planning::plan(accepted);
    construction::construct(plan)
}
