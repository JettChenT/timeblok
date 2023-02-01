use crate::filter::{BDF, Filter};
use crate::ir::NumVal::Number;
use crate::ir::{
    Date, DateTime, ExactDate, ExactDateTime, FlexDate, FlexField, NumVal, Time, TimeZoneChoice,
};
use crate::resolver::{resolve_date, resolve_time};
use chrono::NaiveDate;
use std::thread::current;
use std::vec::IntoIter;

pub struct Environment {
    pub date_time: ExactDateTime,
    pub parent: Option<Box<Environment>>,
    pub current: DateTime,
}

pub struct EnvIterator {
    env: Environment,
    cur: Date,
    cur_date: NaiveDate,
    filter: BDF<ExactDate>,
    fit_date: Date,
}

impl Environment {
    fn get_loc(&self, i: usize) -> Option<NumVal> {
        match i {
            0 => Some(self.current.date?.year),
            1 => Some(self.current.date?.month),
            2 => Some(self.current.date?.day),
            3 => Some(self.current.time?.hour),
            4 => Some(self.current.time?.minute),
            5 => Some(self.current.time?.second),
            _ => None,
        }
    }
    pub fn get(&self, i: usize) -> Option<NumVal> {
        match self.get_loc(i) {
            Some(v) => Some(v),
            None => match &self.parent {
                Some(p) => p.get(i),
                None => None,
            },
        }
    }
}

fn max_fit_date(env: &Environment) -> Option<Date> {
    let date = &env.current.date;
    match date {
        None => max_fit_date(env.parent.as_ref().unwrap()),
        Some(date) => {
            let mut ndate = Date::new();
            if let Number(year) = date.year {
                ndate.year = Number(year);
            } else {
                return Some(ndate);
            }
            if let Number(month) = date.month {
                ndate.month = Number(month);
            } else {
                return Some(ndate);
            }
            if let Number(day) = date.day {
                ndate.day = Number(day);
            } else {
                return Some(ndate);
            }
            Some(ndate)
        }
    }
}

impl Iterator for EnvIterator {
    type Item = Date;
    fn next(&mut self) -> Option<Self::Item> {
        // This conveniently assumes dates are continuous, don't use for non-continuous filters
        let cur_date = Date::from_naive(self.cur_date);
        if !self.filter.check(&(resolve_date(&cur_date, &self.env).ok())?, Some(&self.env)) {
            return None;
        }
        let new_date = self.cur_date + chrono::Duration::days(1);
        self.cur_date = new_date;
        Some(cur_date)
    }
}

impl IntoIterator for Environment {
    type Item = Date;
    type IntoIter = EnvIterator;
    fn into_iter(self) -> Self::IntoIter {
        let fit_date = max_fit_date(&self).unwrap();
        let filter = Box::new(FlexDate {
            day: FlexField::NumVal(fit_date.day),
            month: FlexField::NumVal(fit_date.month),
            year: FlexField::NumVal(fit_date.year),
        });
        let filldat = |n:NumVal| match n {
            Number(n) => n,
            _ => 1
        };
        let cur_date = NaiveDate::from_ymd_opt(
            filldat(fit_date.year) as i32,
            filldat(fit_date.month) as u32,
            filldat(fit_date.day) as u32,
        ).unwrap();
        EnvIterator {
            env: self,
            cur: fit_date,
            cur_date,
            filter: filter as BDF<ExactDate>,
            fit_date,
        }
    }
}
