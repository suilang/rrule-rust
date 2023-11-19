use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
#[derive(PartialEq, Debug)]
pub struct ParsedDateString {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub min: u32,
    pub sec: u32,
}

impl ParsedDateString {
    /// Parses a date string with format `YYYYMMDDTHHMMSSZ`
    pub fn parse_from_str(s: &str) -> Result<ParsedDateString, &'static str> {
        let year: i32 = s[0..4].parse().map_err(|_err| "invalid string")?;
        let month: u32 = s[4..6]
            .to_string()
            .parse()
            .map_err(|_err| "invalid string")?;
        let day: u32 = s[6..8]
            .to_string()
            .parse()
            .map_err(|_err| "invalid string")?;

        let t = s[8..9].eq("T");
        if !t {
            return Err("invalid string format");
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

        Ok(ParsedDateString {
            year,
            month,
            day,
            hour,
            min,
            sec,
        })
    }
}

pub fn get_tz_from_str(tz: &str) -> Tz {
    let chrono_tz: Tz = tz.parse().unwrap();
    chrono_tz
}

pub fn get_time_with_timezone(data_time: &ParsedDateString, tz: &Tz) -> DateTime<Tz> {
    let rs = tz
        .with_ymd_and_hms(
            data_time.year,
            data_time.month,
            data_time.day,
            data_time.hour,
            data_time.min,
            data_time.sec,
        )
        .single()
        .unwrap();
    rs
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;
    use chrono_tz::Tz;

    use super::get_tz_from_str;

    use super::{get_time_with_timezone, ParsedDateString};

    #[test]
    fn parse_timestr() {
        let s = String::from("20231115T191020");
        let rs = ParsedDateString::parse_from_str(&s).unwrap();
        assert_eq!((rs.year, rs.month, rs.day), (2023, 11, 15));
        assert_eq!((rs.hour, rs.min, rs.sec), (19, 10, 20))
    }

    #[test]
    fn parse_tz() {
        let s = String::from("America/New_York");
        assert_eq!(get_tz_from_str(&s), Tz::America__New_York);
    }

    #[test]
    fn parse_time_with_tz() {
        let tz: Tz = get_tz_from_str("America/New_York");
        let pds = ParsedDateString::parse_from_str("20231115T191020").unwrap();
        let time = get_time_with_timezone(&pds, &tz);
        assert_eq!(
            time,
            Tz::America__New_York
                .with_ymd_and_hms(2023, 11, 15, 19, 10, 20)
                .single()
                .unwrap()
        );
    }
}
