/// Configurable recipients.
use crate::types::*;
pub type ConfiguredRecipients = Vec<(Address, Amount)>;

/// Return true if recipient is configurable
use crate::schedule::{Schedule, ReleaseMode};
pub fn is_configurable (s: Schedule) -> bool {
    match s.release_mode {
        ReleaseMode::Configurable => true,
        _ => false
    }
}
