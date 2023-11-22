use crate::point_time::PointTime;
use crate::rrule::weekday::NWeekday;
use crate::rrule::{get_tz_from_str, parse_dt_strart_str, RRule};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Weekday};
use chrono_tz::Tz;
pub struct RRuleSet {
    rrule: Vec<RRule>,
    tz: Tz,
    start_point_time: Option<PointTime>,
    limit: u32,
}

impl RRuleSet {
    // 解析整个字符串，单行，不处理dt_start
    pub fn from_str(s: &str) -> Result<RRuleSet, String> {
        let lines: Vec<_> = s.split("\n").collect();
        if lines.len() == 2 {
            let start_point_time = parse_dt_strart_str(lines[0])?;

            let rrule = RRule::from_str(lines[1]);
            return Ok(RRuleSet {
                rrule: vec![rrule],
                start_point_time: Some(start_point_time),
                tz: Tz::UTC,
                limit: 0,
            });
        }
        let rrule = RRule::from_str(lines[0]);
        Ok(RRuleSet {
            rrule: vec![rrule],
            tz: Tz::UTC,
            start_point_time: None,
            limit: 0,
        })
    }
    pub fn add_rrule(&mut self, rrule: &str) {
        self.rrule.push(RRule::from_str(rrule))
    }

    pub fn set_dt_start(&mut self, str: &str) {
        let time = str.parse::<PointTime>();
        if let Ok(point_time) = time {
            self.start_point_time = Some(point_time);
        }
    }

    pub fn tz(&mut self, tz: &str) {
        self.tz = get_tz_from_str(tz)
    }

    pub fn all(&self) -> Vec<DateTime<Tz>> {
        if self.start_point_time.is_none() {
            return Vec::new();
        }
        if self.rrule.len() == 0 {
            return Vec::new();
        }

        // 只要长度不为0，就一定有值
        let rrule = self.rrule.get(0).unwrap();

        // 如果没设置长度和截止时间，直接返回[]
        if self.limit == 0 && rrule.until.is_none() {
            return Vec::new();
        }

        // 如果长度为0，并且开始时间大于截止时间，直接返回[]
        if self.limit == 0 {
            let start_point_time = self.start_point_time.as_ref().unwrap();
            let end_point_time = rrule.until.as_ref().unwrap();
            if start_point_time > end_point_time {
                return vec![];
            }
        }

        match rrule.freq {
            crate::rrule::Frequency::Yearly => todo!(),
            crate::rrule::Frequency::Monthly => self
                .expand_by_month()
                .unwrap()
                .into_iter()
                .map(|p| p.with_timezone(&self.tz))
                .collect(),
            crate::rrule::Frequency::Weekly => self
                .expand_by_week()
                .unwrap()
                .into_iter()
                .map(|p| p.with_timezone(&self.tz))
                .collect(),
            crate::rrule::Frequency::Daily => self
                .expand_by_day()
                .unwrap()
                .into_iter()
                .map(|p| p.with_timezone(&self.tz))
                .collect(),
            crate::rrule::Frequency::Hourly => todo!(),
            crate::rrule::Frequency::Minutely => todo!(),
            crate::rrule::Frequency::Secondly => todo!(),
        }
    }

    /// 按周扩展，无效则报错
    fn expand_by_week(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();

        let rrule = self.rrule.get(0).unwrap();
        let limit = self.limit;
        let binding = vec![NWeekday::Every(Weekday::Mon)];
        let by_day = rrule.by_day.as_deref().unwrap_or(&binding);
        let weekday = by_day[0].get_weekday();
        let interval = rrule.interval;

        let curr = Tz::UTC
            .with_ymd_and_hms(point_time.year, point_time.month, point_time.day, 12, 0, 0)
            .single()
            .unwrap();

        let currday = curr.weekday();

        let mut next = curr.clone();
        if currday != *weekday {
            while next.weekday() != *weekday {
                next = next + Duration::days(1);
            }
        }
        let mut dates: Vec<PointTime> = Vec::new();
        let generate_point_time = |d: &DateTime<Tz>| PointTime {
            year: d.year(),
            month: d.month(),
            day: d.day(),
            hour: point_time.hour,
            min: point_time.min,
            sec: point_time.sec,
        };
        let by_weekdays: Vec<Weekday> = by_day.iter().map(|n| n.get_weekday().clone()).collect();
        // let until = rrule.until;
        match &rrule.until {
            Some(until) => {
                let end_time = Self::get_datetime_by_point_time(until, &Tz::UTC);
                while next < end_time {
                    let weekday = next.weekday();
                    if by_weekdays.contains(&weekday) {
                        dates.push(generate_point_time(&next));
                    }
                    if weekday == rrule.week_start && interval != 1 {
                        next = next + Duration::weeks((interval - 1).into())
                    } else {
                        next = next + Duration::days(1);
                    }
                }
            }
            None => {
                for _ in 0..limit {
                    dates.push(generate_point_time(&next));
                    next = next + Duration::weeks(interval.into());
                }
            }
        }

        Ok(dates)
    }
    /// 按天扩展，无效则报错
    fn expand_by_day(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();

        let rrule = self.rrule.get(0).unwrap();
        let limit = self.limit;
        let interval = rrule.interval;
        let curr = Tz::UTC
            .with_ymd_and_hms(point_time.year, point_time.month, point_time.day, 12, 0, 0)
            .single()
            .unwrap();

        let mut next = curr.clone();

        let mut dates: Vec<PointTime> = Vec::new();
        let generate_point_time = |d: &DateTime<Tz>| PointTime {
            year: d.year(),
            month: d.month(),
            day: d.day(),
            hour: point_time.hour,
            min: point_time.min,
            sec: point_time.sec,
        };

        match &rrule.until {
            Some(until) => {
                let end_time = Self::get_datetime_by_point_time(until, &Tz::UTC);
                while next < end_time {
                    dates.push(generate_point_time(&next));
                    next = next + Duration::days(interval.into());
                }
            }
            None => {
                for _ in 0..limit {
                    dates.push(generate_point_time(&next));
                    next = next + Duration::days(interval.into());
                }
            }
        }

        Ok(dates)
    }

    /// 按月扩展，无效则报错
    /// byMonth 需要考虑 limit until interval byday
    fn expand_by_month(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();
        let rrule = self.rrule.get(0).unwrap();
        let limit = self.limit;
        let interval = rrule.interval;
        let by_day = rrule.by_day.as_deref();
        let mut dates: Vec<PointTime> = Vec::new();

        let month_start = NaiveDate::from_ymd_opt(point_time.year, point_time.month, 1).unwrap();
        let mut next = point_time.clone();
        // if by_day is none, simply plus month
        if by_day.is_none() {
            match &rrule.until {
                Some(until) => {
                    while &next < until {
                        dates.push(next.clone());
                        next = next.add_month(interval);
                    }
                }
                None => {
                    for _ in 0..limit {
                        dates.push(next.clone());
                        next = next.add_month(interval);
                    }
                }
            };
            return Ok(dates);
        }

        let by_day = rrule.by_day.as_ref().unwrap();
        for n_weekday in by_day.iter() {
            match n_weekday {
                NWeekday::Every(weekday) => {
                    for i in 1..5 {
                        let naive_date = NaiveDate::from_weekday_of_month_opt(
                            next.year, next.month, *weekday, i,
                        );
                        if let Some(naive) = naive_date {
                            dates.push(PointTime {
                                year: naive.year(),
                                month: naive.month(),
                                day: naive.day(),
                                hour: next.hour,
                                min: next.min,
                                sec: next.sec,
                            });
                        }
                    }
                }
                NWeekday::Nth(n, weekday) => {
                    let naive_date =
                        NaiveDate::from_weekday_of_month_opt(next.year, next.month, *weekday, n);
                    if let Some(naive) = naive_date {
                        dates.push(PointTime {
                            year: naive.year(),
                            month: naive.month(),
                            day: naive.day(),
                            hour: next.hour,
                            min: next.min,
                            sec: next.sec,
                        });
                    }
                }
            }
        }
        Ok(dates)
    }

    fn get_datetime_by_point_time(point_time: &PointTime, tz: &Tz) -> DateTime<Tz> {
        Tz::UTC
            .with_ymd_and_hms(point_time.year, point_time.month, point_time.day, 12, 0, 0)
            .single()
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::rrule::Frequency;

    use super::*;
    use chrono_tz::Tz;

    #[test]
    fn test_expand_by_week() {
        let point_time = "20231023T180000Z".parse().unwrap();
        let set = RRuleSet {
            start_point_time: Some(point_time),
            rrule: vec![RRule {
                by_day: Some(vec![NWeekday::Every(Weekday::Tue)]),
                ..RRule::default()
            }],
            tz: Tz::UTC,
            limit: 10,
        };
        let dates = set.expand_by_week().unwrap();
        assert_eq!(dates.len(), 10);
        let first = dates.get(0).unwrap();
        assert_eq!(first, &"20231024T180000Z".parse().unwrap());
        let last = dates.get(9).unwrap();
        assert_eq!(last, &"20231226T180000Z".parse().unwrap())
    }

    #[test]
    fn test_expand_by_week_interval2() {
        let point_time = "20231023T180000Z".parse().unwrap();
        let set = RRuleSet {
            start_point_time: Some(point_time),
            rrule: vec![RRule::from_str("Freq=WEEKLY;BYDAY=TU;INTERVAL=2")],
            tz: Tz::UTC,
            limit: 2,
        };
        let dates = set.expand_by_week().unwrap();
        assert_eq!(dates.len(), 2);
        let first = dates.get(0).unwrap();
        assert_eq!(first, &"20231024T180000Z".parse().unwrap());
        let last = dates.get(1).unwrap();
        assert_eq!(last, &"20231107T180000Z".parse().unwrap())
    }

    #[test]
    fn test_expand_by_day() {
        let point_time = "20231023T180000Z".parse().unwrap();
        let set = RRuleSet {
            start_point_time: Some(point_time),
            rrule: vec![RRule {
                freq: Frequency::Daily,
                ..RRule::default()
            }],
            tz: Tz::UTC,
            limit: 10,
        };
        let dates = set.expand_by_day().unwrap();
        assert_eq!(dates.len(), 10);

        let first = dates.get(0).unwrap();
        assert_eq!(
            first,
            &PointTime {
                year: 2023,
                month: 10,
                day: 23,
                hour: 18,
                min: 0,
                sec: 0,
            }
        );

        let last = dates.get(9).unwrap();
        assert_eq!(
            last,
            &PointTime {
                month: 11,
                day: 1,
                ..set.start_point_time.unwrap()
            }
        )
    }
    #[test]
    fn test_expand_by_day_interval2() {
        let point_time = "20231023T180000Z".parse().unwrap();
        let set = RRuleSet {
            start_point_time: Some(point_time),
            rrule: vec![RRule {
                freq: Frequency::Daily,
                interval: 2,
                ..RRule::default()
            }],
            tz: Tz::UTC,
            limit: 5,
        };
        let dates = set.expand_by_day().unwrap();
        assert_eq!(dates.len(), 5);

        let first = dates.get(0).unwrap();
        assert_eq!(
            first,
            &PointTime {
                year: 2023,
                month: 10,
                day: 23,
                hour: 18,
                min: 0,
                sec: 0,
            }
        );

        let last = dates.get(4).unwrap();
        assert_eq!(
            last,
            &PointTime {
                month: 10,
                day: 31,
                ..set.start_point_time.unwrap()
            }
        )
    }

    #[test]
    fn test_rruleset() {
        let mut rrule_set = RRuleSet::from_str("RRULE:FREQ=WEEKLY;COUNT=3").unwrap();
        rrule_set.set_dt_start("20231001T180000");
        rrule_set.tz("America/New_York");
        rrule_set.limit = 10;
        let dates = rrule_set.all();
        assert_eq!(dates.len(), 10);
        println!(
            "{:?}",
            dates.iter().map(|d| d.to_string()).collect::<Vec<_>>()
        );
    }

    // #[test]
    // fn test_rruleset_by_str() {
    //     let rrule_set = RRuleSet::from_str("DTSTART:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;by_day=MO,TU,WE,TH,FR;UNTIL=20231121T235959");
    // }
}
