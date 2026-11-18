use super::builtin::mask;
use super::custom::{self, CustomFormat, custom_format};
use super::{ApiVersion, Format};
use crate::prelude::*;
use std::fmt::{self, Debug, Display, Formatter, Write};

impl Debug for ApiVersion<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct(stringify!(ApiVersion));

        match self.format() {
            Format::Unknown => return debug.field("neutral", &true).finish(),
            Format::Numeric => {
                debug
                    .field("major", &self.major().unwrap())
                    .field("minor", &self.minor().unwrap());

                // only show the extension when present so built-in versions are unchanged
                let extension = self.bits() & mask::NUMERIC;

                if extension > 0 {
                    debug.field("extension", &extension);
                }
            }
            Format::Date => {
                debug.field("date", &self.date().unwrap());

                let extension = self.bits() & mask::DATE;

                if extension > 0 {
                    debug.field("extension", &extension);
                }
            }
            Format::Custom => {
                debug.field("custom", &self.bits());
            }
        }

        debug.field("status", &self.status().unwrap_or_default()).finish()
    }
}

impl Display for ApiVersion<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.display(f, custom_format())
    }
}

impl ApiVersion<'_> {
    fn display(&self, f: &mut Formatter<'_>, custom: Option<&dyn CustomFormat>) -> fmt::Result {
        match custom {
            // a neutral API version can never be reinterpreted
            Some(format) if self.format() != Format::Unknown => custom::fmt(format, self, f),
            _ => fmt(self, f),
        }
    }
}

pub(super) fn fmt(version: &ApiVersion<'_>, f: &mut Formatter<'_>) -> fmt::Result {
    match version.format() {
        Format::Unknown => {
            f.write_char('*')?;
        }
        Format::Numeric => {
            Display::fmt(&version.major().unwrap_or_default(), f)?;

            let minor = version.minor().unwrap_or_default();

            // omit minor version in the alt form when 0
            if minor > 0 || !f.alternate() {
                f.write_char('.')?;
                Display::fmt(&minor, f)?;
            }

            if let Some(status) = version.status() {
                f.write_char('-')?;
                f.write_str(status)?;
            }
        }
        Format::Date => {
            Display::fmt(&version.date().unwrap(), f)?;

            if let Some(status) = version.status() {
                f.write_char('-')?;
                f.write_str(status)?;
            }
        }
        // only the custom format knows how to render itself
        Format::Custom => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests;
