use crate::error::Error;
use std::{fmt, str::FromStr};

/// An Apple financial report region code (e.g. `US`, `EU`, `JP`, `WW`), as used in
/// `filter[regionCode]`. A financial report covers one Apple-defined region per
/// request, not a single country. This is a validating newtype rather than a
/// hardcoded enum, since Apple's region list isn't closed and would go stale.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionCode(String);

impl FromStr for RegionCode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(Error::EmptyRegion);
        }
        Ok(Self(trimmed.to_ascii_uppercase()))
    }
}

impl fmt::Display for RegionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for RegionCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_uppercases() {
        assert_eq!(
            " us ".parse::<RegionCode>().unwrap(),
            "US".parse::<RegionCode>().unwrap()
        );
        assert_eq!("eu".parse::<RegionCode>().unwrap().to_string(), "EU");
    }

    #[test]
    fn rejects_empty() {
        assert!(matches!("".parse::<RegionCode>(), Err(Error::EmptyRegion)));
        assert!(matches!(
            "   ".parse::<RegionCode>(),
            Err(Error::EmptyRegion)
        ));
    }
}
