use std::ops::Range;

use crate::point_time::PointTime;
use crate::rrule::weekday::{self, NWeekday};
use crate::rrule::{self, get_tz_from_str, parse_dt_strart_str, RRule};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Weekday};
use chrono_tz::{Tz, WET};

const MAX_UNTIL_STR: &str = "23000101T000000Z";
#[derive(Debug)]
pub struct RRuleSet {
    rrule: Vec<RRule>,
    tz: Tz,
    start_point_time: Option<PointTime>,
    max_until_time: PointTime,
}

impl RRuleSet {
    // 解析整个字符串，单行，不处理dt_start
    pub fn from_str(s: &str) -> Result<RRuleSet, String> {
        let lines: Vec<_> = s.split("\n").collect();
        let rrule: RRule;
        let mut start_point_time: Option<PointTime> = None;
        if lines.len() == 2 {
            start_point_time = Some(parse_dt_strart_str(lines[0])?);
            rrule = RRule::from_str(lines[1]);
        } else {
            rrule = RRule::from_str(lines[0]);
        }

        Ok(RRuleSet {
            rrule: vec![rrule],
            tz: Tz::UTC,
            start_point_time: start_point_time,
            max_until_time: MAX_UNTIL_STR.parse::<PointTime>().unwrap(),
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
            &until
        } else {
            &self.max_until_time
        };
        let max = if limit == 0 { 65535 } else { limit as usize };

        let naive_dt_start =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();
        let naive_end_time =
            NaiveDate::from_ymd_opt(end_time.year, end_time.month, end_time.day).unwrap();

        if rrule.by_month_day.is_empty() && rrule.by_day.is_empty() {
            let mut dates: Vec<PointTime> = Vec::new();
            let mut next: PointTime = point_time.clone();

            while &next <= end_time && dates.len() < max {
                dates.push(next.clone());
                next = next.add_month(interval);
            }

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

        let generate_dates_in_month = |curr: &PointTime, dates: &mut Vec<PointTime>| {
            // todo 缓存下两个序列取出的值 做一下交集
            // 并不关心by_month_day是否有值，默认为vec[]
            let vec_by_monthday = rrule
                .by_month_day
                .iter()
                .map(|n| Self::get_nth_day_of_month(curr.year, curr.month, *n))
                .filter(|n| n.is_some())
                .map(|n| n.unwrap())
                .collect::<Vec<NaiveDate>>();

            let vec_by_day = rrule
                .by_day
                .iter()
                .map(|n_weekday| {
                    Self::get_weekdays_by_nweekday_of_month(curr.year, curr.month, n_weekday)
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

        let mut curr = point_time.clone();
        let mut dates: Vec<PointTime> = vec![];

        while dates.len() < max && curr < *end_time {
            generate_dates_in_month(&curr, &mut dates);
            curr = curr.add_month(interval);
        }
        let len = if dates.len() < max { dates.len() } else { max };
        dates = dates[0..len].to_vec();
        dates.sort();
        Ok(dates)
    }

    fn expand_by_year(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();
        let rrule = self.rrule.get(0).unwrap();
        let max = if rrule.count == 0 {
            65535
        } else {
            rrule.count as usize
        };
        let interval = rrule.interval;
        let end_time = if let Some(until) = &rrule.until {
            &until
        } else {
            &self.max_until_time
        };

        let naive_dt_start =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();
        let naive_end_time =
            NaiveDate::from_ymd_opt(end_time.year, end_time.month, end_time.day).unwrap();

        let by_month = &rrule.by_month;

        let is_valid_time_by_naivedate = |date: &NaiveDate| {
            if date < &naive_dt_start || date > &naive_end_time {
                return false;
            }
            return true;
        };

        // 判断下是否符合起止时间
        let is_valid_time = |date: &Option<NaiveDate>| {
            if date.is_none() {
                return false;
            }
            return is_valid_time_by_naivedate(&date.unwrap());
        };

        //
        let generate_by_year = |curr_year: i32| -> Vec<NaiveDate> {
            // 只有BYMONTH的时候不会降级，还是执行按年重复，符合条件可以提前退出
            if rrule.by_year_day.is_empty()
                && rrule.by_month_day.is_empty()
                && rrule.by_week_no.is_empty()
                && rrule.by_day.is_empty()
            {
                let bindings = vec![point_time.month as u8];
                return (if rrule.by_month.is_empty() {
                    &bindings
                } else {
                    &rrule.by_month
                })
                .iter()
                .map(|month| {
                    NaiveDate::from_ymd_opt(curr_year, *month as u32, naive_dt_start.day())
                })
                .filter(|n| is_valid_time(n))
                .map(|n| n.unwrap())
                .collect::<Vec<NaiveDate>>();
            }

            // 先缓存符合一定条件的，再过滤不符合另一部分条件的
            let mut list: Vec<NaiveDate> = vec![];

            // 如果指定了yearday但是无结果，则直接返回，如果有，push到list里
            if !rrule.by_year_day.is_empty() {
                // 先生成所有的year_day
                let vec_by_year_day = rrule
                    .by_year_day
                    .iter()
                    .map(|day| Self::get_nth_day_of_year(curr_year, *day))
                    .filter(|n| is_valid_time(n))
                    .map(|n| n.unwrap())
                    .collect::<Vec<NaiveDate>>();

                if vec_by_year_day.is_empty() {
                    return vec![];
                }
                vec_by_year_day.into_iter().for_each(|n| list.push(n));
            };

            // 如果指定了bymonthday但是无结果，则直接返回
            // 如果有值，则判断与list是做交集还是直接塞入
            // 运算完后还是没有，则直接返回
            if !rrule.by_month_day.is_empty() {
                // 有则降级为月，但是与by_month可以直接取交集，性能可以接受，所以直接获取所有
                let vec_by_month_day = rrule
                    .by_month_day
                    .iter()
                    .map(|month_day| {
                        (1..12)
                            .filter(|month_day| {
                                if rrule.by_month.is_empty() {
                                    true
                                } else {
                                    rrule.by_month.contains(month_day)
                                }
                            })
                            .map(|i| Self::get_nth_day_of_month(curr_year, i as u32, *month_day))
                            .filter(|n| is_valid_time(n))
                            .map(|n| n.unwrap())
                            .collect::<Vec<NaiveDate>>()
                    })
                    .flatten()
                    .collect::<Vec<NaiveDate>>();

                if vec_by_month_day.is_empty() {
                    return vec![];
                }
                if !list.is_empty() {
                    list = list
                        .into_iter()
                        .filter(|n| vec_by_month_day.contains(&n))
                        .collect::<Vec<NaiveDate>>();
                } else {
                    vec_by_month_day.into_iter().for_each(|n| list.push(n));
                }
                if list.is_empty() {
                    return vec![];
                }
            };

            // - by_month和by_month_day结合会强制限定为某月某日，(同时无需在此时考虑by_day，因为后面还会按照by_day过滤)，
            //   但是该逻辑已经在上面处理过了，所以只需要考虑by_month_day不存在的场景
            // - 如果此时存在by_day，则会强制为该月所有周或指定周，类似于按月循环，可直接调用按月获取指定by_day的逻辑
            if rrule.by_month_day.is_empty() && !rrule.by_month.is_empty() {
                let vec_by_month = match rrule.by_day.is_empty() {
                    true => rrule
                        .by_month
                        .iter()
                        .map(|month| {
                            NaiveDate::from_ymd_opt(curr_year, *month as u32, point_time.day)
                        })
                        .filter(|n| is_valid_time(n))
                        .map(|n| n.unwrap())
                        .collect::<Vec<NaiveDate>>(),
                    false => rrule
                        .by_month
                        .iter()
                        .map(|month| {
                            return rrule
                                .by_day
                                .iter()
                                .map(|weekday| {
                                    Self::get_weekdays_by_nweekday_of_month(
                                        curr_year,
                                        *month as u32,
                                        weekday,
                                    )
                                })
                                .flatten()
                                .filter(|n| is_valid_time_by_naivedate(n))
                                .collect::<Vec<NaiveDate>>();
                        })
                        .flatten()
                        .filter(|n| is_valid_time_by_naivedate(n))
                        .collect::<Vec<NaiveDate>>(),
                };

                if !list.is_empty() {
                    list = list
                        .into_iter()
                        .filter(|n| vec_by_month.contains(&n))
                        .collect::<Vec<NaiveDate>>();
                } else {
                    vec_by_month.into_iter().for_each(|n| list.push(n));
                }
                if list.is_empty() {
                    return vec![];
                }
            }

            // week_no只能跟by_day交集
            if !rrule.by_week_no.is_empty() {
                let vec_by_week_no = rrule
                    .by_week_no
                    .iter()
                    .map(|week_no| {
                        return Self::get_all_weekday()
                            .into_iter()
                            .filter(|weekday| {
                                if rrule.by_day.is_empty() {
                                    true
                                } else {
                                    rrule.by_day.contains(&NWeekday::Every(*weekday))
                                }
                            })
                            .map(|weekday| {
                                // NaiveDate::from_isoywd_opt(curr_year, *week_no as u32, weekday)
                                if let Some(start) =
                                    Self::get_nth_week_by_week_no(curr_year, *week_no)
                                {
                                    return Some(Self::get_next_weekday_by_time(&start, &weekday));
                                }
                                None
                            })
                            .filter(|n| is_valid_time(n))
                            .map(|n| n.unwrap())
                            .collect::<Vec<NaiveDate>>();
                    })
                    .flatten()
                    .collect::<Vec<NaiveDate>>();

                if !list.is_empty() {
                    list = list
                        .into_iter()
                        .filter(|n| vec_by_week_no.contains(&n))
                        .collect::<Vec<NaiveDate>>();
                } else {
                    vec_by_week_no.into_iter().for_each(|n| list.push(n));
                }
                if list.is_empty() {
                    return vec![];
                }
            };

            // 此处不考虑every和nth同时存在的场景，

            // 如果是nth，则规则为获取，然后筛选
            if !rrule.by_day.is_empty() {
                // 如果是every，则规则为筛选
                let by_day_every = rrule
                    .by_day
                    .iter()
                    .filter(|n| n.is_every())
                    .map(|n| n.get_weekday().clone())
                    .collect::<Vec<Weekday>>();
                // 无值筛选，直接返回
                if list.is_empty() && !by_day_every.is_empty() {
                    return vec![];
                }

                list = list
                    .into_iter()
                    .filter(|n| by_day_every.contains(&n.weekday()))
                    .collect::<Vec<NaiveDate>>();

                // 筛选未通过，直接返回
                if list.is_empty() {
                    return vec![];
                }

                // 出于性能考虑，只处理by_day为nth的场景,不考虑Every
                let vec_by_day = rrule
                    .by_day
                    .iter()
                    .filter(|n| !n.is_every())
                    .map(|n_weekday| {
                        if rrule.by_month.is_empty() {
                            if let Some(time) =
                                Self::get_weekday_by_nweekday_of_year(curr_year, n_weekday)
                            {
                                return vec![time];
                            }
                            return vec![];
                        }
                        // 如果指定了月份，则找当月的指定日期
                        return rrule
                            .by_month
                            .iter()
                            .map(|month| {
                                return Self::get_nth_weekday_of_month(
                                    curr_year,
                                    *month as u32,
                                    n_weekday.get_weekday(),
                                    n_weekday.get_nth(),
                                );
                            })
                            .filter(|n| is_valid_time(n))
                            .map(|n| n.unwrap())
                            .collect::<Vec<NaiveDate>>();
                    })
                    .flatten()
                    .collect::<Vec<NaiveDate>>();

                if vec_by_day.is_empty() {
                    return vec![];
                }

                vec_by_day.into_iter().for_each(|n| list.push(n))
            }

            // 到这里肯定有值了

            list.sort();
            // todo by_set_pos

            return list;
        };

        let mut curr_year = point_time.year;
        let end_year = naive_end_time.year();
        let mut result: Vec<PointTime> = vec![];

        while result.len() < max && curr_year <= end_year {
            let rs = generate_by_year(curr_year);
            rs.into_iter().for_each(|n| {
                result.push(PointTime {
                    year: n.year(),
                    month: n.month(),
                    day: n.day(),
                    hour: point_time.hour,
                    min: point_time.min,
                    sec: point_time.sec,
                })
            });
            curr_year = curr_year + 1;
        }
        let len = if result.len() < max {
            result.len()
        } else {
            max
        };
        result = result[0..len].to_vec();
        Ok(result)

        // ByDay(Vec<NWeekday>) 过滤/获取, 有则降级为按周循环
        // ByMonthDay(Vec<i16>) 获取，已处理, 有则降级为每月几号
        // ByYearDay 获取，已处理,
        // ByWeekNo 获取，过滤,
        // ByMonth, 有则指定月
        // BySetPos,
        // 需要使用dt_start+limit 来限制一下，用于优化
    }

    fn get_datetime_by_point_time(point_time: &PointTime, _tz: &Tz) -> DateTime<Tz> {
        Tz::UTC
            .with_ymd_and_hms(point_time.year, point_time.month, point_time.day, 12, 0, 0)
            .single()
            .unwrap()
    }

    /// 获取某月第n天，支持正负
    fn get_nth_day_of_month(year: i32, month: u32, ordinal: i16) -> Option<NaiveDate> {
        if ordinal > 0 {
            return NaiveDate::from_ymd_opt(year, month, ordinal as u32);
        }
        if ordinal < 0 {
            let last = Self::get_last_day_of_month(year, month);
            let last_day = last.day();
            if last_day as i16 + ordinal + 1 > 0 {
                let curr = last.with_day((last_day as i16 + ordinal + 1) as u32);
                return curr;
            }
        }
        return None;
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
    /// 获取该年最后一天
    fn get_last_day_of_year(year: i32) -> NaiveDate {
        return NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
    }

    /// 获取每年第n天，支持正负数
    fn get_nth_day_of_year(year: i32, ordinal: i16) -> Option<NaiveDate> {
        if ordinal > 0 {
            return NaiveDate::from_yo_opt(year, ordinal as u32);
        }
        if ordinal < 0 {
            let first_day_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
            let last_day_of_year = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
            let last_nth_day_of_year = last_day_of_year - chrono::Duration::days(40);

            if last_nth_day_of_year >= first_day_of_year {
                return Some(last_nth_day_of_year);
            }
            return None;
        }
        return None;
    }

    /// 获取指定周数的周一
    fn get_nth_week_by_week_no(year: i32, week_no: i8) -> Option<NaiveDate> {
        if week_no > 0 {
            return NaiveDate::from_isoywd_opt(year, week_no as u32, Weekday::Mon);
        }
        if week_no < 0 {
            let week = -week_no as u32;
            let date = NaiveDate::from_ymd_opt(year, 12, 31).unwrap(); // 创建日期对象，表示当年的12月31日
            let weekday = date.weekday(); // 获取当天是星期几

            let days_until_last_week = match weekday {
                Weekday::Mon => 6, // 如果当天是星期一，则离最后一周还有6天
                Weekday::Tue => 5, // 如果当天是星期二，则离最后一周还有5天
                Weekday::Wed => 4, // 如果当天是星期三，则离最后一周还有4天
                Weekday::Thu => 3, // 如果当天是星期四，则离最后一周还有3天
                Weekday::Fri => 2, // 如果当天是星期五，则离最后一周还有2天
                Weekday::Sat => 1, // 如果当天是星期六，则离最后一周还有1天
                Weekday::Sun => 0, // 如果当天是星期日，则离最后一周还有0天
            };

            let last_week_start = date - Duration::days(days_until_last_week as i64); // 计算最后一周的开始日期
            let last_week_end = date; // 最后一周的结束日期即为当天

            let week_diff = last_week_end.iso_week().week() + week; // 计算要获取的周数与最后一周的周数差

            if week_diff >= 0 {
                return Some(last_week_start - Duration::weeks(week_diff as i64));
            // 返回指定周数的开始日期
            } else {
                return None; // 如果指定的周数超过了最后一周的周数，则返回None
            }
        }
        return None;
    }

    /// 给定一周的开始，返回指定星期的日期
    fn get_next_weekday_by_time(date: &NaiveDate, weekday: &Weekday) -> NaiveDate {
        let diff = match weekday {
            Weekday::Mon => 0, // 如果当天是星期一，则离最后一周还有6天
            Weekday::Tue => 1, // 如果当天是星期二，则离最后一周还有5天
            Weekday::Wed => 2, // 如果当天是星期三，则离最后一周还有4天
            Weekday::Thu => 3, // 如果当天是星期四，则离最后一周还有3天
            Weekday::Fri => 4, // 如果当天是星期五，则离最后一周还有2天
            Weekday::Sat => 5, // 如果当天是星期六，则离最后一周还有1天
            Weekday::Sun => 6, // 如果当天是星期日，则离最后一周还有0天
        };
        *date + Duration::days(diff)
    }

    /// 获取一周7天的数据
    fn get_all_weekday() -> Vec<Weekday> {
        return vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
    }

    /// 获取指定月份所有指定的星期
    fn get_all_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> Vec<NaiveDate> {
        (1..5)
            .map(|n| NaiveDate::from_weekday_of_month_opt(year, month, weekday, n))
            .filter(|n| n.is_some())
            .map(|n| n.unwrap())
            .collect::<Vec<NaiveDate>>()
    }

    /// 获取指定月份下第n个周的指定星期几
    fn get_nth_weekday_of_month(
        year: i32,
        month: u32,
        weekday: &Weekday,
        n: i16,
    ) -> Option<NaiveDate> {
        if n > 0 {
            return NaiveDate::from_weekday_of_month_opt(year, month, *weekday, n as u8);
        }
        // 下面处理 n < 0 的场景
        let first_day_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let last_day_of_month = Self::get_last_day_of_month(year, month);
        let mut date = last_day_of_month;

        // 找到最后一个weekday
        while date.weekday() != *weekday {
            date = date.pred_opt().unwrap();
        }

        let diff = n + 1;
        date = date + Duration::weeks(diff as i64);
        if date >= first_day_of_month {
            return Some(date);
        }
        return None;
    }

    /// 获取指定月份下，第n_weekday的时间列表
    fn get_weekdays_by_nweekday_of_month(
        year: i32,
        month: u32,
        n_weekday: &NWeekday,
    ) -> Vec<NaiveDate> {
        let mut vec: Vec<NaiveDate> = vec![];
        match n_weekday {
            NWeekday::Every(weekday) => Self::get_all_weekday_of_month(year, month, *weekday)
                .into_iter()
                .for_each(|n| vec.push(n)),
            NWeekday::Nth(n, weekday) => {
                if let Some(time) = Self::get_nth_weekday_of_month(year, month, weekday, *n) {
                    vec.push(time)
                }
            }
        };
        return vec;
    }

    /// 获取每年指定nth的星期，不处理Every场景
    fn get_weekday_by_nweekday_of_year(year: i32, n_weekday: &NWeekday) -> Option<NaiveDate> {
        let n = n_weekday.get_nth();
        let weekday = n_weekday.get_weekday();
        let first_day_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let last_day_of_year = Self::get_last_day_of_year(year);
        if n > 0 {
            let next_weekday = Self::get_next_weekday_by_time(&first_day_of_year, weekday)
                + Duration::weeks((n - 1) as i64);
            if next_weekday > last_day_of_year {
                return None;
            }
            return Some(next_weekday);
        }
        if n < 0 {
            let mut date = last_day_of_year;

            // 找到最后一个weekday
            while date.weekday() != *weekday {
                date = date.pred_opt().unwrap();
            }

            let diff = n + 1;
            date = date + Duration::weeks(diff as i64);
            if date >= first_day_of_year {
                return Some(date);
            }
            return None;
        }
        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rrule::Frequency;
    use chrono_tz::Tz;

    #[test]
    fn test_expand_by_day() {
        let set: RRuleSet =
            RRuleSet::from_str("DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=10").unwrap();

        let dates = set.expand_by_day().unwrap();
        assert_eq!(dates.len(), 10);

        let first = dates.get(0).unwrap();
        assert_eq!(first, &"20231023T180000Z".parse().unwrap());

        let last = dates.get(9).unwrap();
        assert_eq!(last, &"20231101T180000Z".parse().unwrap())
    }
    #[test]
    fn test_expand_by_day_interval2() {
        let set =
            RRuleSet::from_str("DTSTART:20231023T180000Z\nRRULE:FREQ=DAILY;COUNT=5;INTERVAL=2")
                .unwrap();

        let dates = set.expand_by_day().unwrap();
        assert_eq!(dates.len(), 5);

        let first = dates.get(0).unwrap();
        assert_eq!(first, &"20231023T180000".parse::<PointTime>().unwrap());

        let last = dates.get(4).unwrap();
        assert_eq!(last, &"20231031T180000".parse::<PointTime>().unwrap());
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
        let set = RRuleSet::from_str(
            "DTSTART:20231023T180000Z\nRRULE:FREQ=WEEKLY;BYDAY=TU;INTERVAL=2;COUNT=2",
        )
        .unwrap();

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
