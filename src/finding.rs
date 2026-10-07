//! Compact, text-free recognizer metadata. No authority is encoded here.

/// Half-open UTF-8 byte offsets into the exact original input.
/// Validation occurs against input during normalization before any slicing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Span {
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset; must be greater than start.
    pub end: usize,
}

/// Stable source identity, independent of submission order.
/// Caller IDs are trusted host registration numbers, never value-derived IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FindingSource {
    /// Canonical policy-applied core result.
    Core,
    /// Statistical NER result.
    Fastner,
    /// Host-registered recognizer; equal IDs denote the same source.
    Caller(u16),
}

/// Closed, bounded transformation category. Never carries recognizer strings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FindingKind {
    /// Credential or authentication material.
    Credential,
    /// Person name.
    Person,
    /// Electronic mail address.
    Email,
    /// Phone number.
    Phone,
    /// Network address.
    NetworkAddress,
    /// Postal address.
    Address,
    /// Other explicitly accepted sensitive type.
    Other,
}

/// Ordinal evidence is not a calibrated probability.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Evidence {
    /// Weak evidence.
    Low,
    /// Moderate evidence.
    Medium,
    /// Strong evidence.
    High,
}

/// Compact confidence with its interpretation preserved.
/// Discriminant ordering is canonicalization only, not cross-model precedence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Confidence {
    /// No confidence supplied.
    Unknown,
    /// Ordinal core evidence.
    Ordinal(Evidence),
    /// Calibrated fixed-point probability: 0..=65535 maps to 0..=1.
    Calibrated(u16),
}

/// Upstream policy intent retained without making authorization decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FindingAction {
    /// Replace this finding.
    Redact,
    /// Refuse onward transformation; not equivalent to redaction.
    Block,
    /// Report only; preserve source bytes.
    Warn,
    /// Informational finding; preserve source bytes.
    Allow,
}

/// Caller-supplied metadata with no matched text or open-ended identifiers.
/// Source claims must come from trusted host wiring, not model output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceFinding {
    /// Original-input UTF-8 byte range.
    pub span: Span,
    /// Bounded sensitive category.
    pub kind: FindingKind,
    /// Stable host-controlled source.
    pub source: FindingSource,
    /// Evidence semantics retained from the recognizer.
    pub confidence: Confidence,
    /// Canonical policy intent.
    pub action: FindingAction,
}
