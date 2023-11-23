use crate::point_time::PointTime;
use chrono::{DateTime, Datelike, Duration, Weekday};
use chrono_tz::Tz;
use std::str::FromStr;

use self::weekday::{parse_weekdays, str_to_weekday, NWeekday};
mod frequency;
pub use frequency::Frequency;
pub mod weekday;


#[derive(Debug)]
pub enum RRuleProperty {
    Freq(Frequency),
    Until(PointTime),
    Count(u32),
    Interval(u32),
    BySecond,
    ByMinute,
    ByHour,
    ByDay(Vec<NWeekday>),
    ByMonthDay(Vec<i16>),
    ByYearDay,
    ByWeekNo,
    ByMonth,
    BySetPos,
    Wkst(Weekday),
}

impl FromStr for RRuleProperty {
    type Err = String;
    /// parse str with FREQ=WEEKLY
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key_value: Vec<&str> = s.split('=').collect();
        if key_value.len() != 2 {
            return Err("invalid rrule property".to_string());
        }
        let key = key_value[0];
        let value = key_value[1];
        let prop = match &key.to_uppercase()[..] {
            "FREQ" => Self::Freq(value.parse()?),
            "UNTIL" => Self::Until(value.parse()?),
            "COUNT" => Self::Count(value.parse().unwrap_or(0)),
            "INTERVAL" => Self::Interval(value.parse().unwrap_or(1)),
            "BYSECOND" => Self::BySecond,
            "BYMINUTE" => Self::ByMinute,
            "BYHOUR" => Self::ByHour,
            "BYWEEKDAY" | "BYDAY" => Self::ByDay(parse_weekdays(value).unwrap()),
            "BYMONTHDAY" => {
                Self::ByMonthDay(value.split(",").map(|s| s.parse().unwrap_or(0)).collect())
            }
            "BYYEARDAY" => Self::ByYearDay,
            "BYWEEKNO" => Self::ByWeekNo,
            "BYMONTH" => Self::ByMonth,
            "BYSETPOS" => Self::BySetPos,
            "WKST" => Self::Wkst(str_to_weekday(value).unwrap()),
            _ => return Err(s.into()),
        };
        Ok(prop)
    }
}
#[derive(PartialEq, Debug)]
pub struct RRule {
    pub freq: Frequency,
    pub until: Option<PointTime>,
    pub count: u32,
    pub by_day: Vec<NWeekday>,
    pub interval: u32,
    pub week_start: Weekday,
    pub by_month_day: Vec<i16>,
}
impl RRule {
    pub fn default() -> RRule {
        RRule {
            freq: Frequency::Weekly,
            count: 0,
            by_day: vec![],
            until: None,
            interval: 1,
            week_start: Weekday::Sun,
            by_month_day: vec![],
        }
    }
    // 解析字符串，RRULE:FREQ=DAILY;COUNT=3。单行，不处理dt_start
    pub fn from_str(rrule_str: &str) -> RRule {
        let mut freq: Frequency = Frequency::Weekly;
        let mut count: u32 = 0;
        let mut until: Option<PointTime> = None;
        let mut by_day = vec![];
        let mut interval = 1;
        let mut week_start = Weekday::Sun;
        let mut by_month_day: Vec<i16> = vec![];
        let lines: Vec<&str> = rrule_str.split(':').collect();
        let parts: Vec<&str> = if lines.len() == 2 {
            lines[1].split(";").collect()
        } else {
            lines[0].split(";").collect()
        };

        for part in parts {
            let key_value: Vec<&str> = part.split('=').collect();
            if key_value.len() == 2 {
                let key: RRuleProperty = part.parse().unwrap();
                match key {
                    RRuleProperty::Freq(f) => {
                        freq = f;
                    }
                    RRuleProperty::Count(c) => {
                        count = c;
                    }
                    RRuleProperty::Until(p) => {
                        until = Some(p);
                    }
                    RRuleProperty::ByDay(days) => {
                        by_day = days;
                    }
                    RRuleProperty::Interval(number) => {
                        interval = number;
                    }
                    RRuleProperty::Wkst(day) => {
                        week_start = day;
                    }
                    RRuleProperty::ByMonthDay(vec) => {
                        by_month_day = vec;
                    }
                    // 其他RRule的参数，可以在这里处理
                    _ => {}
                }
            }
        }
        RRule {
            freq,
            count,
            until,
            by_day,
            interval,
            week_start,
            by_month_day,
        }
    }

    pub fn set_count(&mut self, count: u32){
        self.count = count;
    }
}

pub fn get_tz_from_str(tz: &str) -> Tz {
    let chrono_tz: Tz = tz.parse().unwrap();
    chrono_tz
}

pub fn parse_dt_strart_str(s: &str) -> Result<PointTime, String> {
    let key_value: Vec<_> = s.split(":").collect();
    key_value[1].parse()
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

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use chrono_tz::Tz;

    #[test]
    fn parse_tz() {
        let s = String::from("America/New_York");
        assert_eq!(get_tz_from_str(&s), Tz::America__New_York);
    }

    #[test]
    fn parse_time_with_tz() {
        let tz: Tz = get_tz_from_str("America/New_York");
        let time: DateTime<Tz> = "20231115T191020"
            .parse::<PointTime>()
            .unwrap()
            .with_timezone(&tz);
        assert_eq!(
            time,
            Tz::America__New_York
                .with_ymd_and_hms(2023, 11, 15, 19, 10, 20)
                .single()
                .unwrap()
        );
    }

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
    fn test_only_rrule() {
        let s = "FREQ=DAILY;COUNT=3;BYDAY=TU,WE";
        let rrule = RRule::from_str(s);
        assert_eq!(rrule, RRule::from_str("FREQ=DAILY;COUNT=3;BYDAY=TU,WE"))
    }
}
