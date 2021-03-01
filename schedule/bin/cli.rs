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

            .arg(Arg::new("PATH")
                .default_value("./schedule.ods")))

        .subcommand(App::new("validate")
            .about("Validate a configuration")

            .arg(Arg::new("PATH")
                .default_value("schedule.ods")))

        .subcommand(App::new("render")
            .about("Generate a portion list from a configuration")

            .arg(Arg::new("PATH")
                .default_value("schedule.ods")))

        .get_matches();

    subcommand!(
        matches "template" {
            let output = matches.value_of("PATH").unwrap();
            let path = std::path::Path::new(&output);
            if path.exists() && !matches.is_present("force") {
                panic!("{} already exists, refusing to overwrite", &output);
            }
            spreadsheet_ods::write_ods(&generate_template(), path);
        }

        matches "validate" {
            let input = matches.value_of("PATH").unwrap();
            let path = std::path::Path::new(&input);
            println!("VALIDATE {:#?}", &path);
        }

        matches "render" {
            let input = matches.value_of("PATH").unwrap();
            let path = std::path::Path::new(&input);
            println!("RENDER {:#?}", &path);
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
    let mut schedule = Schedule!(
        1000
        (Founders  500 (Founder1  250)
                       (Founder2  250))
        (Investors 500 (Investor1 250)
                       (Investor2 250))
    );

    let (st_header, st_schedule, st_pool, st_account) = add_cell_styles(&mut book);

    macro_rules! zone {
        ( $style:expr
        ; $row:expr
        ; $col:expr
        ; $rows:expr
        ; $cols:expr
        $(; $(($value_row:expr, $value_col:expr, $value:expr)),*)? ) => {
            for row in $row..$row+$rows {
                for col in $col..$col+$cols {
                    println!("{} {} <{}> = {}", row, col, stringify!($style), "");
                    sheet.set_styled_value(row, col, " ", &$style);
                }
            }
            $(add_values!($(($value_row, $value_col, $value, $style),)*))?
        };
    }

    macro_rules! add_values {
        ($(( $row:expr, $col:expr, $value:expr, $style:expr ),)*) => {
            $(
                println!("{} {} <{}> = {}", $row, $col, stringify!($style), $value);
                sheet.set_styled_value($row, $col, $value, &$style);
            )*
        };
    };

    zone!(st_header // style
         ;0         // starting row
         ;0         // starting column
         ;2         // height
         ;22        // width
         ;          // items
        (0,  0, "Schedule"),
        (1,  0, "Total"),

        (0,  1, "Pool"),
        (1,  1, "Name"),
        (1,  2, "Subtotal"),

        (0,  3, "Account"),
        (1,  3, "Name"),
        (1,  4, "Amount"),
        (1,  5, "% of total"),
        (1,  7, "Days"),
        (1,  8, "(seconds)"),
        (1, 12, "SIENNA"),

        (0, 13, "Allocation"),
        (1, 13, "Address"),
        (1, 14, "Amount"));

    zone!(st_schedule ; 2 ; 0 ; 10 ; 22);
    zone!(st_pool     ; 3 ; 1 ;  9 ; 21);
    zone!(st_account  ; 4 ; 3 ;  3 ; 21);
    zone!(st_account  ; 8 ; 3 ;  3 ; 21);

    book.push_sheet(sheet);
    book
}

use spreadsheet_ods::CellStyleRef;
fn add_cell_styles (book: &mut spreadsheet_ods::WorkBook)
    -> (CellStyleRef, CellStyleRef, CellStyleRef, CellStyleRef) {
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
