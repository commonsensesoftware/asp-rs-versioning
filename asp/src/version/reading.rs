use super::builtin::mask;
use super::{ApiVersion, DAY_SHIFT, EPOCH, Format, MAJOR_SHIFT, MINOR_SHIFT, MONTH_SHIFT, YEAR_SHIFT};
use crate::prelude::*;
use time::{Date, Month};

impl ApiVersion<'_> {
    /// Gets the API version [format](Format).
    ///
    /// # Remarks
    ///
    /// This function is intentionally undocumented and is only intended for custom API version formats. Normal usage is
    /// not meant to ever directly know or access the format.
    #[doc(hidden)]
    pub const fn format(&self) -> Format {
        match self.0 >> 30 {
            0 => Format::Unknown,
            1 => Format::Numeric,
            2 => Format::Date,
            _ => Format::Custom,
        }
    }

    /// Gets a value indicating whether the API version is [neutral](ApiVersion::NEUTRAL).
    #[inline]
    pub const fn is_neutral(&self) -> bool {
        matches!(self.format(), Format::Unknown)
    }

    /// Gets the raw bits stored in the API version.
    ///
    /// # Remarks
    ///
    /// This function is intentionally undocumented and is only intended for custom API version formats. Normal usage is
    /// not meant to ever directly access the raw bits.
    #[doc(hidden)]
    pub const fn bits(&self) -> u32 {
        self.0 & mask::CUSTOM
    }
}

impl ApiVersionStatusExt for ApiVersion<'_> {
    fn status(&self) -> Option<&str> {
        self.1.as_deref()
    }
}

impl NumericApiVersionExt for ApiVersion<'_> {
    fn major(&self) -> Option<u8> {
        if self.format() == Format::Numeric {
            Some((self.0 >> MAJOR_SHIFT) as u8)
        } else {
            None
        }
    }

    fn minor(&self) -> Option<u8> {
        if self.format() == Format::Numeric {
            Some((self.0 >> MINOR_SHIFT) as u8)
        } else {
            None
        }
    }
}

impl DateApiVersionExt for ApiVersion<'_> {
    fn date(&self) -> Option<Date> {
        if self.format() == Format::Date {
            let year = (((self.0 >> YEAR_SHIFT) & 0xff) + EPOCH) as i32;
            let month = Month::try_from(((self.0 >> MONTH_SHIFT) & 0x0f) as u8).unwrap();
            let day = ((self.0 >> DAY_SHIFT) & 0x1f) as u8;

            Some(Date::from_calendar_date(year, month, day).unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
