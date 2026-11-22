use crate::readers::{CombinedReader, Reader};

/// Defines the behavior of a combinable API version reader.
pub trait Combine: Sized {
    /// Combines the current reader into a single reader that aggregates the results from multiple API version readers.
    ///
    /// # Arguments
    ///
    /// * `reader` - the API version [reader](Reader) to combine
    fn combine(self, reader: impl Reader + 'static) -> CombinedReader;
}

impl<T: Reader + 'static> Combine for T {
    fn combine(self, reader: impl Reader + 'static) -> CombinedReader {
        CombinedReader::new(self).combine(reader)
    }
}
