use crate::types::*;

/// This is needed to import the schedule from JSON during compilation.
const SCHEDULE_SRC: &str = include_str!("schedule.yml");
lazy_static! {
    static ref SCHEDULES: Vec<Schedule> =
        serde_yaml::from_str(&SCHEDULE_SRC).unwrap();
}

const DAY:   Time = 24*60*60;
const MONTH: Time = 30*DAY;

/// Distil the value in question from the schedule.

pub fn at (
    recipient: &cosmwasm_std::HumanAddr,
    launched: Time,
    now:      Time,
) -> Amount {
    for s in SCHEDULES.iter() {
        if s.addr != *recipient {
            continue
        }
        let t_start = launched + match s.cliff_months {
            Some(t) => t as u64 * MONTH,
            None    => 0
        };
        if now < t_start {
            return 0
        };
        match s.release_mode {
            ReleaseMode::Immediate => s.amount,
            ReleaseMode::Configurable => todo!(),
            _ => {
                let t_end = t_start + match s.release_months {
                    Some(t) => t as u64 * MONTH,
                    _ => panic!("missing `release_months` on daily/monthly vesting")
                };
                if now > t_end {
                     0
                } else {
                    let cliff_amount = match s.cliff_percent {
                        None => 0,
                        Some(c) => {
                            let c = c as u64;
                            if c * s.amount % 100 > 0 {
                                println!("WARNING: division with remainder for {} cliff amount", s.addr)
                            }
                            c * s.amount / 100
                        }
                    };
                    let (t_elapsed, t_total) = match s.release_mode {
                        ReleaseMode::Daily => (
                            (  now - t_start) / DAY,
                            (t_end - t_start) / DAY
                        ),
                        ReleaseMode::Monthly => (
                            (  now - t_start) / MONTH,
                            (t_end - t_start) / MONTH
                        ),
                        _ => unreachable!()
                    };
                    if s.amount % t_total > 0 {
                        println!("WARNING: division with remainder for {} vesting amount", s.addr)
                    }
                    cliff_amount + s.amount * t_elapsed / t_total
                }
            }
        };
    }
    0
}

/*
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "release_mode")]
enum Schedule {
    Configurable {
        amount: Amount
    },
    Immediate {
        amount: Amount,
        addr:   cosmwasm_std::HumanAddr
    },
    Daily {
        amount: Amount,
        addr: cosmwasm_std::HumanAddr,
        release_months: u16,
        cliff_months:   u16,
        cliff_percent:  u16
    },
    Monthly {
        amount: Amount,
        addr: cosmwasm_std::HumanAddr,
        release_months: u16,
        cliff_months:   u16,
        cliff_percent:  u16
    },
}
*/
