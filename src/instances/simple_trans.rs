use crate::*;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug)]
pub struct PS<'a, T> {
    pub all: &'a HashSet<T>,
    pub subset: HashSet<T>,
}

impl<'a, T: Eq + Hash + Clone> CLat for PS<'a, T> {
    type Info = ();
    fn leq(&self, rhs: &Self) -> (bool, Self::Info) {
        (self.subset.is_subset(&rhs.subset), ())
    }
    fn bot(&self) -> Self {
        PS {
            all: self.all,
            subset: HashSet::new(),
        }
    }
    fn top(&self) -> Self {
        PS {
            all: self.all,
            subset: self.all.clone(),
        }
    }
    fn meet(&self, rhs: &Self) -> Self {
        PS {
            all: self.all,
            subset: self.subset.intersection(&rhs.subset).cloned().collect(),
        }
    }
}

impl<'a, T: Clone + Hash + Eq> Heuristics<PS<'a, T>> for PS<'a, T> {
    fn f_candidate(&self, alpha: &PS<'a, T>, _: &<PS<'a, T> as CLat>::Info) -> PS<'a, T> {
        PS {
            all: self.all,
            subset: self.subset.difference(&alpha.subset).cloned().collect(),
        }
    }
    fn f_decide(
        &self,
        ci: &PS<'a, T>,
        f: &dyn Fn(&PS<'a, T>) -> PS<'a, T>,
        _solver: &<PS<'a, T> as CLat>::Info,
    ) -> PS<'a, T> {
        let mut subset: HashSet<T> = self.subset.clone();
        subset.retain(|x: &T| {
            let fx: PS<T> = f(&PS {
                all: self.all,
                subset: HashSet::from([x.clone()]),
            });
            !(fx.subset.is_disjoint(&ci.subset))
        });
        PS {
            all: self.all,
            subset,
        }
    }
    fn f_conflict(
        &self,
        ci: &PS<'a, T>,
        f: &dyn Fn(&PS<'a, T>) -> PS<'a, T>,
        _solver: &<PS<'a, T> as CLat>::Info,
    ) -> PS<'a, T> {
        let fxi1 = f(self);
        let ci_fxi1: HashSet<T> = ci.subset.difference(&fxi1.subset).cloned().collect();
        let subset: HashSet<T> = self.all.difference(&ci_fxi1).cloned().collect();
        PS {
            all: self.all,
            subset,
        }
    }
}

pub fn forward_ps<'a, T: Eq + Hash + Clone>(
    init: &'a HashSet<T>,
    delta: &'a dyn Fn(&T) -> HashSet<T>,
) -> impl Fn(&PS<'a, T>) -> PS<'a, T> + 'a {
    |ps| {
        let mut subset: HashSet<T> = HashSet::new();
        subset = subset.union(init).cloned().collect();
        for s in ps.subset.iter() {
            subset = subset.union(&delta(s)).cloned().collect();
        }
        PS {
            all: ps.all,
            subset,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instances::simple_trans::{forward_ps, PS};
    use crate::*;
    use std::collections::HashSet;
    fn is_valid<T>(result: PDRAnswer<T>) -> bool {
        match result {
            Valid(_) => true,
            InValid(_) => false,
        }
    }

    fn delta1(s: &u32) -> HashSet<u32> {
        match s {
            1 => HashSet::from([2, 3]),
            2 => HashSet::from([1, 3]),
            4 => HashSet::from([5]),
            _ => HashSet::new(),
        }
    }

    #[test]
    fn case1() {
        let init = HashSet::from([1]);
        let all = HashSet::from_iter(1..6);
        let f = forward_ps(&init, &delta1);
        let alpha = PS {
            all,
            subset: HashSet::from_iter(1..5),
        };
        let result = lt_pdr(Options::default_opt(), &f, alpha);
        assert!(is_valid(result));
    }
    #[test]
    fn case2() {
        let init = HashSet::from([1]);
        let all = HashSet::from_iter(1..6);
        let f = forward_ps(&init, &delta1);
        let alpha = PS {
            all,
            subset: HashSet::from_iter(1..3),
        };
        let result = lt_pdr(Options::default_opt(), &f, alpha);
        assert!(!is_valid(result));
    }
}
