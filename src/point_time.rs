use chrono::{DateTime, NaiveDate, TimeZone};
use chrono_tz::Tz;
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct PointTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub min: u32,
    pub sec: u32,
}

impl FromStr for PointTime {
    type Err = String;
    /// Parses a date string with format `YYYYMMDDTHHMMSSZ`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let year: i32 = s[0..4].parse().map_err(|_err| "invalid string")?;
        let month: u32 = s[4..6]
            .to_string()
            .parse()
            .map_err(|_err| "invalid month")?;
        let day: u32 = s[6..8].to_string().parse().map_err(|_err| "invalid day")?;

        let t = s[8..9].eq("T");
        if !t {
            return Err("invalid string format".to_string());
        }

        let hour: u32 = s[9..11]
            .to_string()
            .parse()
            .map_err(|_err| "invalid hour")?;
        let min: u32 = s[11..13]
            .to_string()
            .parse()
            .map_err(|_err| "invalid minute")?;
        let sec: u32 = s[13..15]
            .to_string()
            .parse()
            .map_err(|_err| "invalid second")?;

        Ok(PointTime {
            year,
            month,
            day,
            hour,
            min,
            sec,
        })
    }
}

impl PartialOrd for PointTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.year != other.year {
            return Some(if self.year > other.year {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if self.month != other.month {
            return Some(if self.month > other.month {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if self.day != other.day {
            return Some(if self.day > other.day {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if self.hour != other.hour {
            return Some(if self.hour > other.hour {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if self.min != other.min {
            return Some(if self.min > other.min {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if self.sec != other.sec {
            return Some(if self.sec > other.sec {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }

        return Some(Ordering::Equal);
    }
}

impl Ord for PointTime {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.partial_cmp(other).unwrap();
    }
}

impl PointTime {
    pub fn with_timezone(&self, tz: &Tz) -> DateTime<Tz> {
        let rs = tz
            .with_ymd_and_hms(
                self.year, self.month, self.day, self.hour, self.min, self.sec,
            )
            .single()
            .unwrap();
        rs
    }

    pub fn add_month(&mut self, mon: u32) -> Self {
        let mut next_point_time = self.clone();

        loop {
            next_point_time.month = next_point_time.month + mon;
            if next_point_time.month > 12 {
                next_point_time.month -= 12;
                next_point_time.year += 1;
            };

            if next_point_time.is_valid() {
                break;
            }
        }
        next_point_time
    }

    /// 判断时间节点是否有效
    pub fn is_valid(&self) -> bool {
        let date_time = NaiveDate::from_ymd_opt(self.year, self.month, self.day);
        match date_time {
            Some(_) => true,
            _ => false,
        }
    }

    pub fn get_max_time<'a>(p1: &'a PointTime, p2: &'a PointTime) -> &'a PointTime {
        if p1 > p2 {
            return p1;
        }
        return p2;
    }
    pub fn get_min_time<'a>(p1: &'a PointTime, p2: &'a PointTime) -> &'a PointTime {
        if p1 > p2 {
            return p2;
        }
        return p1;
    }
}

#[cfg(test)]
mod test {
    use super::PointTime;

    #[test]
    fn parse_timestr() {
        let s = String::from("20231115T191020");
        let rs: PointTime = s.parse().unwrap();
        assert_eq!((rs.year, rs.month, rs.day), (2023, 11, 15));
        assert_eq!((rs.hour, rs.min, rs.sec), (19, 10, 20))
    }

    #[test]
    fn test_add_month() {
        assert_eq!(
            "20231031T180000Z"
                .parse::<PointTime>()
                .unwrap()
                .add_month(1),
            "20231231T180000Z".parse().unwrap()
        );

        assert_eq!(
            "20231031T180000Z"
                .parse::<PointTime>()
                .unwrap()
                .add_month(2),
            "20231231T180000Z".parse().unwrap()
        );

        assert_eq!(
            "20231031T180000Z"
                .parse::<PointTime>()
                .unwrap()
                .add_month(3),
            "20240131T180000Z".parse().unwrap()
        );

        assert_eq!(
            "20231029T180000Z"
                .parse::<PointTime>()
                .unwrap()
                .add_month(1),
            "20231129T180000Z".parse().unwrap()
        );
    }

    #[test]
    fn test_ord() {
        assert_eq!(
            "20231115T191020".parse::<PointTime>().unwrap()
                < "20231116T191020".parse::<PointTime>().unwrap(),
            true
        );
        assert_eq!(
            "20231116T191020".parse::<PointTime>().unwrap()
                > "20231115T191020".parse::<PointTime>().unwrap(),
            true
        );
        assert_eq!(
            "20231116T191020".parse::<PointTime>().unwrap()
                == "20231116T191020".parse::<PointTime>().unwrap(),
            true
        );
        assert_eq!(
            "20231029T191020".parse::<PointTime>().unwrap()
                < "23000101T000000".parse::<PointTime>().unwrap(),
            true
        );
    }
}
