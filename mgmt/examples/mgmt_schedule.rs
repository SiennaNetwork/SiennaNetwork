use std::env::current_dir;
use std::fs::create_dir_all;

use svg::Document;
use svg::node::Text as TextNode;
use svg::node::element::{Rectangle, Group, Text, Line};
use svg::node::element::path::Data;

use sienna_mgmt::schedule::SCHEDULE;
use sienna_mgmt::types::Stream;

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

    let total = SCHEDULE.total.u128();
    let (t_min, t_max) = (0, 1000000000); // TODO: get those from the schedule

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

    // data
    for stream in SCHEDULE.predefined.iter() {

        let mut g = svg!(Group class="stream");

        let mut bg = svg!(Rectangle class="stream-bg"
            x=0 y=0 width=width);
        g = g.add(bg);

        let mut addr: String;
        let mut amount: u128;
        match stream {
            Stream::Immediate{addr:_addr, amount:_amount} => {
                g.set("class", "stream immediate");
                addr = _addr.to_string();
                amount = _amount.u128();
            },
            Stream::Monthly{addr, amount, ..} => {
                g.set("class", "stream monthly");
                addr = _addr.to_string();
                amount = _amount.u128();
            },
            Stream::Daily{addr, amount, ..} => {
                g.set("class", "stream daily");
                addr = _addr.to_string();
                amount = _amount.u128();
            }
        }

        g.set("id", addr.to_string());

        let percent = amount / total;
        g.set("data-percent", percent);

        let h = f64::from(percent * 10000).ln() * 30f64;
        g.set("data-h", h);
        bg.set("height", h);

        doc = doc.add(g)

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
    let day_width = width / ((t_max-t_min)/(24*60*60));
    let week_width = 15 * day_width;
    let n_weeks = 47;
    for i in 0..n_weeks {
        let x = i * week_width;
        grid = grid.add(
            svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.2)"));
    }
    for i in 0..n_weeks/6 {
        let x = i * 6 * week_width;
        grid = grid.add(
            svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.4)"));
    }

    // add grid to document
    doc = doc.add(grid);

    svg::save("docs/schedule.svg", &doc).unwrap();
}
