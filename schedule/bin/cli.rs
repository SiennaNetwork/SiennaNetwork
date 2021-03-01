use std::path::Path;
use clap::{App, Arg};
use sienna_schedule::*;

macro_rules! subcommand {
    ($($matches:ident $subcommand:literal $body:block)+) => {
        $(if let Some(ref $matches) = $matches.subcommand_matches($subcommand) {
            $body
        })*
    }
}

fn main () {
    let matches = App::new("sienna_schedule")
        .version("1.0")
        .author("Adam A. <adam@hack.bg>")
        .about("Manages vesting schedule definitions")

        .subcommand(App::new("template")
            .about("Generate a template spreadsheet")

            .arg(Arg::new("force")
                .short('f').long("force").takes_value(false)
                .about("Overwrite the destination file if it exists"))

            .arg(Arg::new("OUTPUT")
                .default_value("schedule.ods")))

        .subcommand(App::new("validate")
            .about("Validate a configuration")

            .arg(Arg::new("INPUT")
                .default_value("schedule.ods")))

        .subcommand(App::new("render")
            .about("Generate a portion list from a configuration")

            .arg(Arg::new("INPUT")
                .default_value("schedule.ods")))

        .get_matches();

    subcommand!(
        matches "template" {
            let output = matches.value_of("OUTPUT").unwrap();
            let path = std::path::Path::new(&output);
            if path.exists() && !matches.is_present("force") {
                panic!("{} already exists, refusing to overwrite", &output);
            }
            spreadsheet_ods::write_ods(&generate_template(), path);
        }

        matches "validate" {
            println!("VALIDATE {:#?}", &matches);
        }

        matches "render" {
            println!("RENDER {:#?}", &matches);
        }
    );
}

fn generate_template () -> spreadsheet_ods::WorkBook {
    use spreadsheet_ods::{
        WorkBook, Sheet, Value,
        CellStyle, defaultstyles::DefaultFormat, style::units::TextAlign,
        write_ods
    };
    let mut book     = WorkBook::new();
    let mut sheet    = Sheet::new();
    let mut schedule = Schedule!(1000
        (Founders  500 (Founder1  250) (Founder2  250))
        (Investors 500 (Investor1 250) (Investor2 250))
    );

    let (st_header, st_schedule, st_pool, st_account) = add_cell_styles(&mut book);

    macro_rules! zone {
        ($row:expr, $col:expr, $rows:expr, $cols:expr, $style:expr) => {
            for row in $row..$row+$rows {
                for col in $col..$col+$cols {
                    println!("{} {} <{}> = {}", row, col, stringify!($style), "");
                    sheet.set_styled_value(row, col, " ", &$style);
                }
            }
        }
    }

    zone!(0, 0,  1, 22, header);
    zone!(1, 0, 10, 22, schedule);
    zone!(2, 1,  9, 21, pool);
    zone!(3, 3,  3, 21, account);
    zone!(7, 3,  3, 21, account);

    macro_rules! add_values {
        ($(( $row:expr, $col:expr, $value:expr, $style:expr ),)*) => {
            $(
                println!("{} {} <{}> = {}", $row, $col, stringify!($style), $value);
                sheet.set_styled_value($row, $col, $value, &$style);
            )*
        };
    };

    add_values!(
        (0, 0, "Total",  header),
        (1, 0, 10000000, schedule),

        (0, 1, "Pool",     header),
        (2, 1, "Founders", pool),

        (0, 2, "Subtotal", header),
        (1, 2, 10000000,   schedule),
        (2, 2, 10000000,   pool),

        (0, 3, "Account",  header),
        (3, 3, "Founder3", account),

        (0,  4, "Amount", header),
        (3,  4,   731000, account),

        (0,  5, "% of Total", header),
        (2,  5, "100%",       pool),
        (3,  5, "100%",       account),

        (0,  6, "Start at\nNth day", header),
        (3,  6, 180,                 account),

        (0,  7, "`start_at`\n(seconds)", header),
        (3,  7, 15552000,                account),

        (0,  8, "Interval\n(days)", header),
        (3,  8, 1,                  account),

        (0,  9, "`interval`\n(seconds)", header),
        (3,  9, 86400,                   account),

        (0, 10, "Duration\n(days)", header),
        (3, 10, 481,                account),

        (0, 11, "`duration`\n(seconds)", header),
        (3, 11, 41558400,                account),

        (0, 12, "Cliff\n%", header),
        (3, 12, 10.0,       account),

        (0, 13, "`cliff`\n(SIENNA)", header),
        (3, 13, 73100,               account),

        (0, 14, "Cliff\nallocation", header),
        (3, 14, 4000,                account),
        (4, 14, 166,                 account),

        (0, 15, "Cliff\nrecipient", header),
        (3, 15, "secret1MyAddressxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", account),
        (4, 15, "secret1MyOtherAddressxxxxxxxxxxxxxxxxxxxxxxxx", account),

        (0, 16, "Regular\nportions", header),
        (3, 16, 480,                 account),

        (0, 17, "Portion\nsize", header),
        (3, 17, 137, account),

        (0, 18, "Portion\nallocation", header),
        (3, 18, 1000, account),
        (4, 18, 370, account),

        (0, 19, "Portion\nrecipient", header),
        (3, 19, "secret1MyAddressxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", account),
        (4, 19, "secret1MyOtherAddressxxxxxxxxxxxxxxxxxxxxxxxx", account),

        (0, 20, "Remainder", header),
        (3, 20, 4166, account),

        (0, 21, "Remainder\nallocation", header),
        (3, 21, 4000, account),
        (4, 21, 166, account),

        (0, 22, "Remainder\nrecipient", header),
        (3, 22, "secret1MyAddressxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", account),
        (4, 22, "secret1MyOtherAddressxxxxxxxxxxxxxxxxxxxxxxxx", account),
    );

    book.push_sheet(sheet);
    book
}

fn add_cell_styles (&mut book: spreadsheet_ods::WorkBook) {
    use color::Rgb;
    use spreadsheet_ods::{
        CellStyle,
        style::units::TextAlign,
        defaultstyles::DefaultFormat
    };

    let mut header = CellStyle::new("header", &DefaultFormat::default());
    header.set_font_bold();
    header.set_color(Rgb::new(255, 255, 255));
    header.set_background_color(Rgb::new(97, 23, 41));
    header.set_text_align(TextAlign::Center);
    let header = book.add_cellstyle(header);

    let mut schedule = CellStyle::new("schedule", &DefaultFormat::default());
    schedule.set_font_bold();
    schedule.set_color(Rgb::new(0, 0, 0));
    schedule.set_background_color(Rgb::new(160, 160, 160));
    schedule.set_text_align(TextAlign::Center);
    let schedule = book.add_cellstyle(schedule);

    let mut pool = CellStyle::new("pool", &DefaultFormat::default());
    pool.set_font_bold();
    pool.set_color(Rgb::new(0, 0, 0));
    pool.set_background_color(Rgb::new(192, 192, 192));
    pool.set_text_align(TextAlign::Center);
    let pool = book.add_cellstyle(pool);

    let mut account = CellStyle::new("account", &DefaultFormat::default());
    account.set_color(Rgb::new(0, 0, 0));
    account.set_background_color(Rgb::new(224, 224, 224));
    account.set_text_align(TextAlign::Center);
    let account = book.add_cellstyle(account);

    (header, schedule, pool, account)
}
