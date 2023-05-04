use crate::ir::filter::{Filter, BDF};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, NaiveWeek, Timelike, Utc};
use icalendar::{CalendarDateTime, DatePerhapsTime, Component};
use anyhow::Result;

use self::{command::CommandCall, ident::Ident};

pub mod command;
pub mod filter;
pub mod ident;
pub mod displays;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum NumVal {
    Number(i64),
    Unsure,
}

#[derive(Debug, Clone)]
pub struct NumRange {
    pub start: NumVal,
    pub end: NumVal,
}

#[derive(Debug, Clone)]
pub enum Range {
    Time(TimeRange),
    Duration(Duration),
    AllDay(Date),
}

#[derive(Debug, Clone)]
pub enum ExactRange {
    TimeRange(ExactTimeRange),
    AllDay(ExactDate),
}

#[derive(Debug)]
pub struct ExactDuration {
    pub start: ExactDateTime,
    pub duration: u64,
}

#[derive(Debug, Clone)]
pub struct Duration {
    pub start: DateTime,
    pub duration: NumVal,
}

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Debug, Clone)]
pub struct ExactTimeRange {
    pub start: ExactDateTime,
    pub end: ExactDateTime,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeZoneChoice {
    Local,
    Utc,
    Offset(chrono::offset::FixedOffset)
}

#[derive(Default, Debug, Clone)]
pub struct DateTime {
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub tz: Option<TimeZoneChoice>,
}

#[derive(Debug, Clone)]
pub struct ExactDateTime {
    pub date: ExactDate,
    pub time: ExactTime,
    pub tz: TimeZoneChoice,
}

#[derive(Debug, Clone)]
pub enum FlexField {
    NumVal(NumVal),
    NumRange(NumRange),
}

#[derive(Debug, Clone)]
pub struct FlexDate {
    pub year: BDF<NumVal>,
    pub month: BDF<NumVal>,
    pub day: BDF<NumVal>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Date {
    pub year: NumVal,
    pub month: NumVal,
    pub day: NumVal,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ExactDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum Tod {
    // Add support for resolving
    AM,
    PM,
}

#[derive(Debug, Copy, Clone)]
pub struct Time {
    pub hour: NumVal,
    pub minute: NumVal,
    pub second: NumVal,
    pub tod: Option<Tod>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExactTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

// PERFORMANCE: Figure out how to use &str instead of String
#[derive(Debug)]
pub struct Event {
    pub range: Range,
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct ExactEvent {
    pub range: ExactRange,
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub enum Record {
    Event(Event),
    Occasion(DateTime),
    Note(String),
    FlexOccasion(FlexOccasion),
    FlexEvents(FlexEvents),
    Command(CommandCall),
}

#[derive(Debug)]
pub enum ExactRecord {
    Event(ExactEvent),
    Note(String),
    Todo(Todo)
}

#[derive(Debug)]
pub enum FlexOccasion {
    Filter(Box<dyn Filter<Date>>),
}

#[derive(Debug)]
pub struct FlexEvents {
    pub occasion: FlexOccasion,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Num(NumVal),
    DateFilter(BDF<Date>),
    NumFilter(BDF<NumVal>),
    Date(Date),
    Ident(Ident),
    String(String),
}

impl DateTime {
    pub fn from_exact(exact: &ExactDateTime) -> Self {
        DateTime {
            date: Some(Date {
                year: NumVal::Number(exact.date.year as i64),
                month: NumVal::Number(exact.date.month as i64),
                day: NumVal::Number(exact.date.day as i64),
            }),
            time: Some(Time {
                hour: NumVal::Number(exact.time.hour as i64),
                minute: NumVal::Number(exact.time.minute as i64),
                second: NumVal::Number(exact.time.second as i64),
                tod: None,
            }),
            tz: Some(exact.tz),
        }
    }

    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        DateTime {
            date: Some(Date {
                year: NumVal::Number(year as i64),
                month: NumVal::Number(month as i64),
                day: NumVal::Number(day as i64),
            }),
            time: Some(Time {
                hour: NumVal::Number(hour as i64),
                minute: NumVal::Number(minute as i64),
                second: NumVal::Number(second as i64),
                tod: None,
            }),
            tz: Some(TimeZoneChoice::Local),
        }
    }
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        DateTime {
            date: Some(Date {
                year: NumVal::Number(year as i64),
                month: NumVal::Number(month as i64),
                day: NumVal::Number(day as i64),
            }),
            time: None,
            tz: Some(TimeZoneChoice::Local),
        }
    }
}

impl ExactTime{
    pub fn from_naive(naive: NaiveTime) -> Self{
        Self{
            hour: naive.hour(),
            minute: naive.minute(),
            second: naive.second()
        }
    }

    pub fn from_hms(hour: u32, minute: u32, second: u32) -> Self {
        Self {
            hour, minute, second
        }
    }
}

impl ExactDateTime {
    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        ExactDateTime {
            date: ExactDate { year, month, day },
            time: ExactTime {
                hour,
                minute,
                second,
            },
            tz: TimeZoneChoice::Local,
        }
    }

    pub fn from_naive(naive: NaiveDateTime) -> Self{
        Self{
            date: ExactDate::from_naive(naive.date()),
            time: ExactTime::from_naive(naive.time()),
            tz: TimeZoneChoice::Local
        }
    }

    pub fn from_date_perhaps_time(d: DatePerhapsTime) -> Self{
        match d{
            DatePerhapsTime::Date(d) => Self{
                date: ExactDate::from_naive(d),
                time: ExactTime{hour: 0, minute:0, second:0},
                tz: TimeZoneChoice::Local
            },
            DatePerhapsTime::DateTime(dt) => match dt {
                CalendarDateTime::Floating(f) => Self::from_naive(f),
                CalendarDateTime::Utc(x) => Self{
                    tz: TimeZoneChoice::Utc,
                    ..Self::from_naive(x.naive_utc())
                },
                CalendarDateTime::WithTimezone {date_time, tzid: tz} => {
                    Self::from_naive(date_time)
                }
            }
        }
    }
}

impl ExactDate {
    pub fn from_naive(naive: NaiveDate) -> Self {
        Self {
            year: naive.year(),
            month: naive.month(),
            day: naive.day(),
        }
    }

    pub fn to_naive(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap()
    }

    pub fn from_date_perhaps_time(d: DatePerhapsTime) -> Self {
        match d {
            DatePerhapsTime::Date(d) => Self::from_naive(d),
            DatePerhapsTime::DateTime(dt) => match dt {
                CalendarDateTime::Floating(f) => Self::from_naive(f.date()),
                CalendarDateTime::Utc(x) => Self::from_naive(x.date_naive()),
                CalendarDateTime::WithTimezone { date_time, tzid: _ } => {
                    Self::from_naive(date_time.date())
                }
            },
        }
    }

    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }
}

impl Date {
    pub fn new() -> Self {
        use NumVal::Unsure;
        Date {
            year: Unsure,
            month: Unsure,
            day: Unsure,
        }
    }

    pub fn from_naive(naive: NaiveDate) -> Self {
        use NumVal::Number;
        Date {
            year: Number((naive.year()) as i64),
            month: Number((naive.month()) as i64),
            day: Number((naive.day()) as i64),
        }
    }

    pub fn to_naive(self) -> Option<NaiveDate> {
        use NumVal::*;
        match (self.year, self.month, self.day) {
            (Number(y), Number(m), Number(d)) => {
                Some(NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)?)
            }
            _ => None,
        }
    }

    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        use NumVal::Number;
        Date {
            year: Number(year as i64),
            month: Number(month as i64),
            day: Number(day as i64),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Todo{
    pub name: String,
    pub due: Option<ExactDate>,
    pub status: icalendar::TodoStatus
}

impl Todo{
    pub fn to_ical(&self, key: Option<String>, tsmp:Option<chrono::DateTime<Utc>>) -> Result<icalendar::Todo>{
        let mut tod = icalendar::Todo::new();
        tod.summary(&self.name);
        if let Some(due) = self.due {
            tod.due(due.to_chrono()?);
        }
        tod.status(self.status);
        if let Some(k) = key {
            tod.uid(&k.as_str());
        }
        if let Some(dt)=tsmp{
            tod.timestamp(dt);
        }
        Ok(tod)
    }

    pub fn from_string(s: String) -> Result<Self>{
        Ok(Self{
            name: s,
            due: None,
            status: icalendar::TodoStatus::NeedsAction
        })
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        Time {
            hour: NumVal::Unsure,
            minute: NumVal::Unsure,
            second: NumVal::Unsure,
            tod: None,
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
