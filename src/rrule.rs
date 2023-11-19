use crate::parse_date_string::{get_time_with_timezone, get_tz_from_str, ParsedDateString};
use chrono::{DateTime, Datelike, Duration, TimeZone, Weekday};
use chrono_tz::Tz;
use std::str::FromStr;

pub enum Frequency {
    /// The recurrence occurs on a yearly basis.
    Yearly = 0,
    /// The recurrence occurs on a monthly basis.
    Monthly = 1,
    /// The recurrence occurs on a weekly basis.
    Weekly = 2,
    /// The recurrence occurs on a daily basis.
    Daily = 3,
    /// The recurrence occurs on an hourly basis.
    Hourly = 4,
    /// The recurrence occurs on a minutely basis.
    Minutely = 5,
    /// The recurrence occurs on a second basis.
    Secondly = 6,
}

impl FromStr for Frequency {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let freq = match &value.to_uppercase()[..] {
            "YEARLY" => Self::Yearly,
            "MONTHLY" => Self::Monthly,
            "WEEKLY" => Self::Weekly,
            "DAILY" => Self::Daily,
            "HOURLY" => Self::Hourly,
            "MINUTELY" => Self::Minutely,
            "SECONDLY" => Self::Secondly,
            val => return Err(val.to_string()),
        };
        Ok(freq)
    }
}

pub enum RRuleProperty {
    Freq,
    Until,
    Count,
    Interval,
    BySecond,
    ByMinute,
    ByHour,
    ByDay,
    ByMonthDay,
    ByYearDay,
    ByWeekNo,
    ByMonth,
    BySetPos,
    Wkst,
}

impl FromStr for RRuleProperty {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prop = match &s.to_uppercase()[..] {
            "FREQ" => Self::Freq,
            "UNTIL" => Self::Until,
            "COUNT" => Self::Count,
            "INTERVAL" => Self::Interval,
            "BYSECOND" => Self::BySecond,
            "BYMINUTE" => Self::ByMinute,
            "BYHOUR" => Self::ByHour,
            "BYWEEKDAY" | "BYDAY" => Self::ByDay,
            "BYMONTHDAY" => Self::ByMonthDay,
            "BYYEARDAY" => Self::ByYearDay,
            "BYWEEKNO" => Self::ByWeekNo,
            "BYMONTH" => Self::ByMonth,
            "BYSETPOS" => Self::BySetPos,
            "WKST" => Self::Wkst,
            _ => return Err(s.into()),
        };
        Ok(prop)
    }
}

pub struct RRule {
    pub freq: Frequency,
    pub until: Option<ParsedDateString>,
    pub count: u32,
}
impl RRule {
    // 解析字符串，RRULE:FREQ=DAILY;COUNT=3。单行，不处理dt_start
    pub fn from_str(rrule_str: &str) -> RRule {
        let mut freq: Frequency = Frequency::Weekly;
        let mut count: u32 = 0;
        let mut until: Option<ParsedDateString> = None;
        let lines: Vec<&str> = rrule_str.split(':').collect();
        let parts: Vec<&str> = lines[1].split(";").collect();
        for part in parts {
            let key_value: Vec<&str> = part.split('=').collect();
            if key_value.len() == 2 {
                let key: RRuleProperty = key_value[0].parse().unwrap();
                let value = key_value[1];

                match key {
                    RRuleProperty::Freq => {
                        freq = value.parse().unwrap();
                    }
                    RRuleProperty::Count => {
                        count = value.parse().unwrap();
                    }
                    RRuleProperty::Until => {
                        until = match ParsedDateString::parse_from_str(value) {
                            Ok(i) => Some(i),
                            Err(_) => None,
                        }
                    }
                    // 其他RRule的参数，可以在这里处理
                    _ => {}
                }
            }
        }
        RRule { freq, count, until }
    }
}

pub struct RRuleSet {
    rrule: Vec<RRule>,
    dt_start: Option<DateTime<Tz>>,
    tz: Tz,
    parse_date_string: Option<ParsedDateString>,
}

// DTSTART:20120201T093000Z\nRRULE:FREQ=DAILY;COUNT=3

impl RRuleSet {
    // 解析整个字符串，单行，不处理dt_start
    pub fn from_str(s: &str) -> Result<RRuleSet, &str> {
        let lines: Vec<_> = s.split("\n").collect();
        if lines.len() == 2 {
            let parse_date_string = parse_dt_strart_str(lines[0])?;

            let rrule = RRule::from_str(lines[1]);
            return Ok(RRuleSet {
                rrule: vec![rrule],
                parse_date_string: Some(parse_date_string),
                tz: Tz::UTC,
                dt_start: None,
            });
        }
        let rrule = RRule::from_str(lines[0]);
        Ok(RRuleSet {
            rrule: vec![rrule],
            dt_start: None,
            tz: Tz::UTC,
            parse_date_string: None,
        })
    }
    pub fn add_rrule(&mut self, rrule: &str) {
        self.rrule.push(RRule::from_str(rrule))
    }

    pub fn tz(&mut self, tz: &str) {
        self.tz = get_tz_from_str(tz)
    }

    pub fn set_dt_start(&mut self, s: &str) {
        self.parse_date_string = Some(ParsedDateString::parse_from_str(s).unwrap());
    }

    pub fn all(&self, limit: u32) -> Vec<DateTime<Tz>> {
        let parse_date_string = match &self.parse_date_string {
            Some(i) => i,
            None => return Vec::new(),
        };
        if self.rrule.len() == 0 {
            return Vec::new();
        }
        // let rrule = &self.rrule[0];
        // let dt_start = get_time_with_timezone(parse_date_string, &self.tz);
        // let next_day = dt_start;
        // let weekday = dt_start.weekday();
        // let dates = get_alltime_by_limit(&dt_start, Weekday::Tue, limit);
        let dates = expand_point_time_by_week(parse_date_string, Weekday::Tue, limit)
            .into_iter()
            .map(|p| get_time_with_timezone(&p, &self.tz))
            .collect();
        dates
    }
}

fn parse_dt_strart_str(s: &str) -> Result<ParsedDateString, &str> {
    let key_value: Vec<_> = s.split(":").collect();
    ParsedDateString::parse_from_str(&key_value[1])
}

/// 找到从给定时间起的下一个指定的星期几的时刻
fn get_start_time_by_week(time: &DateTime<Tz>, weekday: Weekday) -> DateTime<Tz> {
    let currday = time.weekday();
    let mut next = time.clone();
    if currday == weekday {
        return next;
    }
    while next.weekday() != Weekday::Tue {
        next = next + Duration::days(1);
    }
    next
}

fn expand_point_time_by_week(
    point_time: &ParsedDateString,
    weekday: Weekday,
    limit: u32,
) -> Vec<ParsedDateString> {
    let curr = Tz::UTC
        .with_ymd_and_hms(point_time.year, point_time.month, point_time.day, 12, 0, 0)
        .single()
        .unwrap();
    let currday = curr.weekday();
    let mut next = curr.clone();
    if currday != weekday {
        while next.weekday() != weekday {
            next = next + Duration::days(1);
        }
    }
    let mut dates: Vec<ParsedDateString> = Vec::new();
    let generate_point_time = |d: &DateTime<Tz>| ParsedDateString {
        year: d.year(),
        month: d.month(),
        day: d.day(),
        hour: point_time.hour,
        min: point_time.min,
        sec: point_time.sec,
    };
    dates.push(generate_point_time(&next));
    for _ in 1..limit {
        next = next + Duration::weeks(1);
        dates.push(generate_point_time(&next));
    }
    dates
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;
    use chrono_tz::Tz;

    use super::*;

    #[test]
    fn test_get_start_time_by_week() {
        let date_time = Tz::UTC.with_ymd_and_hms(2023, 10, 23, 18, 0, 0).unwrap();
        let start = get_start_time_by_week(&date_time, chrono::Weekday::Tue);
        assert_eq!(
            start,
            Tz::UTC.with_ymd_and_hms(2023, 10, 24, 18, 0, 0).unwrap()
        )
    }

    #[test]
    fn test_get_alltime_by_limit() {
        let point_time = ParsedDateString {
            year: 2023,
            month: 10,
            day: 23,
            hour: 18,
            min: 0,
            sec: 0,
        };
        let dates = expand_point_time_by_week(&point_time, Weekday::Tue, 10);
        assert_eq!(dates.len(), 10);
        let last = dates.get(9).unwrap();
        assert_eq!(
            last,
            &ParsedDateString {
                month: 12,
                day: 26,
                ..point_time
            }
        )
    }

    #[test]
    fn test_rruleset() {
        let mut rrule_set = RRuleSet::from_str("RRULE:FREQ=WEEKLY;COUNT=3").unwrap();
        rrule_set.set_dt_start("20231001T180000");
        rrule_set.tz("America/New_York");
        let dates = rrule_set.all(10);
        assert_eq!(dates.len(), 10);
        println!(
            "{:?}",
            dates.iter().map(|d| d.to_string()).collect::<Vec<_>>()
        );
    }
}
