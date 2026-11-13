//! Calendar date of the simulated day.

use serde::{Deserialize, Serialize};

/// A proleptic Gregorian date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarDate {
    /// Year.
    pub year: i32,
    /// Month, 1..=12.
    pub month: u8,
    /// Day of the month, 1-based.
    pub day: u8,
}

const MONTH_DAYS: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// True for leap years.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in a year.
pub fn days_in_year(year: i32) -> u16 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

impl CalendarDate {
    /// A date; month and day are clamped to valid values.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let month = month.clamp(1, 12);
        let mut date = Self {
            year,
            month,
            day: 1,
        };
        date.day = day.clamp(1, date.days_in_month());
        date
    }

    /// Days in this date's month.
    pub fn days_in_month(self) -> u8 {
        let extra = u16::from(self.month == 2 && is_leap_year(self.year));
        (MONTH_DAYS[usize::from(self.month) - 1] + extra) as u8
    }

    /// Day of the year, 1-based.
    pub fn day_of_year(self) -> u16 {
        let before: u16 = MONTH_DAYS[..usize::from(self.month) - 1].iter().sum();
        let leap = u16::from(self.month > 2 && is_leap_year(self.year));
        before + leap + u16::from(self.day)
    }

    /// The date with the given 1-based day of the year (clamped to the year length).
    pub fn from_day_of_year(year: i32, day_of_year: u16) -> Self {
        let mut left = day_of_year.clamp(1, days_in_year(year));
        for month in 1..=12u8 {
            let probe = Self::new(year, month, 1);
            let length = u16::from(probe.days_in_month());
            if left <= length {
                return Self::new(year, month, left as u8);
            }
            left -= length;
        }
        Self::new(year, 12, 31)
    }

    /// Julian day number at 0h UT of this date.
    pub fn julian_day(self) -> f64 {
        let (y, m) = if self.month <= 2 {
            (self.year - 1, i32::from(self.month) + 12)
        } else {
            (self.year, i32::from(self.month))
        };
        let a = y.div_euclid(100);
        let b = 2 - a + a.div_euclid(4);
        (365.25 * f64::from(y + 4716)).floor()
            + (30.6001 * f64::from(m + 1)).floor()
            + f64::from(self.day)
            + f64::from(b)
            - 1524.5
    }
}
