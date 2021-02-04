use std::env::current_dir;
use std::fs::create_dir_all;

use svg::Document;
use svg::node::Text as TextNode;
use svg::node::element::{Rectangle, Group, Text, Line, Polyline, Circle};
use svg::node::element::path::Data;

use sienna_mgmt::schedule::SCHEDULE;
use sienna_mgmt::constants::{ONE_SIENNA, DAY, MONTH};
use sienna_mgmt::types::{Stream, Vesting, Interval};
use sienna_mgmt::vesting::claimable;

macro_rules! svg {
    ($El:ident $($attr:ident = $value:expr)+) => {
        $El::new()
        $(.set(str::replace(stringify!($attr), "_", "-"), $value))*
    };
    ($text:expr) => {
        TextNode::new($text)
    }
}

fn main () {

    let total = SCHEDULE.total.u128() / ONE_SIENNA;

    let t_min = 0;
    let mut t_max: u64 = 0;

    for Stream { addr, amount, vesting } in SCHEDULE.predefined.iter() {
        let mut _start_at = 0;
        let mut _duration = 0;
        match vesting {
            Vesting::Periodic {start_at, duration, ..} => {
                _start_at = *start_at;
                _duration = *duration;
            },
            _ => {}
        }
        if _start_at > t_max { t_max = _start_at }
        if _start_at + _duration > t_max { t_max = _start_at + _duration }
    }
    t_max += MONTH;

    let width = 2000f64;
    let height = 3500f64;
    let margin = 500f64;
    let t_scale = width / (t_max - t_min) as f64;
    let viewbox = (-0.75*margin, 0.5*margin, width+2.0*margin, height+1.0*margin);

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
    for Stream { addr, amount, vesting } in SCHEDULE.predefined.iter() {

        let amount = amount.u128() / ONE_SIENNA;

        let mut g = svg!(Group id=addr.to_string() class="stream"
            transform=format!("translate(0,{})", y));

        let mut bg = svg!(Rectangle class="stream-bg"
            x=0 y=0 width=width fill="transparent");

        // determine height from percentage of total
        let percent = amount as f64 / total as f64;
        g = g.set("data-percent", percent.to_string());

        // by applying some random repeated-log scaling
        // which maintains the relative proportions
        // but also maintains visibility of 5% vs 37%
        let h = ((percent * 10000.0).ln().ln()*100.0).ln() * 50.0;
        g = g.set("data-h", h);

        g = g.add(svg!(Line class="stream-border"
            x1=0 y1=h x2=width y2=h
            stroke="#000" stroke_width=0.5));

        g = g.add(svg!(Text class="stream-id"
            x=width+10.0 y=h/2.0 text_anchor="start")
            .add(svg!(&addr.to_string())));

        let portion: u128;
        let vestings: u128;
        let mut cliff_amount: u128 = 0;
        let mut start_day: u128 = 0;

        match vesting {

            Vesting::Immediate {} => {
                g = g.set("class", "stream immediate");
                bg = bg.set("fill", "rgba(64,255,64,0.2");
                cliff_amount = amount;
                portion = 0;
                vestings = 0;
            },

            Vesting::Periodic {interval, start_at, duration, cliff} => {
                start_day = (start_at / DAY).into();
                cliff_amount = *cliff as u128 * amount / 100;
                match interval {
                    Interval::Daily => {
                        g = g.set("class", "stream daily");
                        vestings = (duration / DAY) as u128;
                    },
                    Interval::Monthly => {
                        g = g.set("class", "stream monthly");
                        vestings = (duration / MONTH) as u128;
                    }
                };
                portion = (amount - cliff_amount) / vestings;
            },

        };

        // set height of background rectangle
        // and add it to the group
        bg = bg.set("height", h);
        g = g.add(bg);

        // render points and polyline that
        // correspond to vesting progress
        let mut now = t_min;
        let mut points = String::new();
        let mut last_x = 0.0;
        let mut last_y = 0.0;
        while now < t_max {
            last_x = now as f64 * t_scale;
            let vested = claimable(addr, &vec![], 0, now) / ONE_SIENNA;
            let y = h - h * (vested as f64 / amount as f64);
            let point = format!("{},{} {},{} ", last_x, last_y, last_x, y);
            //println!("{} @{} {}/{} = {}", &addr, &now, &vested, &amount, &point);
            points.push_str(&point);
            last_y = y;
            now += DAY;
            g = g.add(svg!(Circle cx=last_x cy=y r=1 fill="red" data_t=now data_a=vested.to_string()));
        }
        points.push_str(&format!("{},{} {},{} {},{}", width, 0, width, h, 0, h));
        g = g.add(svg!(Polyline class="stream-flow"
            fill="rgba(64,255,64,0.2)"
            stroke="rgba(0,128,0,0.5)"
            stroke_width=0.5
            points=points));

        g = g.add(svg!(Text class="stream-amount" font_weight="bold"
            x=-10 y=h*0.2 text_anchor="end")
            .add(svg!(format!("{} SIENNA", amount.to_string()))));
        g = g.add(svg!(Text class="stream-amount"
            x=-10 y=h*0.4 text_anchor="end")
            .add(svg!(format!("{} of total", percent))));
        g = g.add(svg!(Text class="stream-amount"
            x=-10 y=h*0.6 text_anchor="end")
            .add(svg!(format!("{} at day {}, plus", cliff_amount, start_day))));
        g = g.add(svg!(Text class="stream-amount"
            x=-10 y=h*0.8 text_anchor="end")
            .add(svg!(format!("{} per {} vestings", portion, vestings))));

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
            .add(svg!("T=0")));
    grid = grid.add(
        svg!(Text x=width y=-15 text_anchor="start")
            .add(svg!(format!("T={} days", (t_max-t_min)/DAY))));

    // grid lines - thin
    let n_days = (t_max - t_min) / DAY;
    let day_width = DAY as f64 * t_scale;
    let mut i = 0;
    while i <= n_days {
        let x = i as f64 * day_width;
        grid = grid.add(
            svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.2)"));
        i += 1;
    }

    // grid lines - thick
    let n_months = (t_max - t_min) / MONTH;
    let month_width = MONTH as f64 * t_scale;
    let mut i = 0;
    while i <= n_months {
        let x = i as f64 * month_width;
        grid = grid.add(
            svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.4)"));
        i += 1;
    }
    //let days_in_month = 30;
    //let mut n_months = n_days/weeks_in_month;
    //if n_weeks % weeks_in_month > 0 { n_months += 1; }
    //for i in 0..n_months {
        //let x = i as f64 * weeks_in_month as f64 * week_width;
        //grid = grid.add(
            //svg!(Line x1=x x2=x y1=0 y2=height stroke="rgba(0,0,0,0.4)"));
    //}

    // add grid to document
    doc = doc.add(grid);

    svg::save("docs/schedule.svg", &doc).unwrap();

    //println!("{} {}", t_min, t_max/MONTH);
}
