use std::collections::HashSet;

use crate::{
    environment::Environment,
    ir::{filter::Filter, Date, ExactDate},
    resolver::resolve_date,
};
use chrono::NaiveDate;
use icalendar::{Calendar, Component};

#[derive(Debug, Clone)]
pub struct SetFilter {
    dates: HashSet<ExactDate>,
}

impl Filter<Date> for SetFilter {
    fn check(&self, value: &Date, env: Option<&Environment>) -> bool {
        self.dates
            .contains(&resolve_date(value, env.unwrap()).unwrap())
    }
}

impl SetFilter {
    pub fn from_naive_dates(dates: impl IntoIterator<Item = NaiveDate>) -> Self {
        let mut set = HashSet::new();
        for date in dates {
            set.insert(ExactDate::from_naive(date));
        }
        SetFilter { dates: set }
    }

    pub fn from_ics(cal: &Calendar) -> Self {
        let mut dates = HashSet::new();
        for c in cal.iter() {
            if let Some(event) = c.as_event() {
                match (event.get_start(), event.get_end()) {
                    (Some(st), Some(nd)) => {
                        let est = ExactDate::from_date_perhaps_time(st).to_naive();
                        let end = ExactDate::from_date_perhaps_time(nd).to_naive();
                        for i in 0..=(end - est).num_days() {
                            dates.insert(ExactDate::from_naive(est + chrono::Duration::days(i)));
                        }
                    }
                    (Some(st), None) => {
                        let est = ExactDate::from_date_perhaps_time(st).to_naive();
                        dates.insert(ExactDate::from_naive(est));
                    }
                    (None, Some(nd)) => {
                        let end = ExactDate::from_date_perhaps_time(nd).to_naive();
                        dates.insert(ExactDate::from_naive(end));
                    }
                    (_, _) => {}
                }
            }
        }
        Self { dates }
    }
}
