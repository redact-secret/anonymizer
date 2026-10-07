use crate::arbitration::Accepted;

pub(crate) struct Plan<'a> {
    pub(crate) input: &'a str,
    pub(crate) capacity: usize,
}

pub(crate) fn plan(accepted: Accepted<'_>) -> Plan<'_> {
    Plan {
        input: accepted.input,
        capacity: accepted.input.len(),
    }
}
