use crate::{AnonymizeError, SourceFinding};

pub(crate) const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_FINDINGS: usize = 100_000;

/// Configurable input and submission bounds. Defaults: 16 MiB, 100,000 findings.
/// Limits are checked before allocating normalization storage; duplicates count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum original-input bytes.
    pub max_input_bytes: usize,
    /// Maximum supplied findings before deduplication.
    pub max_findings: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input_bytes: MAX_INPUT_BYTES,
            max_findings: MAX_FINDINGS,
        }
    }
}

/// Validate and canonicalize findings against the exact original input.
///
/// Rejects empty, reversed, out-of-bounds, and non-UTF-8-boundary spans. Returns
/// full-metadata unique findings in lexicographic field order, independently of
/// submission order. Overlaps remain present for the separate arbitration stage.
/// No input substring is allocated, normalized, logged, or included in errors.
pub fn normalize_findings(
    input: &str,
    findings: &[SourceFinding],
    limits: Limits,
) -> Result<Vec<SourceFinding>, AnonymizeError> {
    if input.len() > limits.max_input_bytes {
        return Err(AnonymizeError::InputLimit);
    }
    if findings.len() > limits.max_findings {
        return Err(AnonymizeError::FindingLimit);
    }
    for finding in findings {
        let span = finding.span;
        if span.start >= span.end || span.end > input.len() {
            return Err(AnonymizeError::InvalidSpan);
        }
        if !input.is_char_boundary(span.start) || !input.is_char_boundary(span.end) {
            return Err(AnonymizeError::InvalidBoundary);
        }
    }
    let mut normalized = Vec::new();
    normalized
        .try_reserve_exact(findings.len())
        .map_err(|_| AnonymizeError::Allocation)?;
    normalized.extend_from_slice(findings);
    normalized.sort_unstable();
    normalized.dedup();
    Ok(normalized)
}

pub(crate) struct Normalized<'a> {
    pub(crate) input: &'a str,
}

pub(crate) fn normalize(input: &str) -> Result<Normalized<'_>, AnonymizeError> {
    normalize_findings(input, &[], Limits::default())?;
    Ok(Normalized { input })
}
