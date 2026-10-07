use crate::{
    normalize_findings, AnonymizeError, FindingAction, FindingKind, FindingSource, Limits,
    SourceFinding, Span,
};

/// Safe explanation of an accepted replacement span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArbitrationReason {
    /// One unique redaction finding.
    Single,
    /// Multiple distinct findings agree on exactly the same byte range.
    ExactAgreement,
    /// The union of a connected component of overlapping redaction ranges.
    OverlapUnion,
}

/// Immutable accepted range and text-free dominant-label metadata.
/// The dominant finding labels the union; losing findings do not lose coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcceptedSpan {
    span: Span,
    dominant: SourceFinding,
    contributors: usize,
    reason: ArbitrationReason,
}

impl AcceptedSpan {
    /// Sorted, non-overlapping replacement range in the original input.
    pub fn span(&self) -> Span {
        self.span
    }
    /// Deterministically selected original finding used for the union label.
    pub fn dominant(&self) -> SourceFinding {
        self.dominant
    }
    /// Unique full-metadata redaction findings contributing to this union.
    pub fn contributors(&self) -> usize {
        self.contributors
    }
    /// Safe explanation; never contains plaintext or authority state.
    pub fn reason(&self) -> ArbitrationReason {
        self.reason
    }
}

/// Normalize and arbitrate caller-supplied findings in one sort and one sweep.
///
/// Every Block rejects the whole request. Warn/Allow do not create replacement
/// spans or suppress Redact. Strictly overlapping redactions are unioned;
/// adjacent spans remain separate. This preserves every submitted redaction byte,
/// including canonical core coverage, with no confidence threshold. Union labels
/// follow the explicit decision table in ARCHITECTURE.md, not coverage priority.
pub fn arbitrate_findings(
    input: &str,
    findings: &[SourceFinding],
    limits: Limits,
) -> Result<Vec<AcceptedSpan>, AnonymizeError> {
    let normalized = normalize_findings(input, findings, limits)?;
    if normalized.iter().any(|f| f.action == FindingAction::Block) {
        return Err(AnonymizeError::Blocked);
    }
    let mut accepted: Vec<AcceptedSpan> = Vec::new();
    accepted
        .try_reserve_exact(normalized.len())
        .map_err(|_| AnonymizeError::Allocation)?;
    for finding in normalized {
        if finding.action != FindingAction::Redact {
            continue;
        }
        if let Some(last) = accepted.last_mut() {
            if finding.span.start < last.span.end {
                if last.reason != ArbitrationReason::OverlapUnion {
                    last.reason = if finding.span == last.span {
                        ArbitrationReason::ExactAgreement
                    } else {
                        ArbitrationReason::OverlapUnion
                    };
                }
                last.span.end = last.span.end.max(finding.span.end);
                last.contributors += 1;
                if label_key(finding) < label_key(last.dominant) {
                    last.dominant = finding;
                }
                continue;
            }
        }
        accepted.push(AcceptedSpan {
            span: finding.span,
            dominant: finding,
            contributors: 1,
            reason: ArbitrationReason::Single,
        });
    }
    Ok(accepted)
}

// This total key selects metadata only. It never excludes any redaction bytes.
fn label_key(finding: SourceFinding) -> (u8, u8, u16, SourceFinding) {
    let (source, registration) = match finding.source {
        FindingSource::Core => (0, 0),
        FindingSource::Caller(id) => (1, id),
        FindingSource::Fastner => (2, 0),
    };
    let kind = if finding.kind == FindingKind::Credential {
        0
    } else {
        1
    };
    (source, kind, registration, finding)
}
