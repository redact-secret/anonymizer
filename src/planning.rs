/// Bounded replacement mode; reversible tokens are introduced separately.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplacementMode {
    /// Fixed category placeholder without reversible mappings.
    Irreversible,
    /// Opaque token placement through an externally owned capture transaction.
    #[cfg(feature = "reversible")]
    Reversible,
}

/// Text-free replacement identity, numbered globally in source-span order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplacementIdentity {
    pub(crate) kind: crate::FindingKind,
    pub(crate) number: usize,
}
impl ReplacementIdentity {
    /// Fixed category used for the placeholder.
    pub fn kind(&self) -> crate::FindingKind {
        self.kind
    }
    /// One-based global replacement ordinal, unrelated to original values.
    pub fn number(&self) -> usize {
        self.number
    }
    pub(crate) fn label(&self) -> &'static str {
        kind_label(self.kind)
    }
    pub(crate) fn len(&self) -> usize {
        self.label().len() + digits(self.number) + 3
    }
}

/// Safe immutable replacement metadata; never contains original text or tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlannedReplacement {
    pub(crate) accepted: crate::AcceptedSpan,
    pub(crate) identity: ReplacementIdentity,
    pub(crate) mode: ReplacementMode,
}
impl PlannedReplacement {
    /// Validated accepted range, contributor count, and arbitration reasoning.
    pub fn accepted(&self) -> crate::AcceptedSpan {
        self.accepted
    }
    /// Text-free replacement identity.
    pub fn identity(&self) -> ReplacementIdentity {
        self.identity
    }
    /// Replacement mode without vault authority metadata.
    pub fn mode(&self) -> ReplacementMode {
        self.mode
    }
}

/// Standalone safe transformation metadata; no input, matched values, or mappings.
#[derive(Debug, Eq, PartialEq)]
pub struct TransformationManifest {
    pub(crate) replacements: Vec<PlannedReplacement>,
    pub(crate) input_bytes: usize,
    pub(crate) output_bytes: usize,
}
impl TransformationManifest {
    /// Ordered, non-overlapping replacement metadata.
    pub fn replacements(&self) -> &[PlannedReplacement] {
        &self.replacements
    }
    /// Original input length; location/length metadata still requires host review.
    pub fn input_bytes(&self) -> usize {
        self.input_bytes
    }
    /// Exact planned output capacity.
    pub fn output_bytes(&self) -> usize {
        self.output_bytes
    }
}

/// Immutable plan bound to the exact borrowed original input.
/// Debug only formats safe metadata. No public input accessor or rebind exists.
pub struct ReplacementPlan<'a> {
    pub(crate) input: &'a str,
    pub(crate) manifest: TransformationManifest,
}
impl ReplacementPlan<'_> {
    /// Safe metadata without plaintext, token identities, or authorization state.
    pub fn manifest(&self) -> &TransformationManifest {
        &self.manifest
    }
    /// Exact capacity checked before any final output allocation.
    pub fn output_bytes(&self) -> usize {
        self.manifest.output_bytes
    }
}
impl std::fmt::Debug for ReplacementPlan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reading only length here also makes source binding explicit before
        // the constructor consumes it in the following implementation stage.
        f.debug_struct("ReplacementPlan")
            .field("input_bytes", &self.input.len())
            .field("manifest", &self.manifest)
            .finish()
    }
}

/// Limits for whole-input irreversible planning, checked before output exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanLimits {
    /// Input and submitted-finding bounds.
    pub input: crate::Limits,
    /// Maximum accepted replacement count.
    pub max_replacements: usize,
    /// Maximum bytes in one generated placeholder.
    pub max_placeholder_bytes: usize,
    /// Maximum absolute final output size.
    pub max_output_bytes: usize,
    /// Maximum positive output growth over original input length.
    pub max_output_growth: usize,
}
impl Default for PlanLimits {
    fn default() -> Self {
        Self {
            input: crate::Limits::default(),
            max_replacements: 100_000,
            max_placeholder_bytes: 64,
            max_output_bytes: 32 * 1024 * 1024,
            max_output_growth: 16 * 1024 * 1024,
        }
    }
}

/// Plan irreversible replacements without constructing output or owning text.
///
/// Uses one normalization/arbitration pass. Fixed `<KIND_N>` placeholders use
/// one-based global span-order numbering. Any reserved `<KIND_` prefix already
/// in input rejects planning when replacements exist, including source-equal
/// placeholders, so output cannot confuse source literals with generated labels.
pub fn plan_irreversible<'a>(
    input: &'a str,
    findings: &[crate::SourceFinding],
    limits: PlanLimits,
) -> Result<ReplacementPlan<'a>, crate::AnonymizeError> {
    use crate::AnonymizeError;
    let accepted = crate::arbitrate_findings(input, findings, limits.input)?;
    if accepted.len() > limits.max_replacements {
        return Err(AnonymizeError::ReplacementLimit);
    }
    if !accepted.is_empty() && reserved_literal(input) {
        return Err(AnonymizeError::PlaceholderCollision);
    }
    let mut replacements = Vec::new();
    replacements
        .try_reserve_exact(accepted.len())
        .map_err(|_| AnonymizeError::Allocation)?;
    let mut removed = 0usize;
    let mut added = 0usize;
    for (index, accepted) in accepted.into_iter().enumerate() {
        let identity = ReplacementIdentity {
            kind: accepted.dominant().kind,
            number: index
                .checked_add(1)
                .ok_or(AnonymizeError::CapacityOverflow)?,
        };
        if identity.len() > limits.max_placeholder_bytes {
            return Err(AnonymizeError::PlaceholderLimit);
        }
        let span = accepted.span();
        removed = removed
            .checked_add(span.end - span.start)
            .ok_or(AnonymizeError::CapacityOverflow)?;
        added = added
            .checked_add(identity.len())
            .ok_or(AnonymizeError::CapacityOverflow)?;
        replacements.push(PlannedReplacement {
            accepted,
            identity,
            mode: ReplacementMode::Irreversible,
        });
    }
    let output_bytes = input
        .len()
        .checked_sub(removed)
        .and_then(|n| n.checked_add(added))
        .ok_or(AnonymizeError::CapacityOverflow)?;
    if output_bytes > limits.max_output_bytes
        || output_bytes.saturating_sub(input.len()) > limits.max_output_growth
    {
        return Err(AnonymizeError::OutputLimit);
    }
    Ok(ReplacementPlan {
        input,
        manifest: TransformationManifest {
            replacements,
            input_bytes: input.len(),
            output_bytes,
        },
    })
}

fn kind_label(kind: crate::FindingKind) -> &'static str {
    use crate::FindingKind;
    match kind {
        FindingKind::Credential => "CREDENTIAL",
        FindingKind::Person => "PERSON",
        FindingKind::Email => "EMAIL",
        FindingKind::Phone => "PHONE",
        FindingKind::NetworkAddress => "NETWORK_ADDRESS",
        FindingKind::Address => "ADDRESS",
        FindingKind::Other => "OTHER",
    }
}
fn digits(mut number: usize) -> usize {
    let mut count = 1;
    while number >= 10 {
        count += 1;
        number /= 10;
    }
    count
}
fn reserved_literal(input: &str) -> bool {
    let bytes = input.as_bytes();
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == b'<'
            && [
                "CREDENTIAL_",
                "PERSON_",
                "EMAIL_",
                "PHONE_",
                "NETWORK_ADDRESS_",
                "ADDRESS_",
                "OTHER_",
            ]
            .iter()
            .any(|prefix| bytes[i + 1..].starts_with(prefix.as_bytes()))
        {
            return true;
        }
    }
    false
}
