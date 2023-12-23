use chrono::Weekday;
use core::fmt;
use std::str::FromStr;

use crate::rrule::weekday;

#[derive(Debug, PartialEq)]
pub enum NWeekday {
    /// When it is every weekday of the month or year.
    Every(Weekday),
    /// When it is the nth weekday of the month or year.
    /// The first member's value is from -366 to -1 and 1 to 366 depending on frequency
    Nth(i16, Weekday),
}

impl NWeekday {
    /// Creates a new week occurrence
    ///
    /// # Arguments
    ///
    /// * `n` - The nth occurrence of the week day. Should be between -366 and 366, and not 0.
    /// * `weekday` - The week day
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::Weekday;
    /// use rrule_rust::rrule::weekday::NWeekday;
    /// let nth_weekday = NWeekday::new(Some(1), Weekday::Mon);
    /// ```
    #[must_use]
    pub fn new(number: Option<i16>, weekday: Weekday) -> Self {
        match number {
            Some(number) => Self::Nth(number, weekday),
            None => Self::Every(weekday),
        }
    }

    pub fn get_weekday(&self) -> &Weekday {
        match self {
            NWeekday::Every(weekday) => weekday,
            NWeekday::Nth(_, weekday) => weekday,
        }
    }

    pub fn weekday_to_str(d: &Weekday) -> String {
        match d {
            Weekday::Mon => "MO".to_string(),
            Weekday::Tue => "TU".to_string(),
            Weekday::Wed => "WE".to_string(),
            Weekday::Thu => "TH".to_string(),
            Weekday::Fri => "FR".to_string(),
            Weekday::Sat => "SA".to_string(),
            Weekday::Sun => "SU".to_string(),
        }
    }

    pub fn is_every(&self) -> bool {
        match self {
            NWeekday::Every(_) => true,
            NWeekday::Nth(_, _) => false,
        }
    }

    pub fn get_nth(&self) -> i16 {
        match self {
            NWeekday::Every(_) => 0,
            NWeekday::Nth(n, _) => *n,
        }
    }
}

impl FromStr for NWeekday {
    type Err = String;

    /// Generates an [`NWeekday`] from a string.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let length = value.len();

        if length < 2 {
            return Err(value.into());
        }

        // it doesn't have any issue, because we checked the string is ASCII above
        let wd = str_to_weekday(&value[(length - 2)..]).map_err(|_| value.to_string())?;
        let nth = value[..(length - 2)].parse::<i16>().unwrap_or_default();

        if nth == 0 {
            Ok(Self::Every(wd))
        } else {
            Ok(Self::new(Some(nth), wd))
        }
    }
}

impl fmt::Display for NWeekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NWeekday::Every(weekday) => NWeekday::weekday_to_str(weekday),
                NWeekday::Nth(n, weekday) => format!("{}{}", n.to_string(), NWeekday::weekday_to_str(weekday)),
            }
        )
    }
}

/// Attempts to convert a `str` to a `Weekday`.
pub(crate) fn str_to_weekday(d: &str) -> Result<Weekday, String> {
    let day = match &d.to_uppercase()[..] {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return Err(d.to_string()),
    };
    Ok(day)
}

/// Parse the "BYWEEKDAY" and "BYDAY" values
/// Example: `SU,MO,TU,WE,TH,FR` or `4MO` or `-1WE`
/// > For example, within a MONTHLY rule, +1MO (or simply 1MO) represents the first Monday
/// > within the month, whereas -1MO represents the last Monday of the month.
pub(crate) fn parse_weekdays(val: &str) -> Result<Vec<NWeekday>, String> {
    let mut wdays = vec![];
    // Separate all days
    for day in val.split(',') {
        let wday = day.parse::<NWeekday>()?;
        wdays.push(wday);
    }
    Ok(wdays)
}
