use super::*;

impl ToString for ExactTime {
    fn to_string(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

impl ToString for ExactDate {
    fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl ToString for ExactDateTime {
    fn to_string(&self) -> String {
        format!(
            "{} {}",
            self.date.to_string(),
            self.time.to_string()
        )
    }
}

impl ToString for ExactTimeRange{
    fn to_string(&self) -> String{
        format!("{}-{}", self.start.to_string(), self.end.to_string())
    }
}

impl ToString for ExactRange{
    fn to_string(&self) -> String{
        match self {
            ExactRange::AllDay(d) => d.to_string(),
            ExactRange::TimeRange(d) =>  d.to_string()
        }
    }
}