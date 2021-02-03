macro_rules! debug { ($($tt:tt)*)=>{} }

macro_rules! SIENNA {
    ($x:expr) => { Uint128::from($x as u128 * ONE_SIENNA) }
}

macro_rules! canon {
    ($deps:ident, $($x:tt)*) => {
        $deps.api.canonical_address($($x)*).unwrap();
    }
}

//macro_rules! human {
    //($deps:ident, $($x:tt)*) => {
        //$deps.api.human_address($($x)*).unwrap();
    //}
//}

/// Schedules
macro_rules! monthly {
    ($a:ident $b:literal $c:literal $d:literal $e:literal %) => {
        Stream {
            addr: recipient!($a),
            amount:  SIENNA!($b),
            vesting: Vesting::Monthly {
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
            vesting: Vesting::Daily {
                start_at: $d * MONTH,
                duration: $c * MONTH,
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
