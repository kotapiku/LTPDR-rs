extern crate ltpdr;
// use ltpdr::instances::simple_trans::{forward_ps, PS};
use good_lp::{default_solver, variable, variables, Solution, SolverModel};
use ltpdr::Verbosity::*;
use ltpdr::*;
use num::traits::{One, Zero};
use num::{one, zero};
use std::collections::HashMap;
use std::ops::{Add, Mul};

fn main() {
    let bad_states = |n: KeyInt| n == 5;
    let state_num = 6;
    let delta1: Delta<f64> = |s| match s {
        0 => vec![HashMap::from([(1, 0.5), (2, 0.5)])],
        1 => vec![HashMap::from([(0, 0.5), (3, 0.5)])],
        2 => vec![
            HashMap::from([(4, 0.5), (5, 0.5)]),
            HashMap::from([(1, 1.0)]),
        ],
        3 => vec![HashMap::from([(4, 1f64 / 3f64), (5, 2f64 / 3f64)])],
        4 => vec![HashMap::from([(4, 1.0)])],
        5 => vec![HashMap::from([(5, 1.0)])],
        _ => panic!("access invalid states"),
    };
    let f = backward_mdp(state_num, &delta1, &bad_states);
    let d_lambda = ProbMapE {
        map: HashMap::from([(0, Eps(0.5, false))]),
        other: 1.0,
    };
    let result = lt_pdr(
        Config { print: PrintAll },
        heuristics_mdp(delta1, bad_states),
        &f,
        d_lambda,
    );
    println!("{result}");
}

type Delta<T> = fn(KeyInt) -> Vec<HashMap<KeyInt, T>>;
fn backward_mdp<'a, T: Zero + One + Clone + Mul + Add + PartialOrd>(
    state_num: u64,
    delta: &'a Delta<T>,
    bad_states: &'a dyn Fn(KeyInt) -> bool,
) -> impl Fn(&ProbMapE<T>) -> ProbMapE<T> + 'a {
    move |prob_map| {
        let map: HashMap<KeyInt, Eps<T>> = HashMap::from_iter((0..state_num).filter_map(|s| {
            if bad_states(s) {
                None
            } else {
                let other: T = delta(s)
                    .iter()
                    .map(|f| {
                        let mut result: T = zero();
                        for (ns, p) in f.iter() {
                            result = result + prob_map.get(ns).0 * p.clone();
                        }
                        result
                    })
                    .fold(zero(), |acc, x| if acc.lt(&x) { x } else { acc });

                Some((s, Eps(other, false)))
            }
        }));
        ProbMapE { map, other: one() }
    }
}

type KeyInt = u64;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Eps<T>(T, bool); // (v, b) = if b then v+epsilon else v
#[derive(PartialEq, Eq, Debug)]
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

impl<T: Zero + One + PartialOrd + Clone> CLat for ProbMapE<T> {
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
        let other = if self.other.le(&rhs.other) {
            self.other.clone()
        } else if self.other.gt(&rhs.other) {
            rhs.other.clone()
        } else {
            panic!("NAN occurs");
        };
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

fn heuristics_mdp<T: Zero + One + PartialOrd + Clone>(
    delta: Delta<T>,
    bad: fn(KeyInt) -> bool,
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
