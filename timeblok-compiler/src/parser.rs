use std::fmt::Debug;

use crate::ir::command::CommandCall;
use crate::ir::filter;
use crate::ir::filter::BinFilt;
use crate::ir::filter::ExcludeFilt;
use crate::ir::filter::BDF;
use crate::ir::ident::{Ident, IdentFilter};
use crate::ir::Range::AllDay;
use crate::ir::*;
use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::PrattParser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "blok.pest"]
pub struct BlokParser;

macro_rules! get_next {
    ($pairs:ident) => {
        match $pairs.next() {
            Some(pair) => pair,
            None => return Err(anyhow!("Expected Token")),
        }
    };
}

macro_rules! get_match {
    ($fnc:ident, $pairs:ident) => {
        $fnc(get_next!($pairs))
    };
}

pub fn parse_file(pair: Pair<Rule>) -> Result<Vec<Record>> {
    let mut records = vec![];
    for record in pair.into_inner() {
        match record.as_rule() {
            Rule::RECORD => records.push(parse_record(record)?),
            _ => {
                return Err(anyhow!("Invalid record"));
            }
        }
    }
    Ok(records)
}

pub fn parse_record(pair: Pair<Rule>) -> Result<Record> {
    let mut pairs = pair.into_inner();
    let record = get_next!(pairs);
    match record.as_rule() {
        Rule::EVENT => {
            let event = parse_event(record)?;
            Ok(Record::Event(event))
        }
        Rule::OCCASION => {
            let occasion = parse_occasion(record)?;
            Ok(Record::Occasion(occasion))
        }
        Rule::NOTE_LINE => {
            let note = parse_note(record);
            Ok(Record::Note(note.to_string()))
        }
        Rule::FLEX_OCCASION => {
            let occasion = parse_flex_occasion(record)?;
            Ok(Record::FlexOccasion(occasion))
        }
        Rule::FLEX_EVENTS => {
            let flex_events = parse_flex_events(record)?;
            Ok(Record::FlexEvents(flex_events))
        }
        Rule::COMMAND => {
            let command = parse_command(record)?;
            Ok(Record::Command(command))
        }
        _ => Err(anyhow!(format!("Invalid record: {:?}", record))),
    }
}

fn parse_flex_events(pair: Pair<Rule>) -> Result<FlexEvents> {
    let mut pairs = pair.into_inner();
    let condition = get_match!(parse_flex_occasion, pairs)?;
    let mut events = vec![];
    while pairs.peek().is_some() {
        let nxt = get_next!(pairs);
        match nxt.as_rule() {
            Rule::EVENT => {
                let event = parse_event(nxt)?;
                events.push(event);
            }
            _ => unreachable!("Invalid rule"),
        }
    }
    Ok(FlexEvents {
        occasion: condition,
        events,
    })
}

fn parse_command(pair: Pair<Rule>) -> Result<CommandCall> {
    let mut pairs = pair.into_inner();
    let command = get_next!(pairs);
    let mut argpairs = get_next!(pairs).into_inner();
    let s = argpairs.as_str();
    let mut args = vec![];
    while argpairs.peek().is_some() {
        let nxt = get_next!(argpairs);
        let res = match nxt.as_rule() {
            Rule::FILTER => Value::NumFilter(parse_num_filter(nxt)?),
            Rule::DATE_FILTER => Value::DateFilter(parse_date_filter(nxt)?),
            Rule::IDENT => Value::Ident(parse_ident(nxt)?),
            Rule::NUM_FIELD => Value::Num(parse_numval(nxt)?),
            Rule::EOI => {break;}
            Rule::CARG => Value::String(nxt.as_str().to_string()),
            Rule::STRING => Value::String(nxt.as_str().to_string()),
            r => {
                eprintln!("unexpected rule: {:?}", r);
                unreachable!()
            },
        };
        args.push(res);
    }
    Ok(CommandCall {
        command: command.as_str().to_string(),
        args,
        plain: s.to_string(),
    })
}

fn parse_occasion(pair: Pair<Rule>) -> Result<DateTime> {
    let mut pairs = pair.clone().into_inner();
    let pair = get_next!(pairs);
    match pair.as_rule() {
        Rule::DATETIME => {
            let date: Date = parse_date(pair)?;
            let time: Time = get_match!(parse_time, pairs)?;
            Ok(DateTime {
                date: Some(date),
                time: Some(time),
                ..Default::default()
            })
        }
        Rule::DATE => {
            // let mut pairs = pair.into_inner();
            let date: Date = parse_date(pair)?;
            Ok(DateTime {
                date: Some(date),
                ..Default::default()
            })
        }
        Rule::TIME => {
            let time: Time = parse_time(pair)?;
            Ok(DateTime {
                time: Some(time),
                ..Default::default()
            })
        }
        _ => Err(anyhow!("Invalid occasion")),
    }
}

fn parse_date(pair: Pair<Rule>) -> Result<Date> {
    let mut pairs = pair.into_inner();
    let year = get_match!(parse_numval, pairs)?;
    let month = get_match!(parse_numval, pairs)?;
    let day = get_match!(parse_numval, pairs)?;
    Ok(Date { year, month, day })
}

fn parse_time(pair: Pair<Rule>) -> Result<Time> {
    let mut pairs = pair.into_inner();
    let hour = get_match!(parse_numval, pairs)?;
    let mut res = Time {
        hour,
        minute: NumVal::Unsure,
        second: NumVal::Unsure,
        tod: None,
    };
    if let Some(r) = pairs.peek() {
        match r.as_rule() {
            Rule::FIELD => {
                res.minute = get_match!(parse_numval, pairs)?;
                if let Some(r) = pairs.next() {
                    res.tod = Some(parse_tod(r)?);
                }
            }
            Rule::TOD => {
                res.tod = Some(get_match!(parse_tod, pairs)?);
            }
            _ => Err(anyhow!("Invalid time"))?,
        }
    };
    Ok(res)
}

pub fn parse_tod(pair: Pair<Rule>) -> Result<Tod> {
    let mut inner = pair.into_inner();
    match get_next!(inner).as_rule() {
        Rule::AM => Ok(Tod::AM),
        Rule::PM => Ok(Tod::PM),
        _ => Err(anyhow!("Invalid TOD")),
    }
}

pub fn parse_event(pair: Pair<Rule>) -> Result<Event> {
    // not quite sure if turn into pairs before or after function execution
    let mut pairs = pair.into_inner();
    let mut event: Event = {
        let raw = get_next!(pairs);
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
    let timerange = get_match!(parse_timerange, pairs)?;
    let name = get_match!(parse_note, pairs).to_string();

    Ok(Event {
        range: timerange,
        name,
        notes: None,
    })
}

fn parse_timerange(pair: Pair<Rule>) -> Result<Range> {
    match pair.as_rule() {
        Rule::RANGE => {
            let mut pairs = pair.into_inner();
            let start = get_match!(parse_occasion, pairs)?;
            let end = get_match!(parse_occasion, pairs)?;
            Ok(Range::Time(TimeRange { start, end }))
        }
        Rule::OCCASION => {
            let occasion = parse_occasion(pair)?;
            match occasion.time {
                Some(_) => Ok(Range::Duration(Duration {
                    start: occasion,
                    duration: NumVal::Unsure,
                })),
                None => match occasion.date {
                    None => return Err(anyhow!("Invalid occasion")),
                    Some(date) => Ok(AllDay(date)),
                },
            }
        }
        _ => Err(anyhow!("Invalid Time Range for Event!")),
    }
}

fn parse_numval(pair: Pair<Rule>) -> Result<NumVal> {
    Ok(match pair.as_str().parse::<i64>() {
        Ok(n) => NumVal::Number(n),
        _ => NumVal::Unsure,
    })
}

fn parse_numrange(pair: Pair<Rule>) -> Result<NumRange> {
    let mut pairs = pair.into_inner();
    let start = get_match!(parse_numval, pairs)?;
    let end = get_match!(parse_numval, pairs)?;
    Ok(NumRange { start, end })
}

fn parse_flex_date(pair: Pair<Rule>) -> Result<FlexDate> {
    let mut pairs = pair.into_inner();
    let year = get_match!(parse_num_filter, pairs)?;
    let month = get_match!(parse_num_filter, pairs)?;
    let day = get_match!(parse_num_filter, pairs)?;
    Ok(FlexDate { year, month, day })
}

fn parse_ident(pair: Pair<Rule>) -> Result<Ident> {
    let name = pair.as_str().to_string();
    Ok(Ident { name })
}

fn parse_ident_date_filter(pair: Pair<Rule>) -> Result<BDF<Date>> {
    let ident = parse_ident(pair)?;
    Ok(Box::new(IdentFilter { ident }))
}

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;
        PrattParser::new()
            .op(Op::infix(OR, Left) | Op::infix(AND, Left))
            .op(Op::prefix(NOT))
    };
}

pub fn parse_date_filter(pair: Pair<Rule>) -> Result<BDF<Date>> {
    let pairs = pair.into_inner();
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::FILTER => parse_date_filter(primary),
            Rule::UNIT_DATE_FILTER => parse_date_filter(primary),
            Rule::DATE_FILTER => parse_date_filter(primary),
            Rule::IDENT => parse_ident_date_filter(primary),
            Rule::RANGE => {
                let trange = parse_timerange(primary)?;
                Ok(Box::new(trange) as BDF<Date>)
            }
            Rule::FLEX_DATE => {
                let date = parse_flex_date(primary)?;
                Ok(Box::new(date) as BDF<Date>)
            }
            _ => {
                eprintln!("Invalid date filter: {:?}", primary);
                todo!()
            }
        })
        .map_infix(|lhs, op, rhs| {
            let lhs = lhs?;
            let rhs = rhs?;
            match op.as_rule() {
                Rule::OR => Ok(Box::new(BinFilt {
                    lhs,
                    rhs,
                    op: filter::Op::OR,
                })),
                Rule::AND => Ok(Box::new(BinFilt {
                    lhs,
                    rhs,
                    op: filter::Op::And,
                })),
                _ => unreachable!("Invalid infix rule"),
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::NOT => {
                let target = rhs?;
                Ok(Box::new(ExcludeFilt { target }))
            }
            _ => unreachable!(),
        })
        .parse(pairs)
}

pub fn parse_num_filter(pair: Pair<Rule>) -> Result<BDF<NumVal>> {
    let pairs = pair.into_inner();
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::NUM_RANGE => {
                let num_range = parse_numrange(primary)?;
                Ok(Box::new(num_range) as BDF<NumVal>)
            }
            Rule::NUM_FIELD => {
                let num = parse_numval(primary)?;
                Ok(Box::new(num) as BDF<NumVal>)
            }
            Rule::UNSURE => Ok(Box::new(NumVal::Unsure) as BDF<NumVal>),
            _ => {
                eprintln!("Invalid number filter: {:?}", primary);
                todo!()
            }
        })
        .map_infix(|lhs, op, rhs| {
            let lhs = lhs?;
            let rhs = rhs?;
            match op.as_rule() {
                Rule::OR => Ok(Box::new(BinFilt {
                    lhs,
                    rhs,
                    op: filter::Op::OR,
                })),
                Rule::AND => Ok(Box::new(BinFilt {
                    lhs,
                    rhs,
                    op: filter::Op::And,
                })),
                _ => unreachable!("Invalid infix rule"),
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::NOT => {
                let target = rhs?;
                Ok(Box::new(ExcludeFilt { target }))
            }
            _ => unreachable!(),
        })
        .parse(pairs)
}

pub fn parse_flex_occasion(pair: Pair<Rule>) -> Result<FlexOccasion> {
    let mut pairs = pair.into_inner();
    let fst = get_next!(pairs);
    match fst.as_rule() {
        Rule::FLEX_DATETIME => todo!(),
        Rule::DATE_FILTER => {
            let filter = parse_date_filter(fst)?;
            Ok(FlexOccasion::Filter(filter))
        }
        sth => unreachable!("Invalid flex occasion rule: {:?}", sth),
    }
}
