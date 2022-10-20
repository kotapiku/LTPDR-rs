extern crate ltpdr;
// use ltpdr::instances::simple_trans::{forward_ps, PS};
// use ltpdr::Verbosity::*;
use ltpdr::*;
use std::collections::HashMap;
// use std::cmp;
use num::traits::{One, Zero};
use num::{one, zero};

fn main() {
    // let init = HashSet::from([0]);
    // let all = &HashSet::from_iter(1..6);
    // let f = forward_ps(&init, &delta1);
    // let alpha = PS {
    //     all,
    //     subset: HashSet::from_iter(1..5),
    // };
    // let result = lt_pdr(Options { print: PrintAll }, &f, alpha);
    // println!("{result}");
}

type KeyInt = u64;
#[derive(Clone, PartialEq)]
pub struct Eps<T>(T, bool); // (v, b) = if b then v+epsilon else v
pub struct ProbMapE<T> {
    // s |-> if s in supp(map) then map(s) else n
    map: HashMap<KeyInt, Eps<T>>,
    other: T,
}

impl<T: PartialOrd> Eps<T> {
    fn le(&self, b: &Self) -> bool {
        match (self, b) {
            (Eps(n1, true), Eps(n2, false)) => n1.lt(n2),
            (Eps(n1, _), Eps(n2, _)) => n1.le(n2),
        }
    }
}

impl<T: Clone> ProbMapE<T> {
    fn get(&self, key: &KeyInt) -> Eps<T> {
        self.map
            .get(key)
            .map_or(Eps(self.other.clone(), false), |eps| eps.clone())
    }
}

impl<T: Zero + One + Ord + Clone> CLat for ProbMapE<T> {
    type Info = HashMap<KeyInt, Eps<T>>;
    fn le(&self, rhs: &Self) -> (bool, Self::Info) {
        if rhs.other.is_one() && rhs.map.len().is_one() {
            // X_{n-1}(s0) <= lambda
            let (s0, eps) = rhs.map.iter().next().unwrap();
            (self.get(s0).le(eps), HashMap::new())
        } else if self.other.is_zero() || rhs.other.is_one() {
            let mut map = self.map.clone();
            map.retain(|s, v| !(v.le(&rhs.get(s))));
            (map.is_empty(), map)
        } else {
            // self.other > rhs.other
            (false, HashMap::new())
        }
    }
    fn bot(&self) -> Self {
        ProbMapE {
            map: HashMap::new(),
            other: zero(),
        }
    }
    fn top(&self) -> Self {
        ProbMapE {
            map: HashMap::new(),
            other: one(),
        }
    }
    fn meet(&self, rhs: &Self) -> Self {
        let other = self.other.clone().min(rhs.other.clone());
        let mut map = self.map.clone();
        for (s, v) in rhs.map.clone().into_iter() {
            if let Some(v2) = map.get_mut(&s) {
                if v.le(v2) {
                    *v2 = v;
                };
            } else {
                map.insert(s, v);
            };
        }
        map.retain(|_, v| *v != Eps(other.clone(), false));
        ProbMapE { map, other }
    }
}

pub type Delta<T> = HashMap<KeyInt, HashMap<KeyInt, T>>;
fn heuristics_mdp<T: Zero + One + Ord + Clone>(
    delta: Delta<T>,
    bad: HashMap<KeyInt, bool>,
) -> Heuristics<ProbMapE<T>> {
    let f_candidate = Box::new(
        |_xn1: &ProbMapE<T>, alpha: &ProbMapE<T>, _info: &HashMap<KeyInt, Eps<T>>| {
            if alpha.other.is_one() && alpha.map.len().is_one() {
                let (&s0, eps) = alpha.map.iter().next().unwrap();
                ProbMapE {
                    map: HashMap::from([(s0, Eps(eps.0.clone(), true))]),
                    other: zero(),
                }
            } else {
                panic!("invalid form in candidate");
            }
        },
    );
    let f_decide = Box::new(
        |xi1: &ProbMapE<T>,
         ci: &ProbMapE<T>,
         f: &dyn Fn(&ProbMapE<T>) -> ProbMapE<T>,
         info: &HashMap<KeyInt, Eps<T>>| {
            // TODO
            panic!("TODO")
        },
    );
    let f_conflict = Box::new(
        |xi1: &ProbMapE<T>,
         _ci: &ProbMapE<T>,
         f: &dyn Fn(&ProbMapE<T>) -> ProbMapE<T>,
         info: &HashMap<KeyInt, Eps<T>>| {
            let flag = info.values().any(|eps| eps.1);
            let mut map = info.clone();
            for (s, eps) in map.iter_mut() {
                if flag {
                    eps.1 = false;
                } else {
                    *eps = Eps(f(xi1).get(s).0, false);
                };
            }
            ProbMapE { map, other: one() }
        },
    );
    Heuristics {
        f_candidate,
        f_decide,
        f_conflict,
    }
}
