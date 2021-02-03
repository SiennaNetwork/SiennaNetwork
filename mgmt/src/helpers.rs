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
        Stream::Monthly {
            addr: recipient!($a),
            amount:  SIENNA!($b),
            release_months:  $c,
            cliff_months:    $d,
            cliff_percent:   $e
        }
    }
}
macro_rules! daily {
    ($a:ident $b:literal $c:literal $d:literal $e:literal %) => {
        Stream::Daily {
            addr: recipient!($a),
            amount:  SIENNA!($b),
            release_months:  $c,
            cliff_months:    $d,
            cliff_percent:   $e
        }
    }
}
macro_rules! immediate {
    ($a:ident $b:literal) => {
        Stream::Immediate {
            addr: recipient!($a),
            amount:  SIENNA!($b),
        }
    }
}
