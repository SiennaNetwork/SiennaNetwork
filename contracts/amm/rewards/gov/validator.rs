use fadroma::{StdError, StdResult};

pub fn validate_text_length(text: &str, name: &str, min: usize, max: usize) -> StdResult<()> {
    if text.len() < min {
        Err(StdError::generic_err(format!(
            "{} is too short. (min: {}, max: {})",
            name, min, max
        )))
    } else if text.len() > max {
        Err(StdError::generic_err(format!(
            "{} is too long. (min: {}, max: {})",
            name, min, max
        )))
    } else {
        Ok(())
    }
}
