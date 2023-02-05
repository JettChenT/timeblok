use crate::ir::ident::{DynFilter, IdentData};
use anyhow::Result;
use chrono::{Datelike, Weekday};
use crate::environment::Environment;
use crate::ir::{Date, ExactDate};
use crate::resolver::resolve_date;

impl ExactDate{
    fn weekday(&self) -> Result<Weekday>{
        Ok(self.to_chrono()?.weekday())
    }
}

fn insert_holidays(env: &mut Environment) -> Result<()>{
    todo!()
}

fn insert_weekdays(env: &mut Environment) -> Result<()>{
    let weekdays = vec![
        "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
        "mon", "tue", "wed", "thu", "fri", "sat", "sun",
    ];
    for w in weekdays{
        let wkday = w.parse::<Weekday>().unwrap();
        let filt = DynFilter {
            filter: Box::new(move |d:&Date, env:Option<&Environment>| {
                resolve_date(d, env.unwrap()).unwrap().weekday().unwrap() == wkday
            }),
            name: w.to_string(),
        };
        env.set(w, IdentData::DateFilter(Box::new(filt)))?;
    }
    // Insert workday and weekend
    let workday = DynFilter {
        filter: Box::new(move |d:&Date, env:Option<&Environment>| {
            let wkday = resolve_date(d, env.unwrap()).unwrap().weekday().unwrap();
            wkday != Weekday::Sat && wkday != Weekday::Sun
        }),
        name: "workday".to_string(),
    };
    env.set("workday", IdentData::DateFilter(Box::new(workday)))?;
    let weekend = DynFilter {
        filter: Box::new(move |d:&Date, env:Option<&Environment>| {
            let wkday = resolve_date(d, env.unwrap()).unwrap().weekday().unwrap();
            wkday == Weekday::Sat || wkday == Weekday::Sun
        }),
        name: "weekend".to_string(),
    };
    env.set("weekend", IdentData::DateFilter(Box::new(weekend)))?;
    Ok(())
}

pub fn insert_preset(env: &mut Environment) -> Result<()>{
    // inserting weekdays
    insert_weekdays(env)?;
    Ok(())
}