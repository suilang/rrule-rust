use chrono::{DateTime, LocalResult, NaiveDate, TimeZone};
use chrono_tz::Tz;
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(PartialEq, Debug, Clone)]
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
        if self.year > other.year {
            return Some(Ordering::Greater);
        }
        if self.month > other.month {
            return Some(Ordering::Greater);
        }
        if self.day > other.day {
            return Some(Ordering::Greater);
        }
        if self.hour > other.hour {
            return Some(Ordering::Greater);
        }
        if self.min > other.min {
            return Some(Ordering::Greater);
        }
        if self.sec > other.sec {
            return Some(Ordering::Greater);
        }
        return Some(Ordering::Equal);
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
        let add = |curr: &mut PointTime| {
            curr.month = curr.month + mon;
            if curr.month > 12 {
                curr.month -= 12;
                curr.year += 1;
            };
        };
        add(&mut next_point_time);

        while !next_point_time.is_valid() {
            add(&mut next_point_time)
        }
        next_point_time
    }

    pub fn is_valid(&self) -> bool {
        let date_time = NaiveDate::from_ymd_opt(self.year, self.month, self.day);
        match date_time {
            Some(_) => false,
            _ => true,
        }
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
        let mut time1: PointTime = "20231031T180000Z".parse().unwrap();

        let time2 = time1.add_month(1);
        assert_eq!(time2, "20231231T180000Z".parse().unwrap());

        let time3 = time1.add_month(2);
        assert_eq!(time3, "20231231T180000Z".parse().unwrap());

        assert_eq!(
            "20231031T180000Z"
                .parse::<PointTime>()
                .unwrap()
                .add_month(3),
            "20240131T180000Z".parse().unwrap()
        )
    }
}
