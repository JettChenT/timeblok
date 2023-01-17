use std::fmt::Debug;

use crate::ir::*;
use crate::ir::NumVal::{Number, Unsure};

pub trait Filter<T>: Debug{
    fn check(&self, value:&T) -> bool;
    fn filter(&self, values:Vec<T>) -> Vec<T>{
        // PERFORMANCE: Change this to parallel execution, use references and lifetimes to improve memory efficiency
        let mut res = vec![];
        for value in values {
            if self.check(&value) {
                res.push(value);
            }
        }
        res
    }
}

#[derive(Debug)]
pub enum Op{
    OR,
    AND
}

pub type BDF<T> = Box<dyn Filter<T>>;


#[derive(Debug)]
pub struct BinFilt<T: Debug>{
    pub lhs: BDF<T>,
    pub rhs: BDF<T>,
    pub op: Op
}

impl<T: Debug> Filter<T> for BinFilt<T> {
    fn check(&self, value: &T) -> bool {
        match self.op {
            Op::OR => self.lhs.check(value) || self.rhs.check(value),
            Op::AND => self.lhs.check(value) && self.rhs.check(value)
        }
    }
}


#[derive(Debug)]
pub struct ExcludeFilt<T>{
    pub target: BDF<T>
}

impl<T:Debug> Filter<T> for ExcludeFilt<T> {
    fn check(&self, value: &T) -> bool {
        !self.target.check(value)
    }
}

impl Filter<NumVal> for NumRange{
    fn check(&self, value: &NumVal) -> bool {
        match *value {
            Number(target) => {
                match self.start {
                    Unsure => match self.end {
                        Unsure => true,
                        Number(n) => target <= n
                    },
                    Number(st) => {
                        match self.end {
                            Unsure => target >= st,
                            Number(nd) => target >= st && target <= nd
                        }
                    }
                }
            },
            _ => true
        }
    }
}

impl Filter<NumVal> for NumVal{
    fn check(&self, value: &NumVal) -> bool {
         if matches!(self, Unsure){true}
         else {
            self==value
         }
    }
}

impl Filter<ExactDate> for ExactRange{
    fn check(&self, value: &ExactDate) -> bool {
        match self {
            ExactRange::TimeRange(tr) => {
                // unwrap or return false
                let start = tr.start.to_chrono().unwrap().date_naive();
                let end = tr.end.to_chrono().unwrap().date_naive();
                let target = value.to_chrono().unwrap();
                target >= start && target <= end
            },
            ExactRange::AllDay(date)=>{
                date == value
            }
        }
    }
}

impl Filter<NumVal> for FlexField{
    fn check(&self, value: &NumVal) -> bool {
        match self {
            FlexField::NumRange(nr) => nr.check(value),
            FlexField::NumVal(nv) => nv.check(value)
        }
    }
}

impl Filter<ExactDate> for FlexDate{
    fn check(&self, value: &ExactDate) -> bool {
         self.year.check(&Number(value.year as i64)) &&
            self.month.check(&Number(value.month as i64)) &&
            self.day.check(&Number(value.day as i64))
    }
}

// Add a unit test for filters
// Thank you copilot
mod tests{
    use super::*;
    #[test]
    fn test_flex_date(){
        let fd = FlexDate{
            year: FlexField::NumVal(Number(2023)),
            month: FlexField::NumRange(NumRange{start: Number(6), end: Number(10)}),
            day: FlexField::NumRange(NumRange{start: Number(8), end: Number(15)})
        };
        assert!(fd.check(&ExactDate{year: 2023, month: 6, day: 8}));
        assert!(!fd.check(&ExactDate{year: 2023, month: 6, day: 7}));
        assert!(!fd.check(&ExactDate{year: 2023, month: 5, day: 8}));
        assert!(!fd.check(&ExactDate{year: 2022, month: 6, day: 8}));
    }

    #[test]
    fn test_tree(){
        // Testing combined filters
        let orfilt = BinFilt{
            lhs: Box::new(NumRange{start: Number(1), end: Number(5)}),
            rhs: Box::new(NumRange{start: Number(10), end: Number(15)}),
            op: Op::OR
        };
        assert!(orfilt.check(&Number(1)));
        assert!(!orfilt.check(&Number(8)));
        assert!(orfilt.check(&Number(13)));
        let andfilt = BinFilt{
            lhs: Box::new(NumRange{start: Number(1), end: Number(8)}),
            rhs: Box::new(NumRange{start: Number(3), end: Unsure}),
            op: Op::AND
        };
        assert!(!andfilt.check(&Number(1)));
        assert!(andfilt.check(&Number(8)));
        assert!(!andfilt.check(&Number(13)));
        let combfilt = BinFilt{
            lhs: Box::new(
                ExcludeFilt{
                    target: Box::new(orfilt)
                }
            ),
            rhs: Box::new(andfilt),
            op: Op::OR
        };
        assert!(combfilt.check(&Number(3)));
        assert!(combfilt.check(&Number(9)));
        assert!(!combfilt.check(&Number(2)));
    }
}
