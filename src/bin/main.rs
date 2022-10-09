extern crate LTPDR_rs;
use LTPDR_rs::*;
use std::cmp;

fn main() {
  let result = lt_pdr(f, alpha);
  println!("{}", result);
}

#[derive(Copy, Clone)]
struct Nat {
    n: u32,
    is_top: bool,
}

impl CLat for Nat {
    type Info = ();
    fn leq(&self, rhs: &Self) -> (bool, Self::Info) {
        if !self.is_top & !rhs.is_top {
            (self.n.le(&rhs.n), ())
        } else {
            (!self.is_top | rhs.is_top, ())
        }
    }

    fn bot(&self) -> Self { Nat { n: 0, is_top: false } }
    fn top(&self) -> Self { Nat { n: 0, is_top: true } }
    fn meet(&self, rhs: &Self) -> Self {
        if self.is_top {
            *rhs
        } else if rhs.is_top {
            *self
        } else {
            Nat { n: cmp::min(self.n, rhs.n), is_top: false }
        }
    }
}
