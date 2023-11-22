use crate::point_time::PointTime;
use crate::rrule::weekday::NWeekday;
use crate::rrule::{get_tz_from_str, parse_dt_strart_str, RRule};
use chrono::{naive, DateTime, Datelike, Duration, NaiveDate, TimeZone, Weekday};
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
        let interval = rrule.interval;

        let curr =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();

        let binding = vec![NWeekday::Every(curr.weekday())];
        let by_day = rrule.by_day.as_deref().unwrap_or(&binding);

        let generate_point_time = |d: &NaiveDate| PointTime {
            year: d.year(),
            month: d.month(),
            day: d.day(),
            hour: point_time.hour,
            min: point_time.min,
            sec: point_time.sec,
        }; 

        let mut next = curr.clone();
        let mut dates: Vec<PointTime> = Vec::new();
        let by_weekdays: Vec<Weekday> = by_day.iter().map(|n| n.get_weekday().clone()).collect();

        match &rrule.until {
            Some(until) => {
                let end_time = NaiveDate::from_ymd_opt(until.year, until.month, until.day).unwrap();
                while next <= end_time {
                    let weekday = next.weekday();
                    println!("--->{}", next);
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
                while next <= end_time {
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
    /// - 先判断是否有bymonthday，有则直接用
    /// - 然后看有没有byday，迭代byday，如果是普通weekday，则取1..5来获取对应周数的时间，有则push到dates中，指定周数的日期，则指定处理
    /// 如果没有byday，则只需从开始时间起，按月累加即可，直到大于截止时间或者超出limit限制。

    fn expand_by_month(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();
        let rrule = self.rrule.get(0).unwrap();
        let limit = self.limit;
        let interval = rrule.interval;
        let until = rrule.until.clone();
        let max = if limit == 0 { 65535 } else { limit as usize };

        let naive_dt_start =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();
        let naive_end_time = until
            .clone()
            .and_then(|until| NaiveDate::from_ymd_opt(until.year, until.month, until.day));

        if rrule.by_month_day.len() == 0 || rrule.by_day.is_none() {
            let mut dates: Vec<PointTime> = Vec::new();
            let mut next = point_time.clone();

            match &rrule.until {
                Some(until) => {
                    while &next <= until && dates.len() < max {
                        dates.push(next.clone());
                        next = next.add_month(interval);
                    }
                }
                None => {
                    for _ in 0..max {
                        dates.push(next.clone());
                        next = next.add_month(interval);
                    }
                }
            };
            return Ok(dates);
        }

        let add_to_dates = |naive: &NaiveDate, vec: &mut Vec<PointTime>| {
            if vec.len() > max {
                return;
            }
            if naive < &naive_dt_start {
                return;
            }
            if naive_end_time.is_some() && naive > &naive_end_time.unwrap() {
                return;
            }
            vec.push(PointTime {
                year: naive.year(),
                month: naive.month(),
                day: naive.day(),
                hour: point_time.hour,
                min: point_time.min,
                sec: point_time.sec,
            })
        };

        let generate_dates_in_month = |curr_month: &PointTime, dates: &mut Vec<PointTime>| {
            // 并不关心by_month_day是否有值，默认为vec[]
            rrule.by_month_day.iter().for_each(|n| {
                if *n > 0 {
                    let naive =
                        NaiveDate::from_ymd_opt(curr_month.year, curr_month.month, *n as u32);
                    if let Some(naive) = naive {
                        add_to_dates(&naive, dates);
                    }
                    return;
                }
                let last = Self::get_last_day_of_month(curr_month.year, curr_month.month);
                let last_day = last.day();
                if last_day as i16 + n + 1 > 0 {
                    let curr = last.with_day((last_day as i16 + n + 1) as u32).unwrap();
                    add_to_dates(&curr, dates);
                }
            });

            if let Some(by_day) = rrule.by_day.as_ref() {
                by_day.iter().for_each(|n_weekday| match n_weekday {
                    NWeekday::Every(weekday) => {
                        for i in 1..5 {
                            let naive_date = NaiveDate::from_weekday_of_month_opt(
                                curr_month.year,
                                curr_month.month,
                                *weekday,
                                i,
                            );
                            if let Some(naive) = naive_date {
                                add_to_dates(&naive, dates);
                            }
                        }
                    }
                    NWeekday::Nth(n, weekday) => {
                        if *n > 0 {
                            let naive_date = NaiveDate::from_weekday_of_month_opt(
                                curr_month.year,
                                curr_month.month,
                                *weekday,
                                *n as u8,
                            );
                            if let Some(naive) = naive_date {
                                add_to_dates(&naive, dates);
                            }
                            return;
                        }
                        let first_day_of_month =
                            NaiveDate::from_ymd_opt(curr_month.year, curr_month.month, 1).unwrap();
                        let last_day_of_month =
                            Self::get_last_day_of_month(curr_month.year, curr_month.month);
                        let mut date = last_day_of_month;
                        // 找到最后一个周三
                        while date.weekday() != Weekday::Wed {
                            date = date.succ_opt().unwrap();
                        }

                        let diff = *n + 1;
                        date = date + Duration::weeks(diff as i64);
                        if date >= first_day_of_month {
                            add_to_dates(&date, dates)
                        }
                    }
                })
            }
        };

        let mut curr_month = point_time.clone();
        let mut dates: Vec<PointTime> = vec![];
        while dates.len() < max
            && if until.is_none() {
                true
            } else {
                until.as_ref().unwrap() >= &curr_month
            }
        {
            generate_dates_in_month(&curr_month, &mut dates);
            curr_month = curr_month.add_month(interval);
        }

        Ok(dates)
    }

    fn get_datetime_by_point_time(point_time: &PointTime, _tz: &Tz) -> DateTime<Tz> {
        Tz::UTC
            .with_ymd_and_hms(point_time.year, point_time.month, point_time.day, 12, 0, 0)
            .single()
            .unwrap()
    }

    /// 获取该月最后一天
    fn get_last_day_of_month(year: i32, month: u32) -> NaiveDate {
        let first_day_of_next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        };

        first_day_of_next_month.unwrap().pred_opt().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rrule::Frequency;
    use chrono_tz::Tz;

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
        assert_eq!(first, &"20231023T180000Z".parse().unwrap());

        let last = dates.get(9).unwrap();
        assert_eq!(last, &"20231101T180000Z".parse().unwrap())
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

    #[test]
    fn test_rruleset_by_str() {
        let rrule_set = RRuleSet::from_str("DTSTART:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959").unwrap();
        let dates = rrule_set.all();
        assert_eq!(dates.get(0).unwrap().to_string(), "2022-05-06 18:00:00 UTC");
        assert_eq!(dates.get(1).unwrap().to_string(), "2022-05-09 18:00:00 UTC");
        assert_eq!(dates.last().unwrap().to_string(), "2023-11-21 18:00:00 UTC");
    }
}
