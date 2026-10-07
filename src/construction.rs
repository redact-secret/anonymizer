use crate::{AnonymizeError, ReplacementIdentity, ReplacementPlan, TransformationManifest};

/// Transformed text with safe metadata. Debug never formats the text.
/// Undetected source bytes are retained; this is not universally safe output.
pub struct AnonymizedOutput {
    pub(crate) text: String,
    pub(crate) manifest: TransformationManifest,
}
impl AnonymizedOutput {
    /// Output for the caller's explicitly trusted destination.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Text-free transformation metadata.
    pub fn manifest(&self) -> &TransformationManifest {
        &self.manifest
    }
    /// Move output and metadata to the caller without cloning either.
    pub fn into_parts(self) -> (String, TransformationManifest) {
        (self.text, self.manifest)
    }
}
impl std::fmt::Debug for AnonymizedOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnonymizedOutput")
            .field("manifest", &self.manifest)
            .finish()
    }
}

/// Consume a validated plan using only its bound original input.
///
/// Reserves exact capacity before writing and makes one ordered output pass.
/// No detector scan, input substitution, substring allocation, or vault capture
/// occurs. Allocation failure returns no partial output.
pub fn construct(plan: ReplacementPlan<'_>) -> Result<AnonymizedOutput, AnonymizeError> {
    let mut output = String::new();
    output
        .try_reserve_exact(plan.output_bytes())
        .map_err(|_| AnonymizeError::Allocation)?;
    let mut cursor = 0;
    for replacement in plan.manifest().replacements() {
        let span = replacement.accepted().span();
        output.push_str(&plan.input[cursor..span.start]);
        append_placeholder(&mut output, replacement.identity());
        cursor = span.end;
    }
    output.push_str(&plan.input[cursor..]);
    Ok(AnonymizedOutput {
        text: output,
        manifest: plan.manifest,
    })
}

fn append_placeholder(output: &mut String, identity: ReplacementIdentity) {
    output.push('<');
    output.push_str(identity.label());
    output.push('_');
    // One byte per binary bit is comfortably sufficient for decimal digits on
    // every supported pointer width, with no heap allocation or formatting error.
    let mut digits = [0u8; usize::BITS as usize];
    let mut number = identity.number();
    let mut used = 0;
    loop {
        digits[used] = b'0' + (number % 10) as u8;
        used += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for digit in digits[..used].iter().rev() {
        output.push(char::from(*digit));
    }
    output.push('>');
}
