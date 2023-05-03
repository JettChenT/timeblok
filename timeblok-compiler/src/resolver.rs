use crate::environment::Environment;
use crate::ir::NumVal::{Number, Unsure};
use crate::ir::*;
use crate::preset::insert_preset;
use anyhow::{anyhow, Result};
use chrono::Local;
use chrono::{prelude as cr, Datelike, Timelike};
use std::rc::Rc;
use std::time::SystemTime;
use crate::ir::ident::{Ident, IdentData};
use crate::ir::Todo;

// TODO: Change all resolve to Result<> based

pub enum ResolverAction {
    Set(Ident, IdentData),
    SetTimeZone(TimeZoneChoice),
    InsertRecord(ExactRecord),
    InsertTodo(Todo),
    InsertRecords(Vec<ExactRecord>)
}

pub fn resolve(records: Vec<Record>, base_t: ExactDateTime) -> Vec<ExactRecord> {

    let mut base = {
        let tyear = base_t.date.year;
        Environment::new(
            base_t,
            DateTime {
                date: Some(Date {
                    year: Number(tyear as i64),
                    month: Unsure,
                    day: Unsure,
                }),
                time: None,
                tz: None,
            },
            None,
        )
    };

    insert_preset(&mut base).unwrap();

    let mut resolved = vec![];
    let mut baseref = Rc::new(base);
    for record in records {
        match record {
            Record::Event(event) => {
                let event = resolve_event(&event, &baseref);
                match event {
                    Ok(event) => resolved.push(ExactRecord::Event(event)),
                    Err(e) => eprintln!("Error resolving event: {}", e),
                }
            }
            Record::Occasion(occasion) => {
                let fixed_occasion = resolve_occasion(&occasion, &baseref);
                // PERFORMANCE: update base inplace
                match fixed_occasion {
                    Ok(o) => {
                        base = Environment::new(o, occasion, Some(Rc::clone(&baseref)));
                        baseref = Rc::new(base);
                    }
                    Err(e) => eprintln!("Error resolving occasion: {}", e),
                }
            }
            Record::Note(note) => {
                resolved.push(ExactRecord::Note(note.to_string()));
            }
            Record::Command(cmd) => {
                match cmd.run(baseref.as_ref()) {
                    Err(e) => {eprintln!("Error when resolving Command: {:?}", e);}
                    Ok(Some(cmds)) => {
                        for c in cmds {
                            match c {
                                ResolverAction::Set(ident, data) => {baseref.as_ref().set(ident.name.as_str(), data).unwrap();},
                                ResolverAction::InsertRecord(rec) => {resolved.push(rec);},
                                ResolverAction::InsertRecords(recs) => {resolved.extend(recs);}
                                ResolverAction::InsertTodo(t) => {resolved.push(ExactRecord::Todo(t));}
                                ResolverAction::SetTimeZone(tz) => {
                                    let nbase = Environment::new(
                                        ExactDateTime {
                                            tz,
                                            ..baseref.as_ref().date_time
                                        },
                                        baseref.as_ref().current.clone(),
                                        Some(Rc::clone(&baseref)));
                                    baseref = Rc::new(nbase);
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                }
            }
            Record::FlexOccasion(occasion) => {
                // Filters
                eprintln!("{:?}", occasion);
                todo!()
            }
            Record::FlexEvents(flex_events) => match &flex_events.occasion {
                FlexOccasion::Filter(filter) => {
                    for date in (*baseref).iter() {
                        let tmp_env = Environment::new(
                            ExactDateTime {
                                date: resolve_date(&date, &baseref).unwrap(),
                                time: ExactTime {
                                    hour: 0,
                                    minute: 0,
                                    second: 0,
                                },
                                tz: TimeZoneChoice::Local,
                            },
                            DateTime {
                                date: Some(date),
                                time: None,
                                tz: None,
                            },
                            Some(Rc::clone(&baseref)),
                        );
                        if filter.check(&date, Some(&(*baseref))) {
                            for event in &flex_events.events {
                                if let Ok(res) = resolve_event(event, &tmp_env) {
                                    resolved.push(ExactRecord::Event(res));
                                }
                            }
                        }
                    }
                }
            },
        }
    }
    resolved
}

// Should it really be named occasion... perhaps rename it to resolve_datetime?
pub fn resolve_occasion(occasion: &DateTime, base: &Environment) -> Result<ExactDateTime> {
    Ok(ExactDateTime {
        date: if let Some(date) = &occasion.date {
            resolve_date(date, base)?
        } else {
            base.date_time.date
        },
        time: if let Some(time) = &occasion.time {
            resolve_time(time, base)?
        } else {
            base.date_time.time
        },
        tz: base.date_time.tz,
    })
}

pub fn resolve_time(time: &Time, base: &Environment) -> Result<ExactTime> {
    let base_time = base.date_time.time;
    Ok(ExactTime {
        hour: match time.hour {
            Number(n) => {
                (match &time.tod {
                    Some(tod) => {
                        if n > 12 {
                            return Err(anyhow!(
                                "Hour value cannot exceed 12 when AM/PM is specified(found: {})",
                                n
                            ));
                        }
                        if n == 12 {
                            match tod {
                                Tod::AM => 0,
                                Tod::PM => 12,
                            }
                        } else {
                            match tod {
                                Tod::AM => n,
                                Tod::PM => n + 12,
                            }
                        }
                    }
                    None => {
                        if n > 23 {
                            return Err(anyhow!("Hour value cannot exceed 23(found: {})", n));
                        }
                        n
                    }
                }) as u32
            }
            _ => base_time.hour,
        },
        minute: match time.minute {
            Number(n) => {
                if n > 59 {
                    return Err(anyhow!("Minute value cannot exceed 59(found: {})", n));
                }
                n as u32
            }
            _ => base_time.minute,
        },
        second: match time.second {
            Number(n) => {
                if n > 59 {
                    return Err(anyhow!("Second value cannot exceed 59(found: {})", n));
                }
                n as u32
            }
            _ => base_time.second,
        },
    })
}

pub fn resolve_date(date: &Date, base: &Environment) -> Result<ExactDate> {
    let base_date = base.date_time.date;
    let res = ExactDate {
        year: match date.year {
            Number(n) => n as i32,
            _ => base_date.year,
        },
        month: match date.month {
            Number(n) => n as u32,
            _ => base_date.month,
        },
        day: match date.day {
            Number(n) => n as u32,
            _ => base_date.day,
        },
    };
    Ok(res)
}

pub fn resolve_event(event: &Event, base: &Environment) -> Result<ExactEvent> {
    Ok(ExactEvent {
        range: resolve_range(&event.range, base)?,
        name: event.name.clone(),
        notes: event.notes.clone(),
    })
}

pub fn resolve_range(range: &Range, base: &Environment) -> Result<ExactRange> {
    Ok(match range {
        Range::AllDay(date) => {
            let date = resolve_date(date, base)?;
            ExactRange::AllDay(date)
        }
        Range::Time(time_range) => {
            let start = resolve_occasion(&time_range.start, base)?;
            let end = resolve_occasion(&time_range.end, base)?;
            ExactRange::TimeRange(ExactTimeRange { start, end })
        }
        Range::Duration(duration) => {
            // ExactRange::Duration(resolve_duration(duration, base)?)
            let start = resolve_occasion(&duration.start, base)?;
            let shift = chrono::Duration::minutes(match duration.duration {
                Number(n) => n,
                _ => 30,
            });
            let end_ch = start.to_chrono()? + shift;
            let end = ExactDateTime::from_chrono(end_ch);
            ExactRange::TimeRange(ExactTimeRange { start, end })
        }
    })
}

pub fn resolve_duration(duration: &Duration, base: &Environment) -> Result<ExactDuration> {
    let start = resolve_occasion(&duration.start, base)?;
    let dur = match duration.duration {
        Number(n) => {
            if n < 0 {
                return Err(anyhow!("Duration cannot be negative"));
            }
            n as u64
        }
        _ => 30,
    };
    Ok(ExactDuration {
        start,
        duration: dur,
    })
}
