#[macro_export] macro_rules! Error {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr { msg: $msg.to_string(), backtrace: None })
    }
}
#[macro_export] macro_rules! valid {
    ($schedule:expr) => {
        assert_eq!($schedule.validate(), Ok(()));
    };
    ($schedule:expr, $value:expr) => {
        assert_eq!($schedule.validate(), Ok($value));
    }
}
#[macro_export] macro_rules! invalid {
    ($schedule:expr, $error:expr) => {
        assert_eq!($schedule.validate(), Error!($error));
    }
}
#[macro_export] macro_rules! claim {
    ($schedule:expr, $addr: expr, $time: expr $(, $res:expr)*) => {
        let actual = $schedule.claimable(&$addr, $time);
        let expected = Ok(vec![$($res),*]);
        if actual != expected {
            println!("---ACTUAL CLAIMS:---");
            match actual {
                Ok(actual) => for portion in actual.iter() {
                    println!("{}", &portion);
                },
                Err(e) => println!("Error: {}", &e)
            }
            println!("---EXPECTED CLAIMS:---");
            for claim in expected.iter() {
                println!("");
                for portion in claim.iter() {
                    println!("{}", &portion);
                }
            }
            panic!("test failed: claim!")
        }
    }
}

