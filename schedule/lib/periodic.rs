use crate::units::*;

pub fn periodic (
    amount:   u128,
    interval: Seconds,
    elapsed:  Seconds,
    duration: Seconds,
    cliff:    Percentage,
) -> u128 {

    // mutable for clarity:
    let mut vest = 0;

    // start with the cliff amount
    let cliff = cliff as u128;
    if cliff * amount % 100 > 0 { warn_cliff_remainder() }
    let cliff_amount = (cliff * amount / 100) as u128;
    vest += cliff_amount;

    // then for every `interval` since `t_start`
    // add an equal portion of the remaining amount

    // then, from the remaining amount and the number of releases
    // determine the size of the portion
    let post_cliff_amount = amount - cliff_amount;
    let n_total: u128 = (duration / interval).into();
    if post_cliff_amount % n_total > 0 { warn_release_remainder() }
    let portion = post_cliff_amount / n_total;

    // then determine how many release periods have elapsed,
    // up to the maximum; `duration - interval` and `1 + n_elapsed`
    // are used to ensure release happens at the begginning of an interval
    let t_elapsed = Seconds::min(elapsed, duration - interval);
    let n_elapsed = t_elapsed / interval;
    let n_elapsed: u128 = (1 + n_elapsed).into();
    //if t_elapsed % interval > interval / 2 { n_elapsed += 1; }

    // then add that amount to the cliff amount
    vest += portion * n_elapsed;

    //println!("periodic {}/{}={} -> {}", n_elapsed, n_total, n_elapsed/n_total, vest);
    vest
}

fn warn_cliff_remainder () {
    //println!("WARNING: division with remainder for cliff amount")
}

fn warn_release_remainder () {
    //println!("WARNING: division with remainder for release amount")
}

#[test]
fn test_periodic () {
    assert_eq!(periodic( 0, 1, 0, 1, 0),  0);
    assert_eq!(periodic( 1, 1, 0, 1, 0),  1);
    assert_eq!(periodic(15, 1, 0, 3, 0),  5);
    assert_eq!(periodic(15, 1, 1, 3, 0), 10);
    assert_eq!(periodic(15, 1, 2, 3, 0), 15);
    assert_eq!(periodic(15, 1, 3, 3, 0), 15);
    assert_eq!(periodic(15, 1, 4, 3, 0), 15);
}


