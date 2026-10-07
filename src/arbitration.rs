use crate::normalization::Normalized;

pub(crate) struct Accepted<'a> {
    pub(crate) input: &'a str,
}

pub(crate) fn arbitrate(normalized: Normalized<'_>) -> Accepted<'_> {
    Accepted {
        input: normalized.input,
    }
}
