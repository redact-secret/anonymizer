//! Forward transformation scaffolding for caller-supplied recognizer findings.
//!
//! This initial API only preserves input. Detection, authorization, mapping
//! persistence, cryptography, and restoration are outside this crate's ownership.
//! Finding submission and transformation contracts are introduced separately.
#![forbid(unsafe_code)]

mod arbitration;
mod construction;
mod finding;
mod normalization;
mod planning;

pub use arbitration::{arbitrate_findings, AcceptedSpan, ArbitrationReason};
pub use finding::{
    Confidence, Evidence, FindingAction, FindingKind, FindingSource, SourceFinding, Span,
};
pub use normalization::{normalize_findings, Limits};
pub use planning::{
    plan_irreversible, PlanLimits, PlannedReplacement, ReplacementIdentity, ReplacementMode,
    ReplacementPlan, TransformationManifest,
};

/// Fixed, source-free failures safe to format in diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnonymizeError {
    /// Input exceeds the initial 16 MiB whole-input bound.
    InputLimit,
    /// The output allocation could not be reserved.
    Allocation,
    /// Supplied finding count exceeds the configured bound.
    FindingLimit,
    /// Span is empty, reversed, or outside input.
    InvalidSpan,
    /// Span endpoint is not a UTF-8 character boundary.
    InvalidBoundary,
    /// Upstream policy blocks transformation of the whole input.
    Blocked,
    /// Accepted replacement count exceeds configuration.
    ReplacementLimit,
    /// Generated placeholder exceeds configured length.
    PlaceholderLimit,
    /// Input already contains a reserved placeholder prefix.
    PlaceholderCollision,
    /// Output length or positive growth exceeds configuration.
    OutputLimit,
    /// Checked capacity calculation overflowed.
    CapacityOverflow,
}

impl std::fmt::Display for AnonymizeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InputLimit => "input limit exceeded",
            Self::Allocation => "output allocation failed",
            Self::FindingLimit => "finding limit exceeded",
            Self::InvalidSpan => "invalid span",
            Self::InvalidBoundary => "invalid UTF-8 boundary",
            Self::Blocked => "transformation blocked",
            Self::ReplacementLimit => "replacement limit exceeded",
            Self::PlaceholderLimit => "placeholder limit exceeded",
            Self::PlaceholderCollision => "placeholder literal collision",
            Self::OutputLimit => "output limit exceeded",
            Self::CapacityOverflow => "capacity overflow",
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
