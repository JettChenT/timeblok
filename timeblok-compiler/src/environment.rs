use crate::ir::filter::{Filter, BDF};
use crate::ir::NumVal::Number;
use crate::ir::{
    ident::IdentData, Date, DateTime, ExactDate, ExactDateTime, FlexDate, FlexField, NumVal,
};
use crate::resolver::resolve_date;
use anyhow::Result;
use chrono::NaiveDate;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    pub date_time: ExactDateTime,
    pub parent: Option<Rc<Environment>>,
    pub current: DateTime,
    pub namespace: RefCell<HashMap<String, IdentData>>,
}

pub struct EnvIterator<'a> {
    env: &'a Environment,
    cur: Date,
    cur_date: NaiveDate,
    filter: BDF<ExactDate>,
    fit_date: Date,
}

impl Environment {
    pub fn new(
        date_time: ExactDateTime,
        current: DateTime,
        parent: Option<Rc<Environment>>,
    ) -> Self {
        Environment {
            date_time,
            parent,
            current,
            namespace: RefCell::new(HashMap::new()),
        }
    }
    pub fn from_exact(dt: ExactDateTime) -> Self {
        let cur = DateTime::from_exact(&dt);
        Environment::new(dt, cur, None)
    }

    pub fn get(&self, name: &str) -> Option<IdentData> {
        match self.namespace.borrow().get(name) {
            Some(ident) => Some((*ident).clone()),
            None => match &self.parent {
                Some(parent) => parent.get(name),
                None => None,
            },
        }
    }

    pub fn set(&self, name: &str, ident: IdentData) -> Result<()> {
        self.namespace.borrow_mut().insert(name.to_string(), ident);
        Ok(())
    }

    pub fn del(&self, name: &str) -> Result<()> {
        self.namespace.borrow_mut().remove(name);
        Ok(())
    }
}

impl Environment {
    // might come in handy later
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
    fn get_recurse(&self, i: usize) -> Option<NumVal> {
        match self.get_loc(i) {
            Some(v) => Some(v),
            None => match &self.parent {
                Some(p) => p.get_recurse(i),
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

impl Iterator for EnvIterator<'_> {
    type Item = Date;
    fn next(&mut self) -> Option<Self::Item> {
        // This conveniently assumes dates are continuous, don't use for non-continuous filters
        let cur_date = Date::from_naive(self.cur_date);
        if !self
            .filter
            .check(&(resolve_date(&cur_date, self.env).ok())?, Some(self.env))
        {
            return None;
        }
        let new_date = self.cur_date + chrono::Duration::days(1);
        self.cur_date = new_date;
        Some(cur_date)
    }
}

impl Environment {
    pub fn iter(&self) -> EnvIterator {
        let fit_date = max_fit_date(self).unwrap();
        let filter = Box::new(FlexDate {
            day: Box::new(FlexField::NumVal(fit_date.day)) as BDF<NumVal>,
            month: Box::new(FlexField::NumVal(fit_date.month)) as BDF<NumVal>,
            year: Box::new(FlexField::NumVal(fit_date.year)) as BDF<NumVal>,
        });
        let filldat = |n: NumVal| match n {
            Number(n) => n,
            _ => 1,
        };
        let cur_date = NaiveDate::from_ymd_opt(
            filldat(fit_date.year) as i32,
            filldat(fit_date.month) as u32,
            filldat(fit_date.day) as u32,
        )
        .unwrap();
        EnvIterator {
            env: self,
            cur: fit_date,
            cur_date,
            filter: filter as BDF<ExactDate>,
            fit_date,
        }
    }
}

mod tests {

    #[test]
    fn test_env() {
        use super::*;
        let env = Environment::from_exact(ExactDateTime::from_ymd_hms(2023, 1, 1, 1, 1, 1));
        let mut daynum = 1;
        for date in env.iter() {
            assert_eq!(
                date,
                Date {
                    year: Number(2023),
                    month: Number(1),
                    day: Number(daynum),
                }
            );
            daynum += 1;
        }
    }
}
