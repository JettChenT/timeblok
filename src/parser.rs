use anyhow::Result;
use anyhow::anyhow;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use crate::ir::*;

#[derive(Parser)]
#[grammar = "blok.pest"]
pub struct BlokParser;

macro_rules! get_match {
    ($fnc:ident, $pairs:ident) => {
        $fnc($pairs.next().unwrap())
    };
}

pub fn parse_file(pair: Pair<Rule>) -> Result<Vec<Record>> {
    let mut records = vec![];
    for record in pair.into_inner() {
        match record.as_rule() {
            Rule::RECORD => {
                records.push(parse_record(record)?)
            }
            _ => {
                return Err(anyhow!("Invalid record"));
            }
        }
    }
    Ok(records)
}

pub fn parse_record(pair: Pair<Rule>) -> Result<Record> {
    let mut pairs = pair.into_inner();
    let record = pairs.next().unwrap();
    match record.as_rule() {
        Rule::EVENT => {
            let event = parse_event(record)?;
            Ok(Record::Event(event))
        },
        Rule::OCCASION => {
            let occasion = parse_occasion(record)?;
            Ok(Record::Occasion(occasion))
        },
        Rule::NOTE_LINE => {
            let note = parse_note(record);
            Ok(Record::Note(note.to_string()))
        },
        _ => {
            Err(anyhow!(format!("Invalid record: {:?}", record)))
        }
    }
}

fn parse_occasion(pair: Pair<Rule>) -> Result<DateTime> {
    let mut pairs = pair.clone().into_inner();
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::DATETIME => {
            let date:Date = parse_date(pair)?;
            let time:Time = get_match!(parse_time, pairs)?;
            Ok(DateTime {
                date: Some(date),
                time: Some(time),
                ..Default::default()
            })
        },
        Rule::DATE => {
            // let mut pairs = pair.into_inner();
            let date:Date = parse_date(pair)?;
            Ok(DateTime {
                date: Some(date),
                ..Default::default()
            })
        },
        Rule::TIME => {
            let time:Time = parse_time(pair)?;
            Ok(DateTime {
                time: Some(time),
                ..Default::default()
            })
        },
        _ => Err(anyhow!("Invalid occasion"))
    }
}

fn parse_date(pair: Pair<Rule>) -> Result<Date> {
    let mut pairs = pair.into_inner();
    let year = parse_numval(pairs.next().unwrap())?;
    let month = parse_numval(pairs.next().unwrap())?;
    let day = parse_numval(pairs.next().unwrap())?;
    Ok(Date {
        year,
        month,
        day
    })
}

fn parse_time(pair: Pair<Rule>) -> Result<Time> {
    let mut pairs = pair.into_inner();
    let hour = get_match!(parse_numval, pairs)?;
    let mut res = Time {
        hour,
        minute: NumVal::Unsure,
        second: NumVal::Unsure,
        tod: None
    };
    if let Some(r) = pairs.peek() {
        match r.as_rule() {
            Rule::FIELD => {
                res.minute = get_match!(parse_numval, pairs)?;
                if let Some(r) = pairs.next() {
                    res.tod = Some(parse_tod(r)?);
                }
            },
            Rule::TOD => {
                res.tod = Some(get_match!(parse_tod, pairs)?);
            },
            _ => {Err(anyhow!("Invalid time"))?}
        }
    };
    Ok(res)
}

pub fn parse_tod(pair: Pair<Rule>) -> Result<Tod> {
    match pair.into_inner().next().unwrap().as_rule() {
        Rule::AM => Ok(Tod::AM),
        Rule::PM => Ok(Tod::PM),
        _ => Err(anyhow!("Invalid TOD"))
    }
}

pub fn parse_event(pair: Pair<Rule>) -> Result<Event> {
    // not quite sure if turn into pairs before or after function execution
    let mut pairs = pair.into_inner();
    let mut event:Event = {
        let raw = pairs.next().unwrap();
        parse_event_header(raw)?
    };
    if pairs.peek().is_some() {
        // Assuming that all stuff are notes for now...
        let notes = parse_notes(&mut pairs);
        let descriptions = notes.join("\n");
        event.notes = Some(descriptions);
    }
    Ok(event)
}

fn parse_note(pair: Pair<Rule>) -> &str {
    pair.as_str()
}

fn parse_notes(pairs: &mut Pairs<Rule>) -> Vec<String> {
    let mut notes = vec![];
    for note in pairs {
        notes.push(parse_note(note).to_string());
    }
    notes
}

fn parse_event_header(pair: Pair<Rule>) -> Result<Event> {
    let mut pairs = pair.into_inner();
    let timerange = parse_timerange(pairs.next().unwrap())?;
    let name = parse_note(pairs.next().unwrap()).to_string();

    Ok(Event {
        range: timerange,
        name,
        notes:None
    })
}

fn parse_timerange(pair: Pair<Rule>) -> Result<Range> {
    match pair.as_rule(){
        Rule::RANGE => {
            let mut pairs = pair.into_inner();
            let start = parse_occasion(pairs.next().unwrap())?;
            let end = parse_occasion(pairs.next().unwrap())?;
            Ok(
                Range::TimeRange(
                    TimeRange {
                        start,
                        end
                    }
                )
            )
        },
        Rule::OCCASION => {
            let occasion = parse_occasion(pair)?;
            match occasion.time {
                Some(_) => {
                    Ok(Range::Duration(Duration{
                        start: occasion,
                        duration: NumVal::Number(30)
                    }))
                },
                None => {
                    Ok(Range::AllDay(occasion.date.unwrap()))
                }
            }
        },
        _ => Err(anyhow!("Invalid Time Range for Event!"))
    }
}

fn parse_numval(pair: Pair<Rule>) -> Result<NumVal> {
    Ok(match pair.as_str().parse::<i64>(){
        Ok(n) => NumVal::Number(n),
        _ => NumVal::Unsure
    })
}