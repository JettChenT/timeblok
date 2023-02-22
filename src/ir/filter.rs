use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::environment::Environment;
use crate::ir::NumVal::{Number, Unsure};
use crate::ir::*;
use crate::resolver::{resolve_date, resolve_range};

pub trait Filter<T>: Debug + DynClone {
    fn check(&self, value: &T, env: Option<&Environment>) -> bool;
    fn filter(&self, values: Vec<T>, env: Option<&Environment>) -> Vec<T> {
        // PERFORMANCE: Change this to parallel execution, use references and lifetimes to improve memory efficiency
        let mut res = vec![];
        for value in values {
            if self.check(&value, env) {
                res.push(value);
            }
        }
        res
    }
}

dyn_clone::clone_trait_object!(<T> Filter<T>);

#[derive(Debug, Clone)]
pub enum Op {
    OR,
    And,
}

pub type BDF<T> = Box<dyn Filter<T>>;

#[derive(Debug, Clone)]
pub struct BinFilt<T: Debug> {
    pub lhs: BDF<T>,
    pub rhs: BDF<T>,
    pub op: Op,
}

impl<T: Debug + Clone> Filter<T> for BinFilt<T> {
    fn check(&self, value: &T, env: Option<&Environment>) -> bool {
        match self.op {
            Op::OR => self.lhs.check(value, env) || self.rhs.check(value, env),
            Op::And => self.lhs.check(value, env) && self.rhs.check(value, env),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExcludeFilt<T> {
    pub target: BDF<T>,
}

impl<T: Debug + Clone> ExcludeFilt<T> {
    pub fn new(target: BDF<T>) -> Self {
        ExcludeFilt { target }
    }
}

impl<T: Debug + Clone> Filter<T> for ExcludeFilt<T> {
    fn check(&self, value: &T, env: Option<&Environment>) -> bool {
        !self.target.check(value, env)
    }
}

impl Filter<NumVal> for NumRange {
    fn check(&self, value: &NumVal, _env: Option<&Environment>) -> bool {
        match *value {
            Number(target) => match (&self.start, &self.end) {
                (Unsure, Unsure) => true,
                (Unsure, Number(nd)) => target <= *nd,
                (Number(st), Unsure) => target >= *st,
                (Number(st), Number(nd)) => target >= *st && target <= *nd,
            },
            _ => true,
        }
    }
}

impl Filter<NumVal> for NumVal {
    fn check(&self, value: &NumVal, _env: Option<&Environment>) -> bool {
        if matches!(self, Unsure) {
            true
        } else {
            self == value
        }
    }
}

impl Filter<ExactDate> for ExactRange {
    fn check(&self, value: &ExactDate, _env: Option<&Environment>) -> bool {
        match self {
            ExactRange::TimeRange(tr) => {
                // unwrap or return false
                let start = tr.start.date.to_chrono().unwrap();
                let end = tr.end.date.to_chrono().unwrap();
                let target = value.to_chrono().unwrap();
                target >= start && target <= end
            }
            ExactRange::AllDay(date) => date == value,
        }
    }
}

impl Filter<Date> for Range {
    fn check(&self, value: &Date, env: Option<&Environment>) -> bool {
        // TODO: Change Resolving to Environment based
        let exact_range = resolve_range(self, env.unwrap()).unwrap();
        let exact_date = resolve_date(value, env.unwrap()).unwrap();
        exact_range.check(&exact_date, env)
    }
}

impl Filter<NumVal> for FlexField {
    fn check(&self, value: &NumVal, env: Option<&Environment>) -> bool {
        match self {
            FlexField::NumRange(nr) => nr.check(value, env),
            FlexField::NumVal(nv) => nv.check(value, env),
        }
    }
}

impl Filter<ExactDate> for FlexDate {
    fn check(&self, value: &ExactDate, env: Option<&Environment>) -> bool {
        self.year.check(&Number(value.year as i64), env)
            && self.month.check(&Number(value.month as i64), env)
            && self.day.check(&Number(value.day as i64), env)
    }
}

impl Filter<Date> for FlexDate {
    fn check(&self, value: &Date, env: Option<&Environment>) -> bool {
        let exact_date = resolve_date(value, env.unwrap()).unwrap();
        self.year.check(&Number(exact_date.year as i64), env)
            && self.month.check(&Number(exact_date.month as i64), env)
            && self.day.check(&Number(exact_date.day as i64), env)
    }
}

// Add a unit test for filters
// Thank you copilot
mod tests {
    use super::*;
    use crate::ir::Range;
    use crate::ir::TimeRange;
    #[test]
    fn test_flex_date() {
        let fd = FlexDate {
            year: Box::new(Number(2023)),
            month: Box::new(NumRange {
                start: Number(6),
                end: Number(10),
            }),
            day: Box::new(NumRange {
                start: Number(8),
                end: Number(15),
            }),
        };
        assert!(fd.check(
            &ExactDate {
                year: 2023,
                month: 6,
                day: 8
            },
            None
        ));
        assert!(!fd.check(
            &ExactDate {
                year: 2023,
                month: 6,
                day: 7
            },
            None
        ));
        assert!(!fd.check(
            &ExactDate {
                year: 2023,
                month: 5,
                day: 8
            },
            None
        ));
        assert!(!fd.check(
            &ExactDate {
                year: 2022,
                month: 6,
                day: 8
            },
            None
        ));
    }

    #[test]
    fn test_time_range() {
        let trange = TimeRange {
            start: DateTime::from_ymd(2020, 1, 3),
            end: DateTime::from_ymd(2020, 2, 1),
        };
        eprintln!("{:?}", trange);
        let range = Range::Time(trange);
        let env = Environment::from_exact(ExactDateTime::from_ymd_hms(2020, 1, 1, 1, 1, 1));
        assert!(range.check(&Date::from_ymd(2020, 1, 3), Some(&env)));
        assert!(!range.check(&Date::from_ymd(2020, 1, 2), Some(&env)));
        assert!(range.check(&Date::from_ymd(2020, 2, 1), Some(&env)));
    }
    #[test]
    fn test_tree() {
        // Testing combined filters
        let orfilt = BinFilt {
            lhs: Box::new(NumRange {
                start: Number(1),
                end: Number(5),
            }),
            rhs: Box::new(NumRange {
                start: Number(10),
                end: Number(15),
            }),
            op: Op::OR,
        };
        assert!(orfilt.check(&Number(1), None));
        assert!(!orfilt.check(&Number(8), None));
        assert!(orfilt.check(&Number(13), None));
        let andfilt = BinFilt {
            lhs: Box::new(NumRange {
                start: Number(1),
                end: Number(8),
            }),
            rhs: Box::new(NumRange {
                start: Number(3),
                end: Unsure,
            }),
            op: Op::And,
        };
        assert!(!andfilt.check(&Number(1), None));
        assert!(andfilt.check(&Number(8), None));
        assert!(!andfilt.check(&Number(13), None));
        let combfilt = BinFilt {
            lhs: Box::new(ExcludeFilt {
                target: Box::new(orfilt),
            }),
            rhs: Box::new(andfilt),
            op: Op::OR,
        };
        assert!(combfilt.check(&Number(3), None));
        assert!(combfilt.check(&Number(9), None));
        assert!(!combfilt.check(&Number(2), None));
    }
}
