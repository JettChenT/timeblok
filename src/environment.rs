use std::collections::HashMap;
use std::rc::Rc;
use crate::ir::filter::{BDF, Filter};
use crate::ir::NumVal::Number;
use crate::ir::{
    Date, DateTime, ExactDate, ExactDateTime, FlexDate, FlexField, NumVal, Time, TimeZoneChoice, ident::Ident
};
use crate::resolver::{resolve_date, resolve_time};
use chrono::NaiveDate;
use std::thread::current;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub struct Environment {
    pub date_time: ExactDateTime,
    pub parent: Option<Rc<Environment>>,
    pub current: DateTime,
}

pub struct EnvIterator<'a> {
    env: &'a Environment,
    cur: Date,
    cur_date: NaiveDate,
    filter: BDF<ExactDate>,
    fit_date: Date,
}

impl Environment {
    pub fn new(dt: ExactDateTime) -> Self{
        Environment{
            current: DateTime::from_exact(&dt),
            date_time: dt,
            parent: None,
        }
    }
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

impl Iterator for EnvIterator<'_> {
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

impl Environment{
    pub fn iter(&self) -> EnvIterator {
        let fit_date = max_fit_date(&self).unwrap();
        let filter = Box::new(FlexDate {
            day: Box::new(FlexField::NumVal(fit_date.day)) as BDF<NumVal>,
            month: Box::new(FlexField::NumVal(fit_date.month)) as BDF<NumVal>,
            year: Box::new( FlexField::NumVal(fit_date.year)) as BDF<NumVal>,
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

mod tests{
    use crate::ir::ExactTime;
    use crate::ir::NumVal::Unsure;
    use super::*;
    #[test]
    fn test_env(){
        use super::*;
        let env = Environment{
            date_time: ExactDateTime{
                date: ExactDate{
                    year: 2023,
                    month: 1,
                    day: 1,
                },
                time: ExactTime{
                    hour: 1,
                    minute: 1,
                    second: 1,
                },
                tz: TimeZoneChoice::Utc,
            },
            parent: None,
            current: DateTime{
                date: Some(Date{
                    year: Number(2023),
                    month: Number(1),
                    day: Unsure,
                }),
                time: Some(Time{
                    hour: Number(0),
                    minute: Number(0),
                    second: Number(0),
                    tod: None,
                }),
                tz: None,
            },
        };
        let mut daynum= 1;
        for date in env.iter(){
            assert_eq!(date, Date{
                year: Number(2023),
                month: Number(1),
                day: Number(daynum),
            });
            daynum+=1;
        }
    }
}
