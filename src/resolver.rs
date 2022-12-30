use crate::ir::*;
use std::time::SystemTime;
use chrono::{Datelike, prelude as cr, Timelike};
use chrono::{TimeZone,Local, Utc};
use crate::ir::NumVal::Number;

pub fn resolve(records: Vec<Box<Record>>, created: SystemTime) -> Vec<Box<ExactRecord>> {
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
        match record.as_ref() {
            Record::Event(event) => {
                let event = resolve_event(event, &base);
                resolved.push(Box::new(ExactRecord::Event(event)));
            }
            Record::Occasion(occasion) => {
                let occasion = resolve_occasion(occasion, &base);
                // PERFORMANCE: update base inplace
                base = occasion;
            }
            Record::Note(note) => {
                resolved.push(Box::new(ExactRecord::Note(note.to_string())));
            }
        }
    }
    resolved
}

// Should it really be named occasion... perhaps rename it to resolve_datetime?
fn resolve_occasion(occasion: &DateTime, base:&ExactDateTime) -> ExactDateTime {
    ExactDateTime{
        date: if let Some(date) = &occasion.date {
            resolve_date(date, &base.date)
        } else {
            base.date.clone()
        },
        time: if let Some(time) = &occasion.time {
            resolve_time(time, &base.time)
        } else {
            base.time.clone()
        },
        tz: base.tz,
    }
}

fn resolve_time(time: &Time, base: &ExactTime) -> ExactTime {
    ExactTime{
        hour: match time.hour {
            Number(n) => n as u32,
            _ => base.hour,
        },
        minute: match time.minute {
            Number(n) => n as u32,
            _ => base.minute,
        },
        second: match time.second {
            Number(n) => n as u32,
            _ => base.second,
        },
    }
}

fn resolve_date(date: &Date, base: &ExactDate) -> ExactDate {
    ExactDate{
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
    }
}


fn resolve_event(event: &Event, base: &ExactDateTime) -> ExactEvent {
    ExactEvent{
        range: resolve_range(&event.range, base),
        name: event.name.clone(),
        notes: event.notes.clone(),
    }
}

fn resolve_range(range: &Range, base: &ExactDateTime) -> ExactRange {
    match range {
        Range::AllDay(date) => {
            let date = resolve_date(date, &base.date);
            ExactRange::AllDay(date)
        },
        Range::TimeRange(time_range) => {
            let start = resolve_occasion(&time_range.start, &base);
            let end = resolve_occasion(&time_range.end, &base);
            ExactRange::TimeRange(ExactTimeRange{start, end})
        },
        Range::Duration(duration) => {
            ExactRange::Duration(resolve_duration(duration, &base))
        },
    }
}

fn resolve_duration(duration: &Duration, base: &ExactDateTime) -> ExactDuration {
    let start = resolve_occasion(&duration.start, &base);
    let dur = match duration.duration {
        Number(n) => {
            if n<0 { panic!("Duration cannot be negative") }
            n as u64
        }
        _ => 30,
    };
    ExactDuration{
        start,
        duration: dur,
    }
}