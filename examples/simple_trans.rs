extern crate ltpdr;
use ltpdr::instances::simple_trans::{forward_ps, PS};
use ltpdr::Verbosity::*;
use ltpdr::*;
use std::collections::HashSet;

fn main() {
    let init = HashSet::from([1]);
    let all = &HashSet::from_iter(1..6);
    let f = forward_ps(&init, &delta1);
    let alpha = PS {
        all,
        subset: HashSet::from_iter(1..5),
    };
    let result = lt_pdr(Options { print: PrintAll }, &f, alpha);
    println!("{result}");
}

fn delta1(s: &u32) -> HashSet<u32> {
    match s {
        1 => HashSet::from([2, 3]),
        2 => HashSet::from([1, 3]),
        4 => HashSet::from([5]),
        _ => HashSet::new(),
    }
}
