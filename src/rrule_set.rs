use crate::point_time::PointTime;
use crate::rrule::weekday::NWeekday;
use crate::rrule::{get_tz_from_str, parse_dt_strart_str_and_tz, RRule};
use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, Weekday};
use chrono_tz::Tz;

const MAX_UNTIL_STR: &str = "23000101T000000Z";
#[derive(Debug)]
pub struct RRuleSet {
    pub rrule: Vec<RRule>,
    pub tz: Tz,
    start_point_time: Option<PointTime>,
    max_until_time: PointTime,
}

impl RRuleSet {
    // 解析整个字符串，单行，不处理dt_start
    pub fn from_str(s: &str) -> Result<RRuleSet, String> {
        let lines: Vec<_> = s.split("\n").collect();
        let rrule: RRule;
        let mut tz = Tz::UTC;
        let mut start_point_time: Option<PointTime> = None;
        if lines.len() == 2 {
            let (start, tz2) = parse_dt_strart_str_and_tz(lines[0])?;
            start_point_time = Some(start);
            if tz2.is_some() {
                tz = tz2.unwrap();
            }
            rrule = RRule::from_str(lines[1]);
        } else {
            rrule = RRule::from_str(lines[0]);
        }

        Ok(RRuleSet {
            rrule: vec![rrule],
            tz,
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

    pub fn set_until(&mut self, str: &str) {
        if self.rrule.len() == 0 {
            return;
        }
        self.rrule.get_mut(0).unwrap().set_until(str);
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

        // todo 提前排除下week_no与by_month\by_year_day的交集是否有效

        match rrule.freq {
            crate::rrule::Frequency::Yearly => self
                .expand_by_year()
                .unwrap()
                .into_iter()
                .map(|p| p.with_timezone(&self.tz))
                .collect(),
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

    /// 按天扩展，无效则报错
    fn expand_by_day(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();
        let rrule = self.rrule.get(0).unwrap();
        let interval = rrule.interval;
        let end_time = if let Some(until) = &rrule.until {
            &until
        } else {
            &self.max_until_time
        };
        let max = if rrule.count == 0 {
            65535
        } else {
            rrule.count as usize
        };
        let by_day_every = rrule
            .by_day
            .iter()
            .map(|n| n.get_weekday().clone())
            .collect::<Vec<Weekday>>();
        let naive_dt_start =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();
        let naive_end_time =
            NaiveDate::from_ymd_opt(end_time.year, end_time.month, end_time.day).unwrap();

        let mut next = naive_dt_start.clone();
        let mut list: Vec<NaiveDate> = Vec::new();

        let go_step = |time: NaiveDate| {
            return time + Duration::days(interval.into());
        };

        while next <= naive_end_time && list.len() < max {
            if !by_day_every.is_empty() && !by_day_every.contains(&next.weekday()) {
                next = go_step(next);
                continue;
            }
            if !rrule.by_month.is_empty() && !rrule.by_month.contains(&(next.month() as u8)) {
                next = go_step(next);
                continue;
            }
            if !rrule.by_month_day.is_empty() {
                let mut flag = false;
                for day in rrule.by_month_day.iter() {
                    if Self::is_nth_day_of_month(&next, *day) {
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    next = go_step(next);
                    continue;
                }
            }
            if !rrule.by_year_day.is_empty() {
                let mut flag = false;
                for day in rrule.by_year_day.iter() {
                    if Self::is_nth_day_of_year(&next, *day) {
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    next = go_step(next);
                    continue;
                }
            }
            if !rrule.by_week_no.is_empty() {
                let mut flag = false;
                for day in rrule.by_week_no.iter() {
                    if Self::is_in_nth_weekno(&next, *day) {
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    next = go_step(next);
                    continue;
                }
            }
            list.push(next.clone());
            next = go_step(next);
        }

        Ok(list
            .into_iter()
            .map(|n| PointTime {
                year: n.year(),
                month: n.month(),
                day: n.day(),
                hour: point_time.hour,
                min: point_time.min,
                sec: point_time.sec,
            })
            .collect::<Vec<PointTime>>())
    }

    /// 按周扩展，无效则报错
    fn expand_by_week(&self) -> Result<Vec<PointTime>, String> {
        let point_time = self.start_point_time.as_ref().unwrap();
        let rrule = self.rrule.get(0).unwrap();
        let interval = rrule.interval;
        let end_time = if let Some(until) = &rrule.until {
            &until
        } else {
            &self.max_until_time
        };
        let max = if rrule.count == 0 {
            65535
        } else {
            rrule.count as usize
        };

        let naive_dt_start =
            NaiveDate::from_ymd_opt(point_time.year, point_time.month, point_time.day).unwrap();
        let naive_end_time =
            NaiveDate::from_ymd_opt(end_time.year, end_time.month, end_time.day).unwrap();
        let dt_start_weekday = naive_dt_start.weekday();

        let binding = vec![dt_start_weekday.clone()];
        let by_day_every = if rrule.by_day.len() == 0 {
            binding
        } else {
            rrule
                .by_day
                .iter()
                .map(|n| n.get_weekday().clone())
                .collect::<Vec<Weekday>>()
        };

        let mut next = naive_dt_start.clone();
        let mut list: Vec<NaiveDate> = Vec::new();

        let go_step = |time: NaiveDate| {
            let weekday = time.weekday();
            return if weekday == rrule.week_start.pred() && interval != 1 {
                time + Duration::weeks((interval - 1).into()) + Duration::days(1)
            } else {
                time + Duration::days(1)
            };
        };

        while next <= naive_end_time && list.len() < max {
            let weekday = next.weekday();
            if !by_day_every.contains(&weekday) {
                next = go_step(next);
                continue;
            }
            if !rrule.by_month.is_empty() && !rrule.by_month.contains(&(next.month() as u8)) {
                next = go_step(next);
                continue;
            }
            if !rrule.by_month_day.is_empty() {
                let mut flag = false;
                for day in rrule.by_month_day.iter() {
                    if Self::is_nth_day_of_month(&next, *day) {
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    next = go_step(next);
                    continue;
                }
            }
            if !rrule.by_year_day.is_empty() {
                let mut flag = false;
                for day in rrule.by_year_day.iter() {
                    if Self::is_nth_day_of_year(&next, *day) {
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    next = go_step(next);
                    continue;
                }
            }
            if !rrule.by_week_no.is_empty() {
                let mut flag = false;
                for day in rrule.by_week_no.iter() {
                    if Self::is_in_nth_weekno(&next, *day) {
                        flag = true;
                        break;
                    }
                }
                if !flag {
                    next = go_step(next);
                    continue;
                }
            }
            list.push(next.clone());
            next = go_step(next);
        }

        Ok(list
            .into_iter()
            .map(|n| PointTime {
                year: n.year(),
                month: n.month(),
                day: n.day(),
                hour: point_time.hour,
                min: point_time.min,
                sec: point_time.sec,
            })
            .collect::<Vec<PointTime>>())
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

        // 存粹按月循环
        if rrule.by_month_day.is_empty()
            && rrule.by_year_day.is_empty()
            && rrule.by_month_day.is_empty()
            && rrule.by_week_no.is_empty()
            && rrule.by_day.is_empty()
        {
            let mut dates: Vec<PointTime> = Vec::new();
            let mut next: PointTime = point_time.clone();

            while &next <= end_time && dates.len() < max {
                dates.push(next.clone());
                next = next.add_month(interval);
            }

            return Ok(dates);
        }

        let generate_dates_in_month = |curr: &NaiveDate| -> Vec<NaiveDate> {
            // 不符合月份直接打回
            if !rrule.by_month.is_empty() && rrule.by_month.contains(&(curr.month() as u8)) {
                return vec![];
            }

            // 先缓存符合一定条件的，再过滤不符合另一部分条件的
            let mut list: Vec<NaiveDate> = vec![];
            let curr_month_start = NaiveDate::from_ymd_opt(curr.year(), curr.month(), 1).unwrap();
            let curr_month_end = Self::get_last_day_of_month(curr.year(), curr.month());

            let valid_start = if curr_month_start > naive_dt_start {
                &curr_month_start
            } else {
                &naive_dt_start
            };
            let valid_end = if curr_month_end > naive_end_time {
                &naive_end_time
            } else {
                &curr_month_end
            };

            // 如果指定了yearday但是无结果，则直接返回，如果有，push到list里
            if !rrule.by_year_day.is_empty() {
                // 先生成所有的year_day
                let vec_by_year_day = rrule
                    .by_year_day
                    .iter()
                    .map(|day| Self::get_nth_day_of_year(curr.year(), *day))
                    .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
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
                let vec_by_month_day = rrule
                    .by_month_day
                    .iter()
                    .map(|month_day| {
                        Self::get_nth_day_of_month(curr.year(), curr.month(), *month_day)
                    })
                    .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
                    .map(|n| n.unwrap())
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

            // week_no只能跟by_day交集
            if !rrule.by_week_no.is_empty() {
                let vec_by_week_no = rrule
                    .by_week_no
                    .iter()
                    .map(|week_no| {
                        return Self::get_all_weekday()
                            .into_iter()
                            .map(|_| Self::get_nth_week_by_week_no(curr.year(), *week_no))
                            .filter(|n| n.is_some())
                            .map(|n| {
                                let first = &n.unwrap();
                                return (0..6)
                                    .map(|num| *first + Duration::days(num))
                                    .collect::<Vec<NaiveDate>>();
                            })
                            .flatten()
                            .filter(|n| Self::is_date_in_range(valid_start, valid_end, n))
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

            if !rrule.by_day.is_empty() {
                let vec_by_day = rrule
                    .by_day
                    .iter()
                    .map(|n_weekday| match n_weekday {
                        NWeekday::Every(weekday) => {
                            Self::get_all_weekday_of_month(curr.year(), curr.month(), weekday)
                        }
                        NWeekday::Nth(_, _) => vec![Self::get_nth_weekday_of_month(
                            curr.year(),
                            curr.month(),
                            n_weekday.get_weekday(),
                            n_weekday.get_nth(),
                        )]
                        .iter()
                        .filter(|n| n.is_some())
                        .map(|n| n.unwrap())
                        .collect::<Vec<NaiveDate>>(),
                    })
                    .flatten()
                    .filter(|n| Self::is_date_in_range(valid_start, valid_end, n))
                    .collect::<Vec<NaiveDate>>();

                if !list.is_empty() {
                    list = list
                        .into_iter()
                        .filter(|n| vec_by_day.contains(&n))
                        .collect::<Vec<NaiveDate>>();
                } else {
                    vec_by_day.into_iter().for_each(|n| list.push(n));
                }
                if list.is_empty() {
                    return vec![];
                }
            }
            list.sort();
            return list;
        };

        let mut next =
            NaiveDate::from_ymd_opt(naive_dt_start.year(), naive_dt_start.month(), 1).unwrap();
        let mut list: Vec<PointTime> = vec![];

        while list.len() < max && next < naive_end_time {
            let rs = generate_dates_in_month(&next);
            rs.into_iter().for_each(|n| {
                list.push(PointTime {
                    year: n.year(),
                    month: n.month(),
                    day: n.day(),
                    hour: point_time.hour,
                    min: point_time.min,
                    sec: point_time.sec,
                })
            });
            next = next.checked_add_months(Months::new(interval)).unwrap();
        }
        let len = if list.len() < max { list.len() } else { max };
        list = list[0..len].to_vec();
        list.sort();

        Ok(list)
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

        //
        let generate_by_year = |curr_year: i32| -> Vec<NaiveDate> {
            let curr_year_start = NaiveDate::from_ymd_opt(curr_year, 1, 1).unwrap();
            let curr_year_end = Self::get_last_day_of_year(curr_year);

            let valid_start = if curr_year_start > naive_dt_start {
                &curr_year_start
            } else {
                &naive_dt_start
            };
            let valid_end = if curr_year_end > naive_end_time {
                &curr_year_end
            } else {
                &naive_end_time
            };

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
                .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
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
                    .filter(|n| Self::is_option_date_in_range(&naive_dt_start, &naive_end_time, n))
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
                            .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
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
            if !rrule.by_month.is_empty() {
                // let vec_by_month = match rrule.by_day.is_empty() {
                //     true => rrule
                //         .by_month
                //         .iter()
                //         .map(|month| {
                //             NaiveDate::from_ymd_opt(curr_year, *month as u32, point_time.day)
                //         })
                //         .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
                //         .map(|n| n.unwrap())
                //         .collect::<Vec<NaiveDate>>(),
                //     false => rrule
                //         .by_month
                //         .iter()
                //         .map(|month| {
                //             return rrule
                //                 .by_day
                //                 .iter()
                //                 .map(|weekday| {
                //                     Self::get_weekdays_by_nweekday_of_month(
                //                         curr_year,
                //                         *month as u32,
                //                         weekday,
                //                     )
                //                 })
                //                 .flatten()
                //                 .filter(|n| Self::is_date_in_range(valid_start, valid_end, n))
                //                 .collect::<Vec<NaiveDate>>();
                //         })
                //         .flatten()
                //         .filter(|n| Self::is_date_in_range(valid_start, valid_end, n))
                //         .collect::<Vec<NaiveDate>>(),
                // };

                // if !list.is_empty() {
                //     list = list
                //         .into_iter()
                //         .filter(|n| vec_by_month.contains(&n))
                //         .collect::<Vec<NaiveDate>>();
                // } else {
                //     vec_by_month.into_iter().for_each(|n| list.push(n));
                // }
                // if list.is_empty() {
                //     return vec![];
                // }

                if list.len() != 0 {
                    list = list
                        .into_iter()
                        .filter(|n| rrule.by_month.contains(&(n.month() as u8)))
                        .collect::<Vec<NaiveDate>>();
                    if list.is_empty() {
                        return vec![];
                    }
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
                            .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
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

            if !rrule.by_day.is_empty() {
                let by_day_every = rrule
                    .by_day
                    .iter()
                    .filter(|n| n.is_every())
                    .map(|n| n.get_weekday().clone())
                    .collect::<Vec<Weekday>>();
                let by_day_nth = rrule
                    .by_day
                    .iter()
                    .filter(|n| !n.is_every())
                    .map(|n| n.get_weekday().clone())
                    .collect::<Vec<Weekday>>();

                // 不支持混排，不知道为啥
                if !by_day_nth.is_empty() && !by_day_every.is_empty() {
                    return vec![];
                }

                // 如果此时有值，先过滤一次every的
                if !list.is_empty() && !by_day_every.is_empty() {
                    list = list
                        .into_iter()
                        .filter(|n| by_day_every.contains(&n.weekday()))
                        .collect::<Vec<NaiveDate>>();
                    if list.is_empty() {
                        return vec![];
                    }
                }

                // 只处理by_day为every的场景,不考虑nth
                let vec_by_day_every = rrule
                    .by_day
                    .iter()
                    .filter(|n| n.is_every())
                    .map(|n_weekday| {
                        if rrule.by_month.is_empty() {
                            return (1..53)
                                .map(|n| {
                                    NaiveDate::from_isoywd_opt(
                                        curr_year,
                                        n,
                                        *n_weekday.get_weekday(),
                                    )
                                })
                                .filter(|n| {
                                    Self::is_option_date_in_range(valid_start, valid_end, n)
                                })
                                .map(|n| n.unwrap())
                                .collect::<Vec<NaiveDate>>();
                        }
                        // 如果指定了月份，则找当月的指定日期
                        return rrule
                            .by_month
                            .iter()
                            .map(|month| {
                                return Self::get_all_weekday_of_month(
                                    curr_year,
                                    *month as u32,
                                    n_weekday.get_weekday(),
                                )
                                .into_iter()
                                .filter(|n| Self::is_date_in_range(valid_start, valid_end, n))
                                .collect::<Vec<NaiveDate>>();
                            })
                            .flatten()
                            .collect::<Vec<NaiveDate>>();
                    })
                    .flatten()
                    .collect::<Vec<NaiveDate>>();

                // 只处理by_day为nth的场景,不考虑Every
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
                            .filter(|n| Self::is_option_date_in_range(valid_start, valid_end, n))
                            .map(|n| n.unwrap())
                            .collect::<Vec<NaiveDate>>();
                    })
                    .flatten()
                    .collect::<Vec<NaiveDate>>();

                let all_by_day = [vec_by_day, vec_by_day_every].concat();
                if !list.is_empty() {
                    list = list
                        .into_iter()
                        .filter(|n| all_by_day.contains(&n))
                        .collect::<Vec<NaiveDate>>();
                } else {
                    all_by_day.into_iter().for_each(|n| list.push(n));
                }
                if list.is_empty() {
                    return vec![];
                }
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
            curr_year = curr_year + interval as i32;
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
            let last_nth_day_of_year =
                last_day_of_year + chrono::Duration::days((ordinal + 1) as i64);

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
            // let week = -week_no as u32;
            let date = NaiveDate::from_ymd_opt(year, 12, 31).unwrap(); // 创建日期对象，表示当年的12月31日
            let iso_week = date.iso_week().week();
            let weekday = date.weekday(); // 获取当天是星期几
            let days_until_last_week = match weekday {
                Weekday::Mon => 0,
                Weekday::Tue => 1,
                Weekday::Wed => 2,
                Weekday::Thu => 3,
                Weekday::Fri => 4,
                Weekday::Sat => 5,
                Weekday::Sun => 6,
            };

            let last_week_start = if iso_week == 1 {
                date - Duration::days(days_until_last_week + 7)
            } else {
                date - Duration::days(days_until_last_week as i64)
            };

            let find_week_start = last_week_start + Duration::weeks(week_no as i64 + 1);

            if find_week_start >= NaiveDate::from_ymd_opt(year, 1, 1).unwrap() {
                return Some(find_week_start);
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
    fn get_all_weekday_of_month(year: i32, month: u32, weekday: &Weekday) -> Vec<NaiveDate> {
        (1..6)
            .map(|n| NaiveDate::from_weekday_of_month_opt(year, month, *weekday, n))
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
            NWeekday::Every(weekday) => Self::get_all_weekday_of_month(year, month, weekday)
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

    /// 判断给定的时间是否是指定的某个月中的一天
    fn is_nth_day_of_month(time: &NaiveDate, day: i16) -> bool {
        if day > 0 {
            return time.day() == day as u32;
        }
        if day < 0 {
            let last_day_of_month = Self::get_last_day_of_month(time.year(), time.month());
            let last_day = last_day_of_month.day() as i16;
            if day > -31 && (last_day + day + 1) == time.day() as i16 {
                return true;
            }

            return false;
        }
        return false;
    }

    /// 判断给定的时间是否是指定的某年中的一天
    fn is_nth_day_of_year(time: &NaiveDate, day: i16) -> bool {
        if let Some(rs) = Self::get_nth_day_of_year(time.year(), day) {
            return &rs == time;
        }
        return false;
    }

    /// 判断给定的时间是否是指定的某周中的一天
    fn is_in_nth_weekno(time: &NaiveDate, week_no: i8) -> bool {
        if week_no == 0 {
            return false;
        }
        let iso_week_no = time.iso_week().week();
        if week_no > 0 {
            return iso_week_no == week_no as u32;
        }
        // 特殊处理下第二年但是属于上年最后一周的场景
        if week_no == -1 && iso_week_no > 51 && time.month() == 1 {
            if let Some(rs) = Self::get_nth_week_by_week_no(time.year() - 1, week_no) {
                return time >= &rs && (*time + Duration::days(7)) > rs;
            }
            return false;
        }
        if let Some(rs) = Self::get_nth_week_by_week_no(time.year(), week_no) {
            return time >= &rs && time < &(rs + Duration::weeks(1));
        }
        return false;
    }

    // 给定的时间是否在范围内
    fn is_date_in_range(start: &NaiveDate, end: &NaiveDate, curr: &NaiveDate) -> bool {
        if curr < start || curr > &end {
            return false;
        }
        return true;
    }

    fn is_option_date_in_range(
        start: &NaiveDate,
        end: &NaiveDate,
        curr: &Option<NaiveDate>,
    ) -> bool {
        if curr.is_none() {
            return false;
        }
        return Self::is_date_in_range(start, end, &curr.unwrap());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vec_contains() {
        assert_eq!(
            vec![Weekday::Mon, Weekday::Thu].contains(&Weekday::Mon),
            true
        );
        assert_eq!(
            "1".split(",")
                .map(|s| s.parse::<u8>())
                .filter(|s| s.is_ok())
                .map(|s| s.unwrap())
                .collect::<Vec<_>>(),
            vec![1]
        )
    }

    #[test]
    fn test_is_nth_day_of_month() {
        assert_eq!(
            RRuleSet::is_nth_day_of_month(&NaiveDate::from_ymd_opt(2023, 10, 31).unwrap(), -1),
            true
        );
        assert_eq!(
            RRuleSet::is_nth_day_of_month(&NaiveDate::from_ymd_opt(2023, 10, 30).unwrap(), -1),
            false
        );
        assert_eq!(
            RRuleSet::is_nth_day_of_month(&NaiveDate::from_ymd_opt(2023, 10, 30).unwrap(), -100),
            false
        );
        assert_eq!(
            RRuleSet::is_nth_day_of_month(&NaiveDate::from_ymd_opt(2023, 10, 15).unwrap(), 15),
            true
        );
        assert_eq!(
            RRuleSet::is_nth_day_of_month(&NaiveDate::from_ymd_opt(2023, 10, 30).unwrap(), -2),
            true
        );
    }

    #[test]
    fn test_is_nth_day_of_year() {
        assert_eq!(
            RRuleSet::is_nth_day_of_year(&NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(), -1),
            true
        );
        assert_eq!(
            RRuleSet::is_nth_day_of_year(&NaiveDate::from_ymd_opt(2023, 12, 30).unwrap(), -2),
            true
        );
        assert_eq!(
            RRuleSet::is_nth_day_of_year(&NaiveDate::from_ymd_opt(2023, 10, 31).unwrap(), -1),
            false
        );
    }

    #[test]
    fn test_get_nth_week_by_week_no() {
        assert_eq!(
            RRuleSet::get_nth_week_by_week_no(2023, -1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
        );
    }
    #[test]
    fn test_is_in_nth_weekno() {
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(), -1),
            true
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2021, 1, 3).unwrap(), -1),
            true
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2021, 1, 4).unwrap(), -1),
            false
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2020, 12, 27).unwrap(), -1),
            false
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2020, 5, 11).unwrap(), 20),
            true
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2020, 5, 10).unwrap(), 20),
            false
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2023, 10, 30).unwrap(), -1),
            false
        );
        assert_eq!(
            RRuleSet::is_in_nth_weekno(&NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), -1),
            true
        );
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
}
