extern crate LTPDR_rs;
use std::cmp;
use std::collections::HashSet;
use std::hash::Hash;
use LTPDR_rs::*;

fn main() {
    let init = HashSet::from([1]);
    let all = HashSet::from_iter(1..6);
    let f = forward_ps(&init, &delta1);
    let alpha = PS {
        all,
        subset: HashSet::from_iter(1..5),
    };
    let result = lt_pdr(&f, alpha);
    println!("{}", result);
}

fn delta1(s: &u32) -> HashSet<u32> {
    match s {
        1 => HashSet::from([2, 3]),
        2 => HashSet::from([1, 3]),
        4 => HashSet::from([5]),
        _ => HashSet::new(),
    }
}

fn forward_ps<'a, T: Eq + Hash + Clone>(
    init: &'a HashSet<T>,
    delta: &'a dyn Fn(&T) -> HashSet<T>,
) -> impl Fn(&PS<T>) -> PS<T> + 'a {
    |ps| {
        let mut subset: HashSet<T> = HashSet::new();
        subset = subset.union(init).cloned().collect();
        for s in ps.subset.iter() {
            subset = subset.union(&delta(s)).cloned().collect();
        }
        PS {
            all: ps.all.clone(),
            subset,
        }
    }
}

#[derive(Debug)]
struct PS<T> {
    all: HashSet<T>,
    subset: HashSet<T>,
}

impl<T: Eq + Hash + Clone> CLat for PS<T> {
    type Info = ();
    fn leq(&self, rhs: &Self) -> (bool, Self::Info) {
        (self.subset.is_subset(&rhs.subset), ())
    }
    fn bot(&self) -> Self {
        PS {
            all: self.all.clone(),
            subset: HashSet::new(),
        }
    }
    fn top(&self) -> Self {
        PS {
            all: self.all.clone(),
            subset: self.all.clone(),
        }
    }
    fn meet(&self, rhs: &Self) -> Self {
        PS {
            all: self.all.clone(),
            subset: self.subset.intersection(&rhs.subset).cloned().collect(),
        }
    }
}

impl<T: Clone + Hash + Eq> Heuristics<PS<T>> for PS<T> {
    fn f_candidate(&self, alpha: &PS<T>, _: &<PS<T> as CLat>::Info) -> PS<T> {
        PS {
            all: self.all.clone(),
            subset: self.subset.difference(&alpha.subset).cloned().collect(),
        }
    }
    fn f_decide(
        &self,
        ci: &PS<T>,
        f: &dyn Fn(&PS<T>) -> PS<T>,
        _solver: &<PS<T> as CLat>::Info,
    ) -> PS<T> {
        let mut subset: HashSet<T> = self.subset.clone();
        subset.retain(|x: &T| {
            let fx: PS<T> = f(&PS {
                all: self.all.clone(),
                subset: HashSet::from([x.clone()]),
            });
            !(fx.subset.is_disjoint(&ci.subset))
        });
        PS {
            all: self.all.clone(),
            subset,
        }
    }
    fn f_conflict(
        &self,
        ci: &PS<T>,
        f: &dyn Fn(&PS<T>) -> PS<T>,
        _solver: &<PS<T> as CLat>::Info,
    ) -> PS<T> {
        let fxi1 = f(self);
        let ci_fxi1: HashSet<T> = ci.subset.difference(&fxi1.subset).cloned().collect();
        let subset: HashSet<T> = self.all.difference(&ci_fxi1).cloned().collect();
        PS {
            all: self.all.clone(),
            subset,
        }
    }
}