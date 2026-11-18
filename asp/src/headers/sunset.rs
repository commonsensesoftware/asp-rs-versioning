use http::HeaderValue;
use httpdate::{HttpDate, parse_http_date};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::time::SystemTime;

/// Represents a `sunset` header.
///
/// # Remarks
///
/// Refer to **The Sunset HTTP Header Field** in [RFC 8594](https://datatracker.ietf.org/doc/html/rfc8594).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sunset(SystemTime);

impl Sunset {
    /// Initializes a new [Sunset] header.
    ///
    /// # Arguments
    ///
    /// * `date` - the sunset [date and time](SystemTime)
    #[inline]
    pub fn new(date: SystemTime) -> Self {
        Self(date)
    }
}

impl From<Sunset> for SystemTime {
    fn from(value: Sunset) -> Self {
        value.0
    }
}

impl From<SystemTime> for Sunset {
    fn from(value: SystemTime) -> Self {
        Self(value)
    }
}

impl AsRef<SystemTime> for Sunset {
    fn as_ref(&self) -> &SystemTime {
        &self.0
    }
}

impl Deref for Sunset {
    type Target = SystemTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Sunset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        HttpDate::from(self.0).fmt(f)
    }
}

impl From<Sunset> for HeaderValue {
    fn from(value: Sunset) -> Self {
        Self::try_from(value.to_string()).unwrap()
    }
}

impl TryFrom<HeaderValue> for Sunset {
    type Error = Box<dyn Error + Send + Sync + 'static>;

    #[inline]
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&HeaderValue> for Sunset {
    type Error = Box<dyn Error + Send + Sync + 'static>;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let text = value.to_str().map_err(Box::new)?;
        let date = parse_http_date(text).map_err(Box::new)?;

        Ok(Self(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::time::{Duration, UNIX_EPOCH};

    fn date(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn new_should_wrap_date() {
        // arrange
        let date = date(1_700_000_000);

        // act
        let sunset = Sunset::new(date);

        // assert
        assert_eq!(*sunset, date);
        assert_eq!(sunset.as_ref(), &date);
        assert_eq!(SystemTime::from(sunset), date);
        assert_eq!(Sunset::from(date), sunset);
    }

    #[rstest]
    #[case(0, "Thu, 01 Jan 1970 00:00:00 GMT")]
    #[case(1_700_000_000, "Tue, 14 Nov 2023 22:13:20 GMT")]
    fn to_string_should_format_http_date(#[case] secs: u64, #[case] expected: &str) {
        // arrange
        let sunset = Sunset::new(date(secs));

        // act
        let text = sunset.to_string();

        // assert
        assert_eq!(text, expected);
    }

    #[test]
    fn to_string_should_truncate_fractional_seconds() {
        // arrange
        let sunset = Sunset::new(date(1_700_000_000) + Duration::from_millis(999));

        // act
        let text = sunset.to_string();

        // assert
        assert_eq!(text, "Tue, 14 Nov 2023 22:13:20 GMT");
    }

    #[test]
    fn into_header_should_use_http_date() {
        // arrange
        let sunset = Sunset::new(date(1_700_000_000));
        let expected = HeaderValue::from_static("Tue, 14 Nov 2023 22:13:20 GMT");

        // act
        let header = HeaderValue::from(sunset);

        // assert
        assert_eq!(header, expected);
    }

    #[rstest]
    #[case("Tue, 14 Nov 2023 22:13:20 GMT")]
    #[case("Tuesday, 14-Nov-23 22:13:20 GMT")]
    #[case("Tue Nov 14 22:13:20 2023")]
    fn try_from_header_should_parse_http_date(#[case] text: &'static str) {
        // arrange
        let header = HeaderValue::from_static(text);
        let expected = Sunset::new(date(1_700_000_000));

        // act
        let sunset = Sunset::try_from(header).unwrap();

        // assert
        assert_eq!(sunset, expected);
    }

    #[rstest]
    #[case("")]
    #[case("not a date")]
    #[case("@1700000000")]
    #[case("2023-11-14T22:13:20Z")]
    #[case("Tue, 14 Nov 2023 22:13:20")]
    #[case("Tue, 31 Nov 2023 22:13:20 GMT")]
    #[case("Tue, 14 Nov 2023 25:13:20 GMT")]
    fn try_from_header_should_fail_for_invalid_date(#[case] text: &'static str) {
        // arrange
        let header = HeaderValue::from_static(text);

        // act
        let result = Sunset::try_from(&header);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn try_from_header_should_fail_for_non_visible_ascii() {
        // arrange
        let header = HeaderValue::from_bytes(b"Tue, 14 Nov 2023 22:13:20 G\xe9T").unwrap();

        // act
        let result = Sunset::try_from(&header);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn sunset_should_round_trip_through_header() {
        // arrange
        let expected = Sunset::new(date(1_700_000_000));
        let header = HeaderValue::from(expected);

        // act
        let sunset = Sunset::try_from(header).unwrap();

        // assert
        assert_eq!(sunset, expected);
    }

    #[test]
    fn sunset_should_order_by_date() {
        // arrange
        let earlier = Sunset::new(date(1));
        let later = Sunset::new(date(2));

        // act
        let ordered = earlier < later;

        // assert
        assert!(ordered);
    }
}
