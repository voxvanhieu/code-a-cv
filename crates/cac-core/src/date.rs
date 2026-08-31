use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

use chrono::{Datelike, NaiveDate};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Period {
    pub start: DatePoint,
    pub end: DatePoint,
}

impl Period {
    pub fn new(start: DatePoint, end: DatePoint) -> Result<Self, PeriodError> {
        if end != DatePoint::Present && start.sort_key() > end.sort_key() {
            return Err(PeriodError { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn is_valid(&self) -> bool {
        self.end == DatePoint::Present || self.start.sort_key() <= self.end.sort_key()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("period starts at {start} after it ends at {end}")]
pub struct PeriodError {
    pub start: DatePoint,
    pub end: DatePoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatePoint {
    Year(i32),
    YearMonth(i32, u8),
    Full(NaiveDate),
    Present,
}

impl DatePoint {
    pub fn year_month(year: i32, month: u8) -> Option<Self> {
        (1..=12)
            .contains(&month)
            .then_some(Self::YearMonth(year, month))
    }

    pub fn granularity(&self) -> u8 {
        match self {
            Self::Year(_) => 1,
            Self::YearMonth(_, _) => 2,
            Self::Full(_) => 3,
            Self::Present => 0,
        }
    }

    fn sort_key(&self) -> (i32, u32, u32) {
        match self {
            Self::Year(year) => (*year, 1, 1),
            Self::YearMonth(year, month) => (*year, u32::from(*month), 1),
            Self::Full(date) => (date.year(), date.month(), date.day()),
            Self::Present => (i32::MAX, 12, 31),
        }
    }
}

impl Display for DatePoint {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Year(value) => write!(formatter, "{value}"),
            Self::YearMonth(year, month) => write!(formatter, "{year:04}-{month:02}"),
            Self::Full(value) => write!(formatter, "{value}"),
            Self::Present => formatter.write_str("Present"),
        }
    }
}

impl Serialize for DatePoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DatePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DatePointVisitor;

        impl Visitor<'_> for DatePointVisitor {
            type Value = DatePoint;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a year, YYYY-MM, YYYY-MM-DD, or `present`")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                i32::try_from(value).map(DatePoint::Year).map_err(E::custom)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                i32::try_from(value).map(DatePoint::Year).map_err(E::custom)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value.eq_ignore_ascii_case("present") {
                    return Ok(DatePoint::Present);
                }
                if let Ok(year) = value.parse::<i32>() {
                    return Ok(DatePoint::Year(year));
                }
                if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                    return Ok(DatePoint::Full(date));
                }
                if let Some((year, month)) = value.split_once('-')
                    && let (Ok(year), Ok(month)) = (year.parse(), month.parse())
                    && let Some(point) = DatePoint::year_month(year, month)
                {
                    return Ok(point);
                }
                Err(E::custom(format!("invalid date point `{value}`")))
            }
        }
        deserializer.deserialize_any(DatePointVisitor)
    }
}

impl PartialOrd for DatePoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sort_key().cmp(&other.sort_key()))
    }
}
