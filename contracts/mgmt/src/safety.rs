/// Auth
#[macro_export] macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if Some($env.message.sender) != $state.admin {
            err_auth($state)
        } else {
            $body
        }
    }
}

/// Errors
lazy_static! {
    /// Error message: assumptions have been violated.
    pub static ref CORRUPTED:   &'static str = "broken";
    /// Error message: unauthorized or nothing to claim right now.
    pub static ref NOTHING:     &'static str = "nothing for you";
    /// Error message: can't launch more than once.
    pub static ref UNDERWAY:    &'static str = "already underway";
    /// Error message: can't do this before launching.
    pub static ref PRELAUNCH:   &'static str = "not launched yet";
    /// Error message: schedule hasn't been set yet.
    pub static ref NO_SCHEDULE: &'static str = "set configuration first";
    /// Error message: can't find channel/pool by name.
    pub static ref NOT_FOUND:   &'static str = "target not found";
}
