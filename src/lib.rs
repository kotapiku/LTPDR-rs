use crate::PDRAnswer::*;
use std::fmt;
use std::fmt::Debug;

pub trait CLat {
    // type Info T
    type Info;
    fn leq(&self, rhs: &Self) -> (bool, Self::Info);
    fn bot(&self) -> Self;
    fn top(&self) -> Self;
    fn meet(&self, rhs: &Self) -> Self;
}

type KTSeq<T> = Vec<T>;
type KleeneSeq<T> = Vec<T>;
#[derive(Debug)]
struct KTKl<T>(KTSeq<T>, KleeneSeq<T>);
pub enum PDRAnswer<T> {
    Valid(KTSeq<T>),
    InValid(KleeneSeq<T>),
}
impl<T> fmt::Display for PDRAnswer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Valid(_) => write!(f, "Valid"),
            Self::InValid(_) => write!(f, "InValid"),
        }
    }
}

// TODO: T = self
pub trait Heuristics<T: CLat> {
    fn f_candidate(&self, alpha: &T, solver: &T::Info) -> T;
    fn f_decide(&self, ci: &T, f: &dyn Fn(&T) -> T, solver: &T::Info) -> T;
    fn f_conflict(&self, ci: &T, f: &dyn Fn(&T) -> T, solver: &T::Info) -> T;
}

pub fn lt_pdr<T: CLat + Heuristics<T> + Debug>(f: &dyn Fn(&T) -> T, alpha: T) -> PDRAnswer<T> {
    // ([X_1, ..., X_{n-1}], [C_{n-1}, ..., C_i])
    let mut current = KTKl(vec![f(&alpha.bot())], Vec::new());
    loop {
        println!("{:?}", &current);
        let KTKl(xs, cs) = &current;

        if check_valid(xs) {
            return Valid(current.0); // valid
        }
        if xs.len() == cs.len() {
            return InValid(current.1); // invalid
        }

        let xn1 = current.0.last().unwrap();
        let (result1, solver1) = xn1.leq(&alpha);
        if result1 {
            // unfold
            println!("Unfold");
            current.0.push(alpha.top());
            current.1 = Vec::new();
            continue;
        } else {
            match current.1.pop() {
                None => {
                    // candidate
                    println!("Candidate");
                    current.1.push(xn1.f_candidate(&alpha, &solver1));
                    continue;
                }
                Some(ci) => {
                    // ([X_1, ..., X_{n-1}], [C_{n-1}, ..., C_{i+1}])
                    // n-1 - ((n-1) - (i+1) + 1) = i
                    let i = current.0.len() - current.1.len();
                    let xi1 = current.0.get(i - 2).unwrap();
                    let (result2, solver2) = ci.leq(&f(xi1));
                    if result2 {
                        // decide
                        println!("Decide");
                        let x = xi1.f_decide(&ci, f, &solver2);
                        current.1.push(ci);
                        current.1.push(x);
                        continue;
                    } else {
                        // conflict
                        println!("Conflict");
                        let x = xi1.f_conflict(&ci, f, &solver2);
                        for v in current.0[..i].iter_mut() {
                            *v = v.meet(&x);
                        }
                    }
                    continue;
                }
            };
        }
    }
}

fn check_valid<T: CLat>(xs: &KTSeq<T>) -> bool {
    let mut result = false;
    for i in 0..(xs.len() - 1) {
        result |= xs[i + 1].leq(&xs[i]).0;
    }
    result
}
