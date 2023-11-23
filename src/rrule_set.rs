use crate::point_time::PointTime;
use crate::rrule::weekday::NWeekday;
use crate::rrule::{get_tz_from_str, parse_dt_strart_str, RRule};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Weekday};
use chrono_tz::Tz;

const MAX_UNTIL_STR: &str = "20300101T000000Z";
#[derive(Debug)]
pub struct RRuleSet {
    rrule: Vec<RRule>,
    tz: Tz,
    start_point_time: Option<PointTime>,
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
            });
        }
        let rrule = RRule::from_str(lines[0]);
        Ok(RRuleSet {
            rrule: vec![rrule],
            tz: Tz::UTC,
            start_point_time: None,
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

    pub fn set_count(&mut self, count: u32) {
        if self.rrule.len() == 0 {
            return;
        }
        self.rrule.get_mut(0).unwrap().set_count(count);
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
        if rrule.count == 0 && rrule.until.is_none() {
            return Vec::new();
        }

        // 如果长度为0，并且开始时间大于截止时间，直接返回[]
        if rrule.count == 0 {
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
        let max = if rrule.count == 0 {
            65535
        } else {
            rrule.count as usize
        };
        let interval = rrule.interval;

        let curr =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();

        let binding = &vec![NWeekday::Every(curr.weekday())];
        let by_day = if rrule.by_day.len() == 0 {
            binding
        } else {
            &rrule.by_day
        };

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
        let end_time = rrule.until.as_ref().map_or_else(
            || NaiveDate::MAX,
            |until| NaiveDate::from_ymd_opt(until.year, until.month, until.day).unwrap(),
        );

        while next <= end_time && dates.len() < max {
            let weekday = next.weekday();
            if by_weekdays.contains(&weekday) {
                dates.push(generate_point_time(&next));
            }
            if weekday == rrule.week_start.pred() && interval != 1 {
                next = next + Duration::weeks((interval - 1).into()) + Duration::days(1);
            } else {
                next = next + Duration::days(1);
            }
        }

        Ok(dates)
    }
    /// 按天扩展，无效则报错
    fn expand_by_day(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();

        let rrule = self.rrule.get(0).unwrap();
        let limit = rrule.count;
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
    /// - 先判断是否有bymonthday，有则直接用，并且确认是否在byday中
    /// - 然后看有没有byday，迭代byday，如果是普通weekday，则取1..5来获取对应周数的时间，有则push到dates中，指定周数的日期，则指定处理
    /// 如果没有byday，则只需从开始时间起，按月累加即可，直到大于截止时间或者超出limit限制。

    fn expand_by_month(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();
        let rrule = self.rrule.get(0).unwrap();
        let limit = rrule.count;
        let interval = rrule.interval;
        let end_time = if let Some(until) = &rrule.until {
            until.clone()
        } else {
            MAX_UNTIL_STR.parse::<PointTime>().unwrap()
        };
        let max = if limit == 0 { 65535 } else { limit as usize };

        let naive_dt_start =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();
        let naive_end_time =
            NaiveDate::from_ymd_opt(end_time.year, end_time.month, end_time.day).unwrap();

        if rrule.by_month_day.len() == 0 && rrule.by_day.len() == 0 {
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

        // 闭包，判断加入的值是否符合条件
        // 1. vec没有超长
        // 2. 时间在指定区间内
        // 3. 如果有byday，则需匹配
        let add_to_dates = |naive: &NaiveDate, vec: &mut Vec<PointTime>| {
            if vec.len() > max {
                return;
            }
            if naive < &naive_dt_start {
                return;
            }
            if naive > &naive_end_time {
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
            // todo 缓存下两个序列取出的值 做一下交集
            // 并不关心by_month_day是否有值，默认为vec[]
            let vec_by_monthday = rrule
                .by_month_day
                .iter()
                .map(|n| {
                    if *n > 0 {
                        let naive =
                            NaiveDate::from_ymd_opt(curr_month.year, curr_month.month, *n as u32);
                        return naive;
                    }
                    let last = Self::get_last_day_of_month(curr_month.year, curr_month.month);
                    let last_day = last.day();
                    if last_day as i16 + n + 1 > 0 {
                        let curr = last.with_day((last_day as i16 + n + 1) as u32);
                        return curr;
                    }
                    return None;
                })
                .filter(|n| n.is_some())
                .map(|n| n.unwrap())
                .collect::<Vec<NaiveDate>>();

            let vec_by_day = rrule
                .by_day
                .iter()
                .map(|n_weekday| {
                    let mut vec: Vec<NaiveDate> = vec![];
                    match n_weekday {
                        NWeekday::Every(weekday) => {
                            for i in 1..5 {
                                let naive_date = NaiveDate::from_weekday_of_month_opt(
                                    curr_month.year,
                                    curr_month.month,
                                    *weekday,
                                    i,
                                );
                                if let Some(naive) = naive_date {
                                    vec.push(naive);
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
                                    vec.push(naive);
                                }
                            } else {
                                // 下面处理 n < 0 的场景
                                let first_day_of_month =
                                    NaiveDate::from_ymd_opt(curr_month.year, curr_month.month, 1)
                                        .unwrap();
                                let last_day_of_month =
                                    Self::get_last_day_of_month(curr_month.year, curr_month.month);
                                let mut date = last_day_of_month;

                                // 找到最后一个weekday
                                while date.weekday() != *weekday {
                                    date = date.pred_opt().unwrap();
                                }

                                let diff = *n + 1;
                                date = date + Duration::weeks(diff as i64);
                                if date >= first_day_of_month {
                                    vec.push(date);
                                }
                            }
                        }
                    }
                    return vec;
                })
                .flatten()
                .collect::<Vec<NaiveDate>>();

            if rrule.by_day.len() == 0 {
                vec_by_monthday.iter().for_each(|n| add_to_dates(&n, dates));
                return;
            }
            if rrule.by_month_day.len() == 0 {
                vec_by_day.iter().for_each(|n| add_to_dates(&n, dates));
                return;
            }

            let new_vec: Vec<NaiveDate> = vec_by_day
                .clone()
                .into_iter()
                .filter(|n| vec_by_monthday.contains(&n))
                .collect();
            new_vec.iter().for_each(|n| add_to_dates(&n, dates));
        };

        let mut curr_month = point_time.clone();
        let mut dates: Vec<PointTime> = vec![];

        while dates.len() < max && curr_month < end_time {
            generate_dates_in_month(&curr_month, &mut dates);
            curr_month = curr_month.add_month(interval);
        }
        let len = if dates.len() < max { dates.len() } else { max };
        dates = dates[0..len].to_vec();
        dates.sort();
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
                count: 10,
                ..RRule::default()
            }],
            tz: Tz::UTC,
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
                count: 5,
                ..RRule::default()
            }],
            tz: Tz::UTC,
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
        let test_vec = vec![
            (
                "DTSTART:20231123T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;",
                vec!["20231123T180000", "20231130T180000", "20231207T180000"],
            ),
            (
                "DTSTART:20231123T180000Z\nRRULE:FREQ=WEEKLY;COUNT=3;WKST=MO;BYDAY=WE",
                vec!["20231129T180000", "20231206T180000", "20231213T180000"],
            ),
        ];
        run_test_by_vec(test_vec);
    }

    #[test]
    fn test_expand_by_week_interval2() {
        let point_time = "20231023T180000Z".parse().unwrap();
        let set = RRuleSet {
            start_point_time: Some(point_time),
            rrule: vec![RRule::from_str("FREQ=WEEKLY;BYDAY=TU;INTERVAL=2;COUNT=2")],
            tz: Tz::UTC,
        };
        let dates = set.expand_by_week().unwrap();
        assert_eq!(dates.len(), 2);
        let first = dates.get(0).unwrap();
        assert_eq!(first, &"20231024T180000Z".parse().unwrap());
        let last = dates.get(1).unwrap();
        assert_eq!(last, &"20231107T180000Z".parse().unwrap())
    }

    #[test]
    fn test_expand_by_month() {
        let test_vec = vec![
            (
                "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;",
                vec!["20231029T091800", "20231129T091800", "20231229T091800"],
            ),
            (
                "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;INTERVAL=2",
                vec!["20231029T091800", "20231229T091800", "20240229T091800"],
            ),
            (
                "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1,3",
                vec!["20231101T091800", "20231103T091800", "20231201T091800"],
            ),
            (
                "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1,3;BYDAY=FR",
                vec!["20231103T091800", "20231201T091800", "20240301T091800"],
            ),
            (
                "DTSTART:20231029T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1;BYDAY=1FR",
                vec!["20231201T091800", "20240301T091800", "20241101T091800"],
            ),
            (
                "DTSTART:20231123T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYMONTHDAY=1;BYDAY=1FR;INTERVAL=2",
                vec!["20240301T091800", "20241101T091800", "20260501T091800"],
            ),
            (
                "DTSTART:20231123T091800Z\nRRULE:FREQ=MONTHLY;COUNT=3;WKST=MO;BYDAY=2FR;",
                vec!["20231208T091800", "20240112T091800", "20240209T091800"],
            ),
        ];
        run_test_by_vec(test_vec);
    }
    #[test]
    fn test_rruleset() {
        let mut rrule_set = RRuleSet::from_str("RRULE:FREQ=WEEKLY;COUNT=3").unwrap();
        rrule_set.set_dt_start("20231001T180000");
        rrule_set.tz("America/New_York");
        let dates = rrule_set.all();
        assert_eq!(dates.len(), 3);
    }

    #[test]
    fn test_rruleset_by_str() {
        let rrule_set = RRuleSet::from_str("DTSTART:20220506T180000Z\nRRULE:FREQ=WEEKLY;WKST=SU;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20231121T235959").unwrap();
        let dates = rrule_set.all();
        assert_eq!(dates.get(0).unwrap().to_string(), "2022-05-06 18:00:00 UTC");
        assert_eq!(dates.get(1).unwrap().to_string(), "2022-05-09 18:00:00 UTC");
        assert_eq!(dates.last().unwrap().to_string(), "2023-11-21 18:00:00 UTC");
    }

    fn run_test_by_vec(test_vec: Vec<(&str, Vec<&str>)>) {
        test_vec.iter().for_each(|(str, vec)| {
            assert_eq!(
                RRuleSet::from_str(str).unwrap().all(),
                vec.iter()
                    .map(|time| time.parse::<PointTime>().unwrap().with_timezone(&Tz::UTC))
                    .collect::<Vec<_>>()
            )
        });
    }
}
