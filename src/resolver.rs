use crate::ir::*;
use std::time::SystemTime;
use anyhow::{Result, anyhow};
use chrono::{Datelike, prelude as cr};
use chrono::Local;
use crate::ir::NumVal::Number;

// TODO: Change all resolve to Result<> based

pub fn resolve(records: Vec<Record>, created: SystemTime) -> Vec<ExactRecord> {
    let base_time:cr::DateTime<Local> = created.into();
    let mut base  = ExactDateTime {
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
                let occasion = resolve_occasion(&occasion, &base);
                // PERFORMANCE: update base inplace
                match occasion {
                    Ok(occasion) => {
                        base = occasion;
                    },
                    Err(e) => eprintln!("Error resolving occasion: {}", e),
                }
            }
            Record::Note(note) => {
                resolved.push(ExactRecord::Note(note.to_string()));
            }
        }
    }
    resolved
}

// Should it really be named occasion... perhaps rename it to resolve_datetime?
fn resolve_occasion(occasion: &DateTime, base:&ExactDateTime) -> Result<ExactDateTime> {
    Ok(ExactDateTime {
        date: if let Some(date) = &occasion.date {
            resolve_date(date, &base.date)?
        } else {
            base.date
        },
        time: if let Some(time) = &occasion.time {
            resolve_time(time, &base.time)?
        } else {
            base.time
        },
        tz: base.tz,
    })
}

fn resolve_time(time: &Time, base: &ExactTime) -> Result<ExactTime> {
    Ok(ExactTime {
        hour: match time.hour {
            Number(n) => (match &time.tod {
                Some(tod) => {
                    if n>12 {
                        return Err(anyhow!("Hour value cannot exceed 12 when AM/PM is specified(found: {})", n));
                    }
                    if n==12 {
                        match tod {
                            Tod::AM => 0,
                            Tod::PM => 12,
                        }
                    } else {
                        match tod {
                            Tod::AM => n,
                            Tod::PM => n+12,
                        }
                    }
                },
                None => {
                    if n>23 {
                        return Err(anyhow!("Hour value cannot exceed 23(found: {})", n));
                    }
                    n
                },
            }) as u32,
            _ => base.hour,
        },
        minute: match time.minute {
            Number(n) => {
                if n>59 {
                    return Err(anyhow!("Minute value cannot exceed 59(found: {})", n));
                }
                n as u32
            },
            _ => base.minute,
        },
        second: match time.second {
            Number(n) => {
                if n>59 {
                    return Err(anyhow!("Second value cannot exceed 59(found: {})", n));
                }
                n as u32
            },
            _ => base.second,
        },
    })
}

fn resolve_date(date: &Date, base: &ExactDate) -> Result<ExactDate> {
    let res = ExactDate {
        year: match date.year {
            Number(n) => n as i32,
            _ => base.year,
        },
        month: match date.month {
            Number(n) => n as u32,
            _ => base.month,
        },
        day: match date.day {
            Number(n) => n as u32,
            _ => base.day,
        },
    };
    Ok(res)
}


fn resolve_event(event: &Event, base: &ExactDateTime) -> Result<ExactEvent> {
    Ok(ExactEvent {
        range: resolve_range(&event.range, base)?,
        name: event.name.clone(),
        notes: event.notes.clone(),
    })
}

fn resolve_range(range: &Range, base: &ExactDateTime) -> Result<ExactRange> {
    Ok(match range {
        Range::AllDay(date) => {
            let date = resolve_date(date, &base.date)?;
            ExactRange::AllDay(date)
        }
        Range::TimeRange(time_range) => {
            let start = resolve_occasion(&time_range.start, base)?;
            let end = resolve_occasion(&time_range.end, base)?;
            ExactRange::TimeRange(ExactTimeRange { start, end })
        }
        Range::Duration(duration) => {
            ExactRange::Duration(resolve_duration(duration, base)?)
        }
    })
}

fn resolve_duration(duration: &Duration, base: &ExactDateTime) -> Result<ExactDuration> {
    let start = resolve_occasion(&duration.start, base)?;
    let dur = match duration.duration {
        Number(n) => {
            if n<0 { return Err(anyhow!("Duration cannot be negative")) }
            n as u64
        }
        _ => 30,
    };
    Ok(ExactDuration {
        start,
        duration: dur,
    })
}