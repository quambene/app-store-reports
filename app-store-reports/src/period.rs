use crate::error::Error;
use std::{fmt, str::FromStr};

/// A calendar month, as used in Apple's `filter[reportDate]=YYYY-MM` query parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Period {
    year: u16,
    month: u8,
}

impl Period {
    pub fn new(year: u16, month: u8) -> Result<Self, Error> {
        if !(1..=12).contains(&month) {
            return Err(Error::InvalidPeriod(format!("{year:04}-{month:02}")));
        }
        Ok(Self { year, month })
    }

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    fn succ(self) -> Self {
        if self.month == 12 {
            Self {
                year: self.year + 1,
                month: 1,
            }
        } else {
            Self {
                year: self.year,
                month: self.month + 1,
            }
        }
    }

    /// Apple's `reportDate` filter labels a report by its fiscal settlement
    /// period, which lags the actual sales dates inside the report by
    /// roughly 3 calendar months (observed empirically; Apple does not
    /// document the mapping). This approximates the real sales month for
    /// display purposes — it is not exact, since Apple's fiscal periods
    /// follow a 4-4-5 week calendar rather than calendar months.
    pub fn approx_sales_period(self) -> Self {
        let total_months = self.year as i32 * 12 + (self.month as i32 - 1) - 3;
        Self {
            year: (total_months.div_euclid(12)) as u16,
            month: (total_months.rem_euclid(12) + 1) as u8,
        }
    }

    /// Builds the inclusive range `self..=end`, iterating month by month.
    ///
    /// Errors with [`Error::InvalidRange`] if `end` is before `self`.
    pub fn inclusive_range(self, end: Self) -> Result<PeriodRange, Error> {
        if end < self {
            return Err(Error::InvalidRange { start: self, end });
        }
        Ok(PeriodRange {
            current: Some(self),
            end,
        })
    }
}

impl FromStr for Period {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let (year_str, month_str) = s
            .split_once('-')
            .ok_or_else(|| Error::InvalidPeriod(s.to_string()))?;
        let year: u16 = year_str
            .parse()
            .map_err(|_| Error::InvalidPeriod(s.to_string()))?;
        let month: u8 = month_str
            .parse()
            .map_err(|_| Error::InvalidPeriod(s.to_string()))?;
        Period::new(year, month).map_err(|_| Error::InvalidPeriod(s.to_string()))
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

/// Inclusive iterator over consecutive [`Period`]s, produced by [`Period::inclusive_range`].
pub struct PeriodRange {
    current: Option<Period>,
    end: Period,
}

impl Iterator for PeriodRange {
    type Item = Period;

    fn next(&mut self) -> Option<Period> {
        let current = self.current?;
        self.current = if current == self.end {
            None
        } else {
            Some(current.succ())
        };
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_periods() {
        assert_eq!(
            "2025-01".parse::<Period>().unwrap(),
            Period::new(2025, 1).unwrap()
        );
        assert_eq!(
            "2025-1".parse::<Period>().unwrap(),
            Period::new(2025, 1).unwrap()
        );
    }

    #[test]
    fn displays_zero_padded() {
        assert_eq!(Period::new(2025, 1).unwrap().to_string(), "2025-01");
        assert_eq!(Period::new(2025, 12).unwrap().to_string(), "2025-12");
    }

    #[test]
    fn rejects_invalid_periods() {
        assert!("2025-13".parse::<Period>().is_err());
        assert!("2025-00".parse::<Period>().is_err());
        assert!("garbage".parse::<Period>().is_err());
        assert!("2025".parse::<Period>().is_err());
    }

    #[test]
    fn orders_chronologically() {
        assert!(Period::new(2025, 12).unwrap() < Period::new(2026, 1).unwrap());
        assert!(Period::new(2025, 1).unwrap() < Period::new(2025, 2).unwrap());
    }

    #[test]
    fn succ_rolls_over_year() {
        assert_eq!(
            Period::new(2024, 12).unwrap().succ(),
            Period::new(2025, 1).unwrap()
        );
        assert_eq!(
            Period::new(2025, 5).unwrap().succ(),
            Period::new(2025, 6).unwrap()
        );
    }

    #[test]
    fn inclusive_range_spans_year_boundary() {
        let start = Period::new(2025, 11).unwrap();
        let end = Period::new(2026, 2).unwrap();
        let periods: Vec<Period> = start.inclusive_range(end).unwrap().collect();
        assert_eq!(
            periods,
            vec![
                Period::new(2025, 11).unwrap(),
                Period::new(2025, 12).unwrap(),
                Period::new(2026, 1).unwrap(),
                Period::new(2026, 2).unwrap(),
            ]
        );
    }

    #[test]
    fn inclusive_range_single_period() {
        let period = Period::new(2025, 3).unwrap();
        let periods: Vec<Period> = period.inclusive_range(period).unwrap().collect();
        assert_eq!(periods, vec![period]);
    }

    #[test]
    fn approx_sales_period_shifts_back_three_months() {
        assert_eq!(
            Period::new(2026, 8).unwrap().approx_sales_period(),
            Period::new(2026, 5).unwrap()
        );
    }

    #[test]
    fn approx_sales_period_rolls_back_across_year_boundary() {
        assert_eq!(
            Period::new(2026, 1).unwrap().approx_sales_period(),
            Period::new(2025, 10).unwrap()
        );
        assert_eq!(
            Period::new(2026, 3).unwrap().approx_sales_period(),
            Period::new(2025, 12).unwrap()
        );
    }

    #[test]
    fn inclusive_range_rejects_end_before_start() {
        let start = Period::new(2025, 3).unwrap();
        let end = Period::new(2025, 2).unwrap();
        assert!(matches!(
            start.inclusive_range(end),
            Err(Error::InvalidRange { .. })
        ));
    }
}
