use std::any::Any;

use crate::filter::Filter;

#[derive(Debug, Eq, PartialEq)]
pub enum NumVal{
    Number(i64),
    Unsure
}

#[derive(Debug)]
pub struct NumRange{
    pub start:NumVal,
    pub end:NumVal
}



#[derive(Debug)]
pub enum Range{
    TimeRange(TimeRange),
    Duration(Duration),
    AllDay(Date)
}

#[derive(Debug)]
pub enum ExactRange{
    TimeRange(ExactTimeRange),
    AllDay(ExactDate)
}

#[derive(Debug)]
pub struct ExactDuration{
    pub start: ExactDateTime,
    pub duration: u64
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

#[derive(Debug, Copy, Clone)]
pub enum TimeZoneChoice {
    Local,
    Utc,
}

#[derive(Default, Debug)]
pub struct DateTime {
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub tz: Option<TimeZoneChoice>,
}

#[derive(Debug)]
pub struct ExactDateTime{
    pub date: ExactDate,
    pub time: ExactTime,
    pub tz: TimeZoneChoice,
}

#[derive(Debug)]
pub enum FlexField{
    NumVal(NumVal),
    NumRange(NumRange),
}

#[derive(Debug)]
pub struct FlexDate{
    pub year: FlexField,
    pub month: FlexField,
    pub day: FlexField,
}

#[derive(Debug)]
pub struct Date {
    pub year:NumVal,
    pub month:NumVal,
    pub day:NumVal
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExactDate{
    pub year: i32,
    pub month: u32,
    pub day: u32
}

#[derive(Debug)]
pub enum Tod {
    // Add support for resolving
    AM,
    PM
}

#[derive(Debug)]
pub struct Time {
    pub hour:NumVal,
    pub minute:NumVal,
    pub second:NumVal,
    pub tod: Option<Tod>
}

#[derive(Debug, Clone, Copy)]
pub struct ExactTime{
    pub hour: u32,
    pub minute: u32,
    pub second: u32
}

// PERFORMANCE: Figure out how to use &str instead of String
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
    Note(String),
    FlexOccasion(FlexOccasion),
    FlexEvents(FlexEvents)
}

#[derive(Debug)]
pub enum ExactRecord{
    Event(ExactEvent),
    Note(String)
}

#[derive(Debug)]
pub enum FlexOccasion {
    Filter(Box<dyn Filter<Date>>),
}

#[derive(Debug)]
pub struct FlexEvents{
    pub occasion: FlexOccasion,
    pub events: Vec<Event>
}