use std::env::current_dir;
use std::fs::create_dir_all;

use svg::Document;
use svg::node::Text as TextNode;
use svg::node::element::{Rectangle, Group, Text, Line, Polyline};
use svg::node::element::path::Data;

use sienna_mgmt::schedule::SCHEDULE;
use sienna_mgmt::types::{Stream, ONE_SIENNA};

macro_rules! svg {
    ($El:ident $($attr:ident = $value:expr)+) => {
        $El::new()
        $(.set(str::replace(stringify!($attr), "_", "-"), $value))*
    };
    ($text:expr) => {
        TextNode::new($text)
    }
}

fn main() {

    let total = SCHEDULE.total.u128() / ONE_SIENNA;
    let (t_min, t_max) = (0, 100000000); // TODO: get those from the schedule

    let width = 2000;
    let height = 3000;
    let margin = 200;
    let t_scale = width / (t_max - t_min);
    let viewbox = (-margin, -margin, width+2*margin, height+2*margin);

    // chart
    let mut doc = svg!(Document
        width=width height=height
        viewBox=viewbox overflow="auto"
        font_family="monospace" font_size="20");

    // chart background
    doc = doc.add(svg!(Rectangle
        width="120%" height="120%" x="-20%" y="-20%"
        fill="white"));

    let mut y = 0f64;

    // data
    for stream in SCHEDULE.predefined.iter() {

        let mut g = svg!(Group class="stream"
            transform=format!("translate(0,{})", y));

        let mut bg = svg!(Rectangle class="stream-bg"
            x=0 y=0 width=width fill="transparent");

        let mut addr: String;
        let mut amount: u128;
        match stream {

            Stream::Immediate{
                addr:_addr, amount:_amount
            } => {
                g = g.set("class", "stream immediate");
                bg = bg.set("fill", "rgba(64,255,64,0.2");
                addr = _addr.to_string();
                amount = _amount.u128();
            },

            Stream::Monthly{
                addr:_addr, amount:_amount,
                release_months, cliff_months, cliff_percent
            } => {
                g = g.set("class", "stream monthly");
                addr = _addr.to_string();
                amount = _amount.u128();
            },

            Stream::Daily{
                addr:_addr, amount:_amount,
                release_months, cliff_months, cliff_percent
            } => {
                g = g.set("class", "stream daily");
                addr = _addr.to_string();
                amount = _amount.u128();
            }

        }

        amount = amount / ONE_SIENNA;

        let percent = amount as f64 / total as f64;
        g = g.set("data-percent", percent.to_string());

        // random repeated-log scaling
        let h = ((percent * 10000.0).ln().ln()*100.0).ln() * 50.0;
        g = g.set("data-h", h);

        println!("{} {}/{} {} {}", &addr, &amount, &total, &percent, &h);

        g = g.add(svg!(Line class="stream-border"
            x1=0 y1=0 x2=width y2=0
            stroke="#000" stroke_width=0.5));

        g = g.add(svg!(Text class="stream-id"
            x=width+10 y=h/2.0 text_anchor="start")
            .add(svg!(&addr)));

        let mut points = String::new();
        //for _ in vec![] {
            //points.push_str("");
        //}
        g = g.add(svg!(Polyline class="stream-flow"
            fill="rgba(64,255,64,0.2)"
            stroke="rgba(0,128,0,0.5)"
            stroke_width=0.5
            points=points));

        g = g.add(svg!(Text class="stream-amount"
            x=-10 y=h/2.0 text_anchor="end")
            .add(svg!(amount.to_string())));

        g = g.set("id", addr.to_string());

        bg = bg.set("height", h);
        g = g.add(bg);

        doc = doc.add(g);

        y += h;

    }

    // grid
    let mut grid = svg!(Group
        id="grid");

    // grid frame
    grid = grid.add(svg!(Rectangle
        x=0 y=0 width=width height=height
        stroke="red" stroke_width=2 fill="none"));

    // grid labels
    grid = grid.add(
        svg!(Text x=0 y=-15 text_anchor="end")
            .add(svg!(format!("T={}", t_min))));
    grid = grid.add(
        svg!(Text x=width y=-15 text_anchor="start")
            .add(svg!(format!("T={}", t_max))));

    // grid lines
    let day_width = width as f64 / ((t_max-t_min)/(24*60*60)) as f64;
    let week_width = 15.0 * day_width;
    println!("{} {} {}", width, day_width, week_width);
    let n_weeks = 47;
    for i in 0..n_weeks {
        let x = i as f64 * week_width;
        grid = grid.add(
            svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.2)"));
    }
    for i in 0..n_weeks/6 {
        let x = i as f64 * 6.0 * week_width;
        grid = grid.add(
            svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.4)"));
    }

    // add grid to document
    doc = doc.add(grid);

    svg::save("docs/schedule.svg", &doc).unwrap();
}
