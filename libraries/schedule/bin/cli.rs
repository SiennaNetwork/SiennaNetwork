use serde::Deserialize;
use sienna_schedule::{*, vesting::*, validate::*};

use std::{path::Path, fs::read_to_string};
use serde_json_wasm::from_str;

use clap::{App, Arg};

fn main () -> Result<(), std::io::Error> {
    let mut app = App::new("sienna_schedule")
        .version("1.0")
        .author("Adam A. <adam@hack.bg>")
        .about("Manages vesting schedule definitions")
        .arg(Arg::new("PATH")
            .default_value("../../settings/schedule.json"));

    let matches = &app.clone().get_matches();
    match matches.value_of("PATH") {
        Some(path) => {
            println!("reading schedule from {}", &path);
            let schedule = get_schedule(path).unwrap();
            println!("{:#?}", &schedule);
            schedule.validate().unwrap();
            for pool in schedule.pools.iter() {
                for account in pool.accounts.iter() {
                    println!("\nAccount:       {}", &account.name);
                    println!("  Amount:        {}", &account.amount);
                    println!("  Cliff:         {}", &account.cliff);
                    println!("  Portion size:  {}", &account.portion_size());
                    println!("  Portion count: {}", &account.portion_count());
                    let mut balance = 0u128;
                    for t in account.start_at..account.end()+1 {
                        let new_balance = account.unlocked(t, &account.address);
                        if new_balance != balance {
                            balance = new_balance;
                            println!("{:>12}│{:>26}", t, balance);
                        }
                    }
                }
            }
        },
        None => {
            &app.print_long_help();
        }
    };

    Ok(())

}

fn get_schedule (path: &str) -> Result<Schedule, serde_json_wasm::de::Error> {
    from_str(&read_to_string(Path::new(path)).unwrap())
}
