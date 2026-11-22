use super::{Reader, RequestInfo};
use crate::parameter;
use smallvec::SmallVec;

/// Represents a combined API version reader that aggregates multiple individual readers.
#[derive(Debug, Default)]
pub struct CombinedReader(Vec<Box<dyn Reader>>);

impl CombinedReader {
    pub(crate) fn new(reader: impl Reader + 'static) -> Self {
        let mut readers = Vec::with_capacity(2);
        readers.push(Box::new(reader) as Box<dyn Reader>);
        Self(readers)
    }

    /// Adds an additional API version reader to the combined reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - the API version [reader](Reader) to add to the combined reader
    pub fn combine(mut self, reader: impl Reader + 'static) -> Self {
        self.0.push(Box::new(reader));
        self
    }
}

impl Reader for CombinedReader {
    fn read<'a>(&self, request: &'a dyn RequestInfo) -> SmallVec<[&'a str; 1]> {
        let mut versions = SmallVec::new();

        for reader in &self.0 {
            for version in reader.read(request) {
                if !versions.contains(&version) {
                    versions.push(version);
                }
            }
        }

        versions
    }

    fn clone(&self) -> Box<dyn Reader> {
        Box::new(Clone::clone(self))
    }
}

impl parameter::Descriptor for CombinedReader {
    fn describe<'a>(&'a self, context: &mut dyn parameter::Context<'a>) {
        for reader in &self.0 {
            reader.describe(context);
        }
    }
}

impl Clone for CombinedReader {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|r| Reader::clone(&**r)).collect())
    }
}

impl From<CombinedReader> for Box<dyn Reader> {
    fn from(reader: CombinedReader) -> Self {
        Box::new(reader)
    }
}

impl From<CombinedReader> for Box<dyn parameter::Descriptor> {
    fn from(reader: CombinedReader) -> Self {
        Box::new(reader)
    }
}
