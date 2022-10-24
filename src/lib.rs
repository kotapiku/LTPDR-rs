use crate::PDRAnswer::*;
use crate::Verbosity::*;
use std::fmt;
use std::fmt::Debug;

pub mod instances;

pub trait CLat {
    type Info;
    fn le(&self, rhs: &Self) -> (bool, Self::Info);
    fn bot(&self) -> Self;
    fn top(&self) -> Self;
    fn meet(&self, rhs: &Self) -> Self;
}

type KTSeq<T> = Vec<T>;
type KleeneSeq<T> = Vec<T>;
struct KTKl<T>(KTSeq<T>, KleeneSeq<T>);
pub enum PDRAnswer<T> {
    Valid(KTSeq<T>),
    InValid(KleeneSeq<T>),
}

impl<T: Debug> Debug for KTKl<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kt = String::from("[")
            + &self
                .0
                .iter()
                .map(|elem| format!("{:?}", elem))
                .collect::<Vec<String>>()
                .join(", ")
            + "]";
        writeln!(f, "KT: {}", kt)?;
        let kl = String::from("[")
            + &self
                .1
                .iter()
                .rev()
                .map(|elem| format!("{:?}", elem))
                .collect::<Vec<String>>()
                .join(", ")
            + "]";
        write!(f, "Kl: {}", kl)
    }
}
impl<T> fmt::Display for PDRAnswer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Valid(_) => write!(f, "Valid"),
            Self::InValid(_) => write!(f, "InValid"),
        }
    }
}

pub struct Heuristics<T: CLat> {
    pub f_candidate: Box<dyn Fn(&T, &T, &T::Info) -> T>,
    pub f_decide: Box<dyn Fn(&T, &T, &dyn Fn(&T) -> T, &T::Info) -> T>,
    pub f_conflict: Box<dyn Fn(&T, &T, &dyn Fn(&T) -> T, &T::Info) -> T>,
}

pub struct Config {
    pub print: Verbosity,
    // optMaxStep: Option<u32>,
}
pub enum Verbosity {
    PrintAll,
    PrintLength,
    NoPrint,
}

impl Config {
    pub fn default_opt() -> Config {
        Config { print: NoPrint }
    }
}
fn print_kt_kl<T: Debug>(opt: &Config, current: &KTKl<T>, rule: &str) {
    match opt.print {
        NoPrint => (),
        PrintAll => {
            println!("{rule}");
            println!("{:?}", current);
        }
        PrintLength => {
            if (rule == "--Init--") | (rule == "--Unfold--") {
                println!("length of kt seq: {}", current.0.len());
            };
        }
    }
}

pub fn lt_pdr<T: CLat + Debug>(
    opt: Config,
    heuristics: Heuristics<T>,
    f: &dyn Fn(&T) -> T,
    alpha: T,
) -> PDRAnswer<T> {
    // ([X_1, ..., X_{n-1}], [C_{n-1}, ..., C_i])
    let mut current = KTKl(vec![f(&alpha.bot())], Vec::new());
    print_kt_kl(&opt, &current, "--Init--");
    loop {
        let KTKl(xs, cs) = &current;

        if check_valid(xs) {
            return Valid(current.0);
        }
        if xs.len() == cs.len() {
            return InValid(current.1);
        }

        let xn1 = current.0.last().unwrap();
        let (result1, info1) = xn1.le(&alpha);
        if result1 {
            current.0.push(alpha.top());
            current.1 = Vec::new();
            print_kt_kl(&opt, &current, "--Unfold--");
            continue;
        } else {
            match current.1.pop() {
                None => {
                    current
                        .1
                        .push((heuristics.f_candidate)(xn1, &alpha, &info1));
                    print_kt_kl(&opt, &current, "--Candidate--");
                    continue;
                }
                Some(ci) => {
                    // ([X_1, ..., X_{n-1}], [C_{n-1}, ..., C_{i+1}])
                    let i = current.0.len() - current.1.len();
                    let xi1 = current.0.get(i - 2).unwrap();
                    let (result2, info2) = ci.le(&f(xi1));
                    if result2 {
                        let x = (heuristics.f_decide)(xi1, &ci, f, &info2);
                        current.1.push(ci);
                        current.1.push(x);
                        print_kt_kl(&opt, &current, "--Decide--");
                    } else {
                        let x = (heuristics.f_conflict)(xi1, &ci, f, &info2);
                        for v in current.0[..i].iter_mut() {
                            *v = v.meet(&x);
                        }
                        print_kt_kl(&opt, &current, "--Conflict--");
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
        result |= xs[i + 1].le(&xs[i]).0;
    }
    result
}
