//macro_rules! debug { ($($tt:tt)*)=>{} }

macro_rules! SIENNA {
    ($x:expr) => { Uint128::from($x as u128 * ONE_SIENNA) }
}

/// Auth
macro_rules! require_admin {
    (|$env:ident, $state:ident| $body:block) => {
        if $env.message.sender != $state.admin {
            err_auth($state)
        } else $body
    }
}

/// Schedules
macro_rules! monthly {
    ($a:ident $b:literal $c:literal $d:literal $e:literal %) => {
        Stream {
            addr: recipient!($a),
            amount:  SIENNA!($b),
            vesting: Vesting::Periodic {
                interval: Interval::Monthly,
                start_at: $c * MONTH,
                duration: $d * MONTH,
                cliff:    $e
            }
        }
    }
}
macro_rules! daily {
    ($a:ident $b:literal $c:literal $d:literal $e:literal %) => {
        Stream {
            addr: recipient!($a),
            amount:  SIENNA!($b),
            vesting: Vesting::Periodic {
                interval: Interval::Daily,
                start_at: $c * MONTH,
                duration: $d * MONTH,
                cliff:    $e
            }
        }
    }
}
macro_rules! immediate {
    ($a:ident $b:literal) => {
        Stream {
            addr: recipient!($a),
            amount:  SIENNA!($b),
            vesting: Vesting::Immediate {}
        }
    }
}
