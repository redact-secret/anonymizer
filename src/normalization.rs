use crate::AnonymizeError;

pub(crate) const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

pub(crate) struct Normalized<'a> {
    pub(crate) input: &'a str,
}

pub(crate) fn normalize(input: &str) -> Result<Normalized<'_>, AnonymizeError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(AnonymizeError::InputLimit);
    }
    Ok(Normalized { input })
}
