use std::{fmt::Display, str::FromStr};

#[derive(PartialEq, Debug)]
pub enum Frequency {
    /// The recurrence occurs on a yearly basis.
    Yearly = 0,
    /// The recurrence occurs on a monthly basis.
    Monthly = 1,
    /// The recurrence occurs on a weekly basis.
    Weekly = 2,
    /// The recurrence occurs on a daily basis.
    Daily = 3,
    /// The recurrence occurs on an hourly basis.
    Hourly = 4,
    /// The recurrence occurs on a minutely basis.
    Minutely = 5,
    /// The recurrence occurs on a second basis.
    Secondly = 6,
}

impl FromStr for Frequency {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let freq = match &value.to_uppercase()[..] {
            "YEARLY" => Self::Yearly,
            "MONTHLY" => Self::Monthly,
            "WEEKLY" => Self::Weekly,
            "DAILY" => Self::Daily,
            "HOURLY" => Self::Hourly,
            "MINUTELY" => Self::Minutely,
            "SECONDLY" => Self::Secondly,
            val => return Err(format!("invalid freq: {}", val.to_string())),
        };
        Ok(freq)
    }
}

impl Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let freq = match self {
            Self::Yearly => "YEARLY",
            Self::Monthly => "MONTHLY",
            Self::Weekly => "WEEKLY",
            Self::Daily => "DAILY",
            Self::Hourly => "HOURLY",
            Self::Minutely => "MINUTELY",
            Self::Secondly => "SECONDLY",
        };
        write!(f, "{}", freq)
    }
}
