use crate::{planning::Plan, AnonymizeError};

pub(crate) fn construct(plan: Plan<'_>) -> Result<String, AnonymizeError> {
    let mut output = String::new();
    output
        .try_reserve_exact(plan.capacity)
        .map_err(|_| AnonymizeError::Allocation)?;
    output.push_str(plan.input);
    Ok(output)
}
