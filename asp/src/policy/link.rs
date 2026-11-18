use crate::headers::{self, AttributeNameError, Link};
use http::{Uri, uri::InvalidUri};
use std::vec;

/// Represents a policy [link](Link) builder.
pub struct LinkBuilder<'a> {
    index: usize,
    list: &'a mut LinkList,
}

impl<'a> LinkBuilder<'a> {
    #[inline]
    fn new(index: usize, list: &'a mut LinkList) -> Self {
        Self { index, list }
    }
}

impl LinkBuilder<'_> {
    /// Adds a language to the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the language to add
    pub fn add_language<S: AsRef<str>>(self, value: S) -> Self {
        self.list.links[self.index].add_language(value);
        self
    }

    /// Sets the title of the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the link title
    pub fn title<S: Into<String>>(self, value: S) -> Self {
        self.list.links[self.index].title = value.into();
        self
    }

    /// Sets the media of the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the link media
    pub fn media<S: Into<String>>(self, value: S) -> Self {
        self.list.links[self.index].media = value.into();
        self
    }

    /// Sets the media type of the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the link media type
    pub fn media_type<S: Into<String>>(self, value: S) -> Self {
        self.list.links[self.index].media_type = value.into();
        self
    }

    /// Adds an extension with the specified key and value to the link being built.
    ///
    /// # Arguments
    ///
    /// * `key` - the extension key
    /// * `value` - the extension value
    ///
    /// # Remarks
    ///
    /// If `value` is empty, no operation is perform. The operation fails if `key`:
    ///
    /// - is empty
    /// - the first character is not a letter
    /// - all other characters are not letters, numbers, '.', '-', or '_'
    pub fn add_extension<S: AsRef<str>>(self, key: S, value: S) -> Result<Self, AttributeNameError> {
        self.list.links[self.index].add_extension(key, value).map(|_| self)
    }

    /// Creates and returns a new link builder.
    ///
    /// # Arguments
    ///
    /// * `target` - the target [URL](Uri)
    ///
    /// # Panics
    ///
    /// This function panics if the the specified `target` [URL](Uri) is invalid.
    #[inline]
    pub fn link(self, target: impl TryInto<Uri, Error = InvalidUri>) -> Self {
        self.list.get_or_add(target)
    }
}

/// Represents a list of [builders](LinkBuilder) that builds a set of links for an API versioning policy.
#[derive(Clone, Debug, Default)]
pub struct LinkList {
    rel: &'static str,
    links: Vec<Link>,
}

impl LinkList {
    /// Initializes a new [LinkList].
    ///
    /// # Arguments
    ///
    /// * `rel` - the link relation type that identifies the semantics of the links
    pub fn new(rel: &'static str) -> Self {
        LinkList { rel, links: Vec::new() }
    }

    /// Gets or creates a new link builder.
    ///
    /// # Arguments
    ///
    /// * `target` - the target [URL](Uri)
    ///
    /// # Panics
    ///
    /// This function panics if the the specified `target` [URL](Uri) is invalid.
    pub fn get_or_add(&mut self, target: impl TryInto<Uri, Error = InvalidUri>) -> LinkBuilder<'_> {
        let key = target.try_into().unwrap();

        for (index, link) in self.links.iter().enumerate() {
            if link.target() == &key {
                return LinkBuilder::new(index, self);
            }
        }

        let index = self.links.len();

        self.links.push(headers::Link::new(key.clone(), self.rel));
        LinkBuilder::new(index, self)
    }
}

impl IntoIterator for LinkList {
    type Item = Link;
    type IntoIter = vec::IntoIter<Link>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.links.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headers::AttributeNameError::InvalidStart;

    #[test]
    fn new_list_should_be_empty() {
        // arrange
        let list = LinkList::new("deprecation");

        // act
        let links: Vec<_> = list.into_iter().collect();

        // assert
        assert!(links.is_empty());
    }

    #[test]
    fn get_or_add_should_create_link_with_relation_type() {
        // arrange
        let mut list = LinkList::new("deprecation");

        // act
        list.get_or_add("https://example.com/policy");

        // assert
        let links: Vec<_> = list.into_iter().collect();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target(), "https://example.com/policy");
        assert_eq!(links[0].rel(), "deprecation");
    }

    #[test]
    fn get_or_add_should_return_existing_link_for_same_target() {
        // arrange
        let mut list = LinkList::new("sunset");

        list.get_or_add("https://example.com/policy").title("Policy");

        // act
        list.get_or_add("https://example.com/policy").media("screen");

        // assert
        let links: Vec<_> = list.into_iter().collect();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].title, "Policy");
        assert_eq!(links[0].media, "screen");
    }

    #[test]
    #[should_panic]
    fn get_or_add_should_panic_for_invalid_target() {
        // arrange
        let mut list = LinkList::new("sunset");

        // act
        list.get_or_add("not a url");

        // assert
    }

    #[test]
    fn link_builder_should_set_all_attributes() {
        // arrange
        let mut list = LinkList::new("sunset");

        // act
        list.get_or_add("https://example.com/policy")
            .title("Policy")
            .media("screen")
            .media_type("text/html")
            .add_language("en")
            .add_language("fr")
            .add_extension("ext", "42")
            .unwrap();

        // assert
        let links: Vec<_> = list.into_iter().collect();
        let link = &links[0];

        assert_eq!(link.title, "Policy");
        assert_eq!(link.media, "screen");
        assert_eq!(link.media_type, "text/html");
        assert_eq!(link.languages(), ["en", "fr"]);
        assert_eq!(link.extensions().get("ext").map(String::as_str), Some("42"));
    }

    #[test]
    fn link_builder_should_fail_for_invalid_extension() {
        // arrange
        let mut list = LinkList::new("sunset");

        // act
        let result = list
            .get_or_add("https://example.com/policy")
            .add_extension("42", "value");

        // assert
        assert_eq!(result.err(), Some(InvalidStart));
    }

    #[test]
    fn link_builder_should_chain_to_another_link() {
        // arrange
        let mut list = LinkList::new("sunset");

        // act
        list.get_or_add("https://example.com/first")
            .title("First")
            .link("https://example.com/second")
            .title("Second");

        // assert
        let links: Vec<_> = list.into_iter().collect();

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].title, "First");
        assert_eq!(links[1].title, "Second");
        assert_eq!(links[1].rel(), "sunset");
    }
}
