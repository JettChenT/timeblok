mod workalendar;

use std::rc::Rc;

use crate::ir::command::Command;
use crate::ir::ident::{DynFilter, IdentData};
use anyhow::{Result, anyhow};
use chrono::{Datelike, Weekday};
use crate::environment::Environment;
use crate::ir::{Date, ExactDate, Value};
use crate::resolver::resolve_date;

impl ExactDate{
    fn weekday(&self) -> Result<Weekday>{
        Ok(self.to_chrono()?.weekday())
    }
}

fn insert_holidays(env: &mut Environment) -> Result<()>{
    todo!()
}

fn insert_commands(env: &mut Environment) -> Result<()>{
    env.set("print", IdentData::Command(Command{
        name: "print".to_string(),
        arity: 1,
        func: Rc::new(|env: &Environment, args: &[Value]| {
            if let Value::Ident(ident) = &args[0] {
                if let Some(dat) = env.get(&ident.name) {
                    println!("{} : {:?}", &ident.name, dat);
                    Ok(())
                }else{
                    Err(anyhow!(format!("Identity {} not found", &ident.name)))
                }
            }else{
                Err(anyhow!(format!("The argument is not an identity.")))
            }
        }
        )
    }))?;
    env.set("set", IdentData::Command(Command{
        name: "set".to_string(),
        arity: 2,
        func: Rc::new(|env: &Environment, args: &[Value]| {
            if let Value::Ident(ident) = &args[0]{
                env.set(&ident.name, IdentData::Value(args[1].clone()))?;
                Ok(())
            }else{
                Err(anyhow!("First argument for /set must be an identity."))
            }
        }) 
    }))?;
    Ok(())
}

fn insert_weekdays(env: &mut Environment) -> Result<()>{
    use Value::DateFilter;
    let weekdays = vec![
        "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
        "mon", "tue", "wed", "thu", "fri", "sat", "sun",
    ];
    for w in weekdays{
        let wkday = w.parse::<Weekday>().unwrap();
        let filt = DynFilter {
            filter: Rc::new(move |d:&Date, env:Option<&Environment>| {
                resolve_date(d, env.unwrap()).unwrap().weekday().unwrap() == wkday
            }),
            name: w.to_string(),
        };
        env.set(w, IdentData::Value(DateFilter(Box::new(filt))))?;
    }
    // Insert workday and weekend
    let workday = DynFilter {
        filter: Rc::new(move |d:&Date, env:Option<&Environment>| {
            let wkday = resolve_date(d, env.unwrap()).unwrap().weekday().unwrap();
            wkday != Weekday::Sat && wkday != Weekday::Sun
        }),
        name: "workday".to_string(),
    };
    env.set("workday", IdentData::Value(DateFilter(Box::new(workday))))?;
    let weekend = DynFilter {
        filter: Rc::new(move |d:&Date, env:Option<&Environment>| {
            let wkday = resolve_date(d, env.unwrap()).unwrap().weekday().unwrap();
            wkday == Weekday::Sat || wkday == Weekday::Sun
        }),
        name: "weekend".to_string(),
    };
    env.set("weekend", IdentData::Value(DateFilter(Box::new(weekend))))?;
    Ok(())
}

pub fn insert_preset(env: &mut Environment) -> Result<()>{
    // inserting weekdays
    insert_weekdays(env)?;
    insert_commands(env)?;
    Ok(())
}