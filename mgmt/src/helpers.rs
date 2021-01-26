macro_rules! debug { ($($tt:tt)*)=>{} }

macro_rules! SIENNA {
    ($x:expr) => {
        cosmwasm_std::coins($x, "SIENNA")
    }
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

