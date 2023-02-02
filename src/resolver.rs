use crate::environment::Environment;
use crate::ir::NumVal::{Number, Unsure};
use crate::ir::*;
use anyhow::{anyhow, Result};
use chrono::Local;
use chrono::{prelude as cr, Datelike, Timelike};
use std::time::SystemTime;

// TODO: Change all resolve to Result<> based

pub fn resolve(records: Vec<Record>, created: SystemTime) -> Vec<ExactRecord> {
    let base_time: cr::DateTime<Local> = created.into();
    let base_t = ExactDateTime {
        date: {
            let date = base_time.date_naive();
            ExactDate {
                year: date.year(),
                month: date.month(),
                day: date.day(),
            }
        },
        time: {
            ExactTime {
                hour: 0,
                minute: 0,
                second: 0,
            }
        },
        tz: TimeZoneChoice::Local,
    };
    let mut base = Environment {
        current: DateTime {
            date: Some(Date {
                year: Number(base_t.date.year as i64),
                month: Unsure,
                day: Unsure,
            }),
            time: None,
            tz: None,
        },
        date_time: base_t,
        parent: None,
    };

    let mut resolved = vec![];
    for record in records {
        match record {
            Record::Event(event) => {
                let event = resolve_event(&event, &base);
                match event {
                    Ok(event) => resolved.push(ExactRecord::Event(event)),
                    Err(e) => eprintln!("Error resolving event: {}", e),
                }
            }
            Record::Occasion(occasion) => {
                let fixed_occasion = resolve_occasion(&occasion, &base);
                // PERFORMANCE: update base inplace
                match fixed_occasion {
                    Ok(o) => {
                        base = Environment {
                            date_time: o,
                            parent: Some(Box::new(base)),
                            current: occasion,
                        };
                    }
                    Err(e) => eprintln!("Error resolving occasion: {}", e),
                }
            }
            Record::Note(note) => {
                resolved.push(ExactRecord::Note(note.to_string()));
            }
            Record::FlexOccasion(occasion) => {
                // Filters
                eprintln!("{:?}", occasion);
                todo!()
            }
            Record::FlexEvents(flex_events) => {
                match &flex_events.occasion{
                    FlexOccasion::Filter(filter) => {
                        for date in base.clone().into_iter(){
                            let mut tmp_env = base.clone();
                            tmp_env.current = DateTime{
                                date: Some(date),
                                tz: None,
                                time: None
                            };
                            tmp_env.date_time.date = resolve_date(&date, &base).unwrap();
                            tmp_env.parent = Some(Box::new(base.clone()));
                            if filter.check(&date, Some(&base)){
                                for event in &flex_events.events{
                                    if let Ok(res) = resolve_event(event, &tmp_env){
                                        resolved.push(ExactRecord::Event(res));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    resolved
}

// Should it really be named occasion... perhaps rename it to resolve_datetime?
pub fn resolve_occasion(occasion: &DateTime, base: &Environment) -> Result<ExactDateTime> {
    Ok(ExactDateTime {
        date: if let Some(date) = &occasion.date {
            resolve_date(date, &base)?
        } else {
            base.date_time.date
        },
        time: if let Some(time) = &occasion.time {
            resolve_time(time, &base)?
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
            let date = resolve_date(date, &base)?;
            ExactRange::AllDay(date)
        }
        Range::TimeRange(time_range) => {
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
