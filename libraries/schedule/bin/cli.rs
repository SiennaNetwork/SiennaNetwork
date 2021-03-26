use std::path::Path;
use clap::{App, Arg};
use sienna_schedule::*;

macro_rules! subcommand {
    ($($matches:ident $subcommand:literal $body:block)+) => {
        $(if let Some(ref $matches) = $matches.subcommand_matches($subcommand) { $body; return } else )*
    }
}

fn main () {
    let app = App::new("sienna_schedule")
        .version("1.0")
        .author("Adam A. <adam@hack.bg>")
        .about("Manages vesting schedule definitions")

        .subcommand(App::new("validate")
            .about("Validate a configuration")
            .arg(Arg::new("PATH")
                .default_value("schedule.ods")))

        .subcommand(App::new("materialize")
            .about("Generate a portion list from a configuration")
            .arg(Arg::new("PATH")
                .default_value("schedule.ods")));

    let matches = &app.get_matches();

    subcommand!(
        matches "validate" {
            let input = matches.value_of("PATH").unwrap();
            let path = std::path::Path::new(&input);
            println!("VALIDATE {:#?}", &path);
        }
        matches "materialize" {
            let input = matches.value_of("PATH").unwrap();
            let path = std::path::Path::new(&input);
            println!("RENDER {:#?}", &path);
        }
        default {
            app.print_long_help();
        }
    );

}
