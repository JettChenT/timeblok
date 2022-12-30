use clap::builder::Str;
use pest::pratt_parser::Op;

#[derive(Debug)]
pub enum NumVal{
    Number(i64),
    Unsure
}

#[derive(Debug)]
pub enum Range{
    TimeRange(TimeRange),
    Duration(Duration),
    AllDay
}

#[derive(Debug)]
pub enum ExactRange{
    TimeRange(ExactTimeRange),
    AllDay
}

#[derive(Debug)]
pub struct Duration{
    pub start: DateTime,
    pub duration: NumVal
}

#[derive(Debug)]
pub struct TimeRange {
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Debug)]
pub struct ExactTimeRange{
    pub start: ExactDateTime,
    pub end: ExactDateTime,
}

#[derive(Default, Debug)]
pub struct DateTime {
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub tz: Option<String>,
    // TODO: Implement Inheritance Time
    pub parent:Option<Box<DateTime>>
}

#[derive(Debug)]
pub struct ExactDateTime{
    pub date: ExactDate,
    pub time: ExactTime,
    pub tz: String,
}

#[derive(Debug)]
pub struct Date {
    pub year:NumVal,
    pub month:NumVal,
    pub day:NumVal
}

#[derive(Debug)]
pub struct ExactDate{
    pub year: i64,
    pub month: i64,
    pub day: i64
}

#[derive(Debug)]
pub enum TOD {
    AM,
    PM
}

#[derive(Debug)]
pub struct Time {
    pub hour:NumVal,
    pub minute:NumVal,
    pub second:NumVal,
    pub tod: Option<TOD>
}

#[derive(Debug)]
pub struct ExactTime{
    pub hour: i64,
    pub minute: i64,
    pub second: i64
}

#[derive(Debug)]
pub struct Event{
    pub range: Range,
    pub name: String,
    pub notes: Option<String>
}

#[derive(Debug)]
pub struct ExactEvent{
    pub range: ExactRange,
    pub name: String,
    pub notes: Option<String>
}

#[derive(Debug)]
pub enum Record{
    Event(Event),
    Occasion(DateTime),
    Note(String)
}

#[derive(Debug)]
pub enum ExactRecord{
    Event(ExactEvent),
    Occasion(ExactDateTime),
    Note(String)
}