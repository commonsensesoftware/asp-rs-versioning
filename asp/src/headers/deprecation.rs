use http::HeaderValue;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::ops::Deref;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents a `deprecation` header.
///
/// # Remarks
///
/// Refer to **The Deprecation HTTP Header Field** in [RFC 9745](https://datatracker.ietf.org/doc/html/rfc9745).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Deprecation(SystemTime);

impl Deprecation {
    /// Initializes a new [Deprecation] header.
    ///
    /// # Arguments
    ///
    /// * `date` - the deprecation [date and time](SystemTime)
    #[inline]
    pub fn new(date: SystemTime) -> Self {
        Self(date)
    }
}

impl From<Deprecation> for SystemTime {
    fn from(value: Deprecation) -> Self {
        value.0
    }
}

impl From<SystemTime> for Deprecation {
    fn from(value: SystemTime) -> Self {
        Self(value)
    }
}

impl AsRef<SystemTime> for Deprecation {
    fn as_ref(&self) -> &SystemTime {
        &self.0
    }
}

impl Deref for Deprecation {
    type Target = SystemTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Deprecation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let time = self.0.duration_since(UNIX_EPOCH).unwrap_or_default();

        f.write_char('@')?;
        time.as_secs().fmt(f)
    }
}

impl From<Deprecation> for HeaderValue {
    fn from(value: Deprecation) -> Self {
        Self::try_from(value.to_string()).unwrap()
    }
}

impl TryFrom<HeaderValue> for Deprecation {
    type Error = Box<dyn Error + Send + Sync + 'static>;

    #[inline]
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&HeaderValue> for Deprecation {
    type Error = Box<dyn Error + Send + Sync + 'static>;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let text = value.to_str().map_err(Box::new)?;

        if text.chars().nth(0) == Some('@')
            && let Ok(secs) = text[1..].parse::<u64>()
            && let Some(date) = UNIX_EPOCH.checked_add(Duration::from_secs(secs))
        {
            Ok(Self(date))
        } else {
            Err(Box::from("invalid deprecation date"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn new_should_wrap_date() {
        // arrange
        let date = UNIX_EPOCH + Duration::from_secs(1_700_000_000);

        // act
        let deprecation = Deprecation::new(date);

        // assert
        assert_eq!(*deprecation, date);
        assert_eq!(deprecation.as_ref(), &date);
        assert_eq!(SystemTime::from(deprecation), date);
        assert_eq!(Deprecation::from(date), deprecation);
    }

    #[rstest]
    #[case(0, "@0")]
    #[case(1_700_000_000, "@1700000000")]
    fn to_string_should_format_structured_date(#[case] secs: u64, #[case] expected: &str) {
        // arrange
        let deprecation = Deprecation::new(UNIX_EPOCH + Duration::from_secs(secs));

        // act
        let text = deprecation.to_string();

        // assert
        assert_eq!(text, expected);
    }

    #[test]
    fn to_string_should_clamp_date_before_unix_epoch() {
        // arrange
        let deprecation = Deprecation::new(UNIX_EPOCH - Duration::from_secs(1));

        // act
        let text = deprecation.to_string();

        // assert
        assert_eq!(text, "@0");
    }

    #[test]
    fn to_string_should_truncate_fractional_seconds() {
        // arrange
        let deprecation = Deprecation::new(UNIX_EPOCH + Duration::from_millis(1_700_000_000_999));

        // act
        let text = deprecation.to_string();

        // assert
        assert_eq!(text, "@1700000000");
    }

    #[test]
    fn into_header_should_use_structured_date() {
        // arrange
        let deprecation = Deprecation::new(UNIX_EPOCH + Duration::from_secs(1_700_000_000));
        let expected = HeaderValue::from_static("@1700000000");

        // act
        let header = HeaderValue::from(deprecation);

        // assert
        assert_eq!(header, expected);
    }

    #[rstest]
    #[case("@0", 0)]
    #[case("@1700000000", 1_700_000_000)]
    fn try_from_header_should_parse_structured_date(#[case] text: &'static str, #[case] secs: u64) {
        // arrange
        let header = HeaderValue::from_static(text);
        let expected = Deprecation::new(UNIX_EPOCH + Duration::from_secs(secs));

        // act
        let deprecation = Deprecation::try_from(header).unwrap();

        // assert
        assert_eq!(deprecation, expected);
    }

    #[rstest]
    #[case("")]
    #[case("@")]
    #[case("1700000000")]
    #[case("@abc")]
    #[case("@-1")]
    #[case("@1.5")]
    #[case("@ 1700000000")]
    #[case("@18446744073709551616")]
    #[case("Tue, 14 Nov 2023 22:13:20 GMT")]
    fn try_from_header_should_fail_for_invalid_date(#[case] text: &'static str) {
        // arrange
        let header = HeaderValue::from_static(text);

        // act
        let result = Deprecation::try_from(&header);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn try_from_header_should_fail_for_non_visible_ascii() {
        // arrange
        let header = HeaderValue::from_bytes(b"@17\xe90").unwrap();

        // act
        let result = Deprecation::try_from(&header);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn deprecation_should_round_trip_through_header() {
        // arrange
        let expected = Deprecation::new(UNIX_EPOCH + Duration::from_secs(1_700_000_000));
        let header = HeaderValue::from(expected);

        // act
        let deprecation = Deprecation::try_from(header).unwrap();

        // assert
        assert_eq!(deprecation, expected);
    }

    #[test]
    fn deprecation_should_order_by_date() {
        // arrange
        let earlier = Deprecation::new(UNIX_EPOCH + Duration::from_secs(1));
        let later = Deprecation::new(UNIX_EPOCH + Duration::from_secs(2));

        // act
        let ordered = earlier < later;

        // assert
        assert!(ordered);
    }
}
