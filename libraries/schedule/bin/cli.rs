//use serde::Deserialize;
use sienna_schedule::{*, vesting::*, validate::*};

use std::{path::Path, fs::read_to_string};
use serde_json_wasm::from_str;

use clap::{App, Arg};

fn main () -> Result<(), std::io::Error> {
    let mut app = App::new("sienna_schedule")
        .version("1.0")
        .author("Adam A. <adam@hack.bg>")
        .about("Converts a schedule from JSON to Markdown, materializing all portions")
        .arg(Arg::new("PATH")
            .default_value("../../settings/schedule.json"));

    let matches = &app.clone().get_matches();
    match matches.value_of("PATH") {
        Some(path) => {
            println!("\n# Schedule");
            println!("\n(Generated from schedule.json)\n");
            let schedule = get_schedule(path).unwrap();
            println!("Internal representation:\n```\n{:#?}\n```", &schedule);
            schedule.validate().unwrap();
            for pool in schedule.pools.iter() {
                println!("\n## Pool: *{}*", &pool.name);
                for account in pool.accounts.iter() {
                    println!("\n### Account: *{}*\n", &account.name);
                    println!("* Amount: **{} attoSIENNA**", &account.amount);
                    println!("* Cliff: **{} attoSIENNA**", &account.cliff);
                    println!("* Portion size: **{} attoSIENNA**", &account.portion_size());
                    println!("* Portion count: **{}**\n", &account.portion_count());
                    let mut portion = if account.cliff > cosmwasm_std::Uint128::zero() { -1 } else { 0 };
                    let mut balance = 0u128;
                    println!("|portion #|day|unlocked amount (attoSIENNA)|");
                    println!("|:-:|:-:|--:|");
                    for t in account.start_at..account.end()+1 {
                        let new_balance = account.unlocked(t, &account.address);
                        if new_balance != balance {
                            portion += 1;
                            balance = new_balance;
                            println!("|{:>7}|{:>7}|{:>26}|",
                                if portion == 0 { "cliff".to_string() } else { portion.to_string() },
                                t / 86400,
                                balance
                            );
                        }
                    }
                }
            }
        },
        None => {
            app.print_long_help()?;
        }
    };

    Ok(())

}

fn get_schedule (path: &str) -> Result<Schedule<HumanAddr>, serde_json_wasm::de::Error> {
    from_str(&read_to_string(Path::new(path)).unwrap())
}
