use crate::{
    arbitrate_findings, AcceptedSpan, AnonymizeError, AnonymizedOutput, PlanLimits,
    PlannedReplacement, ReplacementIdentity, ReplacementMode, SourceFinding,
    TransformationManifest,
};

/// Borrowed accepted source slice for one bulk capture. Debug excludes its value.
pub struct Capture<'a> {
    value: &'a str,
    accepted: AcceptedSpan,
}
impl Capture<'_> {
    /// Original value for the trusted sink only; never log or retain independently.
    pub fn value(&self) -> &str {
        self.value
    }
    /// Safe accepted metadata. Source claims do not grant capture authority.
    pub fn accepted(&self) -> AcceptedSpan {
        self.accepted
    }
}
impl std::fmt::Debug for Capture<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Capture")
            .field("accepted", &self.accepted)
            .finish()
    }
}

/// Trusted host-provided transaction boundary; no production vault adapter exists.
///
/// Sink owns identity, mappings, policy, persistence, and keys. Before beginning,
/// host wiring must supply trusted tenant/principal/source/sink/purpose context.
/// `begin` may partially fail; `abort` must be valid after any begin attempt.
/// `stage` returns exactly one unique issued token per capture, in order, and
/// must obey supplied bounds before allocating. Tokens follow the pinned vault
/// grammar `<rsv_` + 26 lowercase base32 characters + `>`; sink must provide
/// unpredictable identities and never encode originals. Tokens confer no authority.
/// Commit must publish all staged mappings atomically. Failed commit must leave
/// mappings abortable, including compensating cleanup of external partial writes.
/// Abort is idempotent; success means no transaction mappings remain. Failed
/// abort requires host reconciliation and withholding any source-derived output.
/// Methods must not panic; cancellation/process loss cleanup belongs to the host.
/// This synchronous contract does not promise durable distributed transactions.
pub trait TokenSink {
    /// Opaque sink errors, never formatted or propagated by anonymizer.
    type Error;
    /// Start one exclusive transaction; may require abort even on error.
    fn begin(&mut self) -> Result<(), Self::Error>;
    /// Bulk borrowed capture, preserving order and honoring count/token bounds.
    fn stage(
        &mut self,
        captures: &[Capture<'_>],
        limits: CaptureLimits,
    ) -> Result<Vec<String>, Self::Error>;
    /// Commit the whole staged capture atomically, or retain abortable state.
    fn commit(&mut self) -> Result<(), Self::Error>;
    /// Remove all state from this attempted transaction, including partial commit.
    fn abort(&mut self) -> Result<(), Self::Error>;
}

/// Bounded whole-input reversible capture; defaults mirror irreversible limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureLimits {
    /// Shared input/replacement/output bounds; placeholder bound is irrelevant here.
    pub plan: PlanLimits,
    /// Maximum accepted captures submitted to sink in one bulk call.
    pub max_captures: usize,
    /// Maximum token bytes; selected profile requires exactly 32.
    pub max_token_bytes: usize,
}
impl Default for CaptureLimits {
    fn default() -> Self {
        Self {
            plan: PlanLimits::default(),
            max_captures: 100_000,
            max_token_bytes: 32,
        }
    }
}

/// Place external tokens only after full local validation and successful commit.
/// No token-to-value map or authority metadata is retained in the returned manifest.
pub fn anonymize_reversible<S: TokenSink>(
    input: &str,
    findings: &[SourceFinding],
    sink: &mut S,
    limits: CaptureLimits,
) -> Result<AnonymizedOutput, AnonymizeError> {
    let accepted = arbitrate_findings(input, findings, limits.plan.input)?;
    if accepted.len() > limits.plan.max_replacements {
        return Err(AnonymizeError::ReplacementLimit);
    }
    if accepted.len() > limits.max_captures {
        return Err(AnonymizeError::CaptureLimit);
    }
    if !accepted.is_empty() {
        if limits.max_token_bytes < 32 {
            return Err(AnonymizeError::InvalidToken);
        }
        if marker_present(input) {
            return Err(AnonymizeError::PlaceholderCollision);
        }
    }
    let mut captures = Vec::new();
    captures
        .try_reserve_exact(accepted.len())
        .map_err(|_| AnonymizeError::Allocation)?;
    let mut replacements = Vec::new();
    replacements
        .try_reserve_exact(accepted.len())
        .map_err(|_| AnonymizeError::Allocation)?;
    let mut removed = 0usize;
    for (index, accepted) in accepted.into_iter().enumerate() {
        let span = accepted.span();
        removed = removed
            .checked_add(span.end - span.start)
            .ok_or(AnonymizeError::CapacityOverflow)?;
        captures.push(Capture {
            value: &input[span.start..span.end],
            accepted,
        });
        replacements.push(PlannedReplacement {
            accepted,
            identity: ReplacementIdentity {
                kind: accepted.dominant().kind,
                number: index
                    .checked_add(1)
                    .ok_or(AnonymizeError::CapacityOverflow)?,
            },
            mode: ReplacementMode::Reversible,
        });
    }
    let added = captures
        .len()
        .checked_mul(32)
        .ok_or(AnonymizeError::CapacityOverflow)?;
    let output_bytes = input
        .len()
        .checked_sub(removed)
        .and_then(|n| n.checked_add(added))
        .ok_or(AnonymizeError::CapacityOverflow)?;
    if output_bytes > limits.plan.max_output_bytes
        || output_bytes.saturating_sub(input.len()) > limits.plan.max_output_growth
    {
        return Err(AnonymizeError::OutputLimit);
    }
    let mut text = String::new();
    text.try_reserve_exact(output_bytes)
        .map_err(|_| AnonymizeError::Allocation)?;
    let manifest = TransformationManifest {
        replacements,
        input_bytes: input.len(),
        output_bytes,
    };
    if captures.is_empty() {
        text.push_str(input);
        return Ok(AnonymizedOutput { text, manifest });
    }
    if sink.begin().is_err() {
        return cleanup(sink, AnonymizeError::CaptureFailed);
    }
    let staged = (|| {
        let tokens = sink
            .stage(&captures, limits)
            .map_err(|_| AnonymizeError::CaptureFailed)?;
        validate_tokens(&tokens, &captures, limits)?;
        let mut cursor = 0;
        for (capture, token) in captures.iter().zip(&tokens) {
            let span = capture.accepted.span();
            text.push_str(&input[cursor..span.start]);
            text.push_str(token);
            cursor = span.end;
        }
        text.push_str(&input[cursor..]);
        sink.commit().map_err(|_| AnonymizeError::CaptureFailed)?;
        Ok(AnonymizedOutput { text, manifest })
    })();
    match staged {
        Ok(output) => Ok(output),
        Err(error) => cleanup(sink, error),
    }
}
fn cleanup<S: TokenSink>(
    sink: &mut S,
    error: AnonymizeError,
) -> Result<AnonymizedOutput, AnonymizeError> {
    if sink.abort().is_err() {
        Err(AnonymizeError::CleanupFailed)
    } else {
        Err(error)
    }
}
fn validate_tokens(
    tokens: &[String],
    captures: &[Capture<'_>],
    limits: CaptureLimits,
) -> Result<(), AnonymizeError> {
    if tokens.len() != captures.len() {
        return Err(AnonymizeError::InvalidToken);
    }
    for token in tokens {
        let bytes = token.as_bytes();
        if bytes.len() != 32
            || bytes.len() > limits.max_token_bytes
            || !bytes.starts_with(b"<rsv_")
            || bytes[31] != b'>'
            || !bytes[5..31]
                .iter()
                .all(|b| b.is_ascii_lowercase() || (b'2'..=b'7').contains(b))
        {
            return Err(AnonymizeError::InvalidToken);
        }
    }
    let mut ordered = Vec::new();
    ordered
        .try_reserve_exact(tokens.len())
        .map_err(|_| AnonymizeError::Allocation)?;
    ordered.extend(tokens.iter().map(String::as_str));
    ordered.sort_unstable();
    if ordered.windows(2).any(|p| p[0] == p[1]) {
        return Err(AnonymizeError::InvalidToken);
    }
    // Compare only token-sized originals: total source comparison work is bounded
    // by token width, without duplicating values or retaining an independent map.
    for capture in captures {
        if capture.value.len() == 32 && ordered.binary_search(&capture.value).is_ok() {
            return Err(AnonymizeError::InvalidToken);
        }
    }
    Ok(())
}
fn marker_present(input: &str) -> bool {
    input
        .as_bytes()
        .windows(4)
        .any(|s| s.eq_ignore_ascii_case(b"rsv_"))
}
