use std::thread::current;
use crate::ir::{Date, DateTime, ExactDateTime, NumVal};
use crate::ir::NumVal::Number;

pub struct Environment{
    pub date_time: ExactDateTime,
    pub parent: Option<Box<Environment>>,
    pub current: DateTime,
}

pub struct EnvIterator<'a>{
    env: &'a Environment,
    cur: ExactDateTime
}

impl Environment {
    fn get_loc(&self, i: usize) -> Option<NumVal>{
        match i {
            0 => Some(self.current.date?.year),
            1 => Some(self.current.date?.month),
            2 => Some(self.current.date?.day),
            3 => Some(self.current.time?.hour),
            4 => Some(self.current.time?.minute),
            5 => Some(self.current.time?.second),
            _ => None
        }
    }
    pub fn get(&self, i: usize) -> Option<NumVal>{
        match self.get_loc(i) {
            Some(v) => Some(v),
            None => match &self.parent {
                Some(p) => p.get(i),
                None => None
            }
        }
    }
}

fn max_fit_date(env: &Environment) -> Option<Date>{
    let date = &env.current.date;
    match date {
        None => {return max_fit_date(env.parent.as_ref().unwrap());}
        Some(date) => {
            let mut cur = env.date_time.clone();
            if let Number(year) = date.year {
                cur.date.year = year as i32;
            }
            if let Number(month) = date.month {
                cur.date.month = month as u32;
            }
            if let Number(day) = date.day {
                cur.date.day = day as u32;
            }
            DateTime::from_exact(cur).date
        }
    }
}