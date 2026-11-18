use super::{
    AttributeNameError::{self, *},
    valid_char,
};
use http::{HeaderValue, Uri};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::str::FromStr;
use thiserror::Error;

/// Represents the possible errors that can occur when parsing a `link` header.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum LinkParseError {
    /// Indicates the value did not contain any links.
    #[error("the value does not contain any links")]
    Empty,

    /// Indicates the opening `<` of the link target was expected at the specified position.
    #[error("expected '<' at position {0}")]
    ExpectedTarget(usize),

    /// Indicates the link target is missing the closing `>`.
    #[error("the link target is missing the closing '>'")]
    UnterminatedTarget,

    /// Indicates the link target is not a valid URL.
    #[error("the link target is not a valid URL")]
    InvalidTarget,

    /// Indicates a link parameter name was expected at the specified position.
    #[error("expected a link parameter at position {0}")]
    ExpectedParameter(usize),

    /// Indicates a link parameter value was expected at the specified position.
    #[error("expected a link parameter value at position {0}")]
    ExpectedValue(usize),

    /// Indicates the quoted string starting at the specified position is not terminated.
    #[error("the quoted string starting at position {0} is not terminated")]
    UnterminatedQuotedString(usize),

    /// Indicates an unexpected character was encountered at the specified position.
    #[error("unexpected character at position {0}")]
    UnexpectedCharacter(usize),

    /// Indicates the extended parameter starting at the specified position has a value that cannot be decoded.
    #[error("the extended parameter at position {0} has an invalid value")]
    InvalidExtendedValue(usize),

    /// Indicates the required link relation type is missing.
    #[error("the link relation type is missing")]
    MissingRelation,

    /// Indicates a single link was expected, but multiple links were found.
    #[error("expected a single link, but found {0}")]
    MultipleLinks(usize),
}

/// Represents a `link` header.
///
/// # Remarks
///
/// Refer to **Web Linking** in [RFC 8288](https://datatracker.ietf.org/doc/html/rfc8288).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Link {
    target: Uri,
    rel: String,
    languages: Vec<String>,
    extensions: HashMap<String, String>,

    /// Gets or sets the title of the link, if any.
    pub title: String,

    /// Gets or sets the intended destination medium or media.
    pub media: String,

    /// Gets or sets the media type associated with the link, if known.
    pub media_type: String,
}

impl Link {
    /// Initializes a new [LinkBuilder].
    ///
    /// # Arguments
    ///
    /// * `target` - the link target URL
    /// * `rel` - the link relation type that identifies the semantics of the link
    #[inline]
    pub fn builder<S: Into<String>>(target: Uri, rel: S) -> LinkBuilder {
        LinkBuilder::new(target, rel)
    }

    /// Initializes a new [Link].
    ///
    /// # Arguments
    ///
    /// * `target` - the link target URL
    /// * `rel` - the link relation type that identifies the semantics of the link
    pub fn new<S: Into<String>>(target: Uri, rel: S) -> Self {
        Self {
            target,
            rel: rel.into(),
            languages: Default::default(),
            title: Default::default(),
            media: Default::default(),
            media_type: Default::default(),
            extensions: Default::default(),
        }
    }

    /// Parses all of the links in the specified `link` header value.
    ///
    /// # Arguments
    ///
    /// * `text` - the text to parse, which may contain one or more comma-separated links
    ///
    /// # Remarks
    ///
    /// Parameter names are case-insensitive and normalized to lowercase. As required by [RFC 8288], only the first
    /// occurrence of the `rel`, `media`, `title`, `title*`, and `type` parameters is used; subsequent occurrences are
    /// ignored. Parameters without a value are ignored.
    ///
    /// The `title*` parameter is decoded as an [RFC 8187] extended value in the `UTF-8` or `ISO-8859-1` character set
    /// and takes precedence over the `title` parameter, regardless of order.
    ///
    /// [RFC 8288]: https://datatracker.ietf.org/doc/html/rfc8288#section-3
    /// [RFC 8187]: https://datatracker.ietf.org/doc/html/rfc8187#section-3.2
    pub fn parse_all(text: &str) -> Result<Vec<Self>, LinkParseError> {
        let mut parser = Parser::new(text);
        let mut link = None;
        let mut links = Vec::new();

        loop {
            parser.skip_whitespace();

            // the #rule allows empty list elements (e.g. "<a>; rel=x, , <b>; rel=y")
            if parser.eat(b',') {
                continue;
            } else if parser.peek().is_none() {
                break;
            }

            if let Some(first) = link.take() {
                links.push(first);
            }

            if links.is_empty() {
                link = Some(parser.link()?);
            } else {
                links.push(parser.link()?);
            }

            parser.skip_whitespace();

            match parser.peek() {
                None => break,
                Some(b',') => parser.pos += 1,
                Some(_) => return Err(LinkParseError::UnexpectedCharacter(parser.pos)),
            }
        }

        if let Some(one) = link {
            Ok(vec![one])
        } else if links.is_empty() {
            Err(LinkParseError::Empty)
        } else {
            Ok(links)
        }
    }

    /// Gets the link target URL.
    #[inline]
    pub fn target(&self) -> &Uri {
        &self.target
    }

    /// Gets the link relation type that identifies the semantics of the link.
    #[inline]
    pub fn rel(&self) -> &str {
        &self.rel
    }

    /// Gets the languages associated with the link, if any.
    #[inline]
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// Gets the link extensions, if any.
    #[inline]
    pub fn extensions(&self) -> &HashMap<String, String> {
        &self.extensions
    }

    /// Adds a new language to the link.
    ///
    /// # Arguments
    ///
    /// * `language` - the language to add
    ///
    /// # Remarks
    ///
    /// If `language` is empty, no operation is performed. It is recommended that [ISO 639] values be used.
    ///
    /// [ISO 639]: https://www.iso.org/standard/74575.html
    pub fn add_language<S: AsRef<str>>(&mut self, language: S) {
        fn add_lang(me: &mut Link, lang: &str) {
            if !lang.is_empty() {
                me.languages.push(lang.into());
            }
        }

        add_lang(self, language.as_ref());
    }

    /// Adds an extension with the specified key and value.
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
    pub fn add_extension<S: AsRef<str>>(&mut self, key: S, value: S) -> Result<(), AttributeNameError> {
        fn add_ext(me: &mut Link, k: &str, v: &str) -> Result<(), AttributeNameError> {
            if v.is_empty() {
                return Ok(());
            } else if k.is_empty() {
                return Err(Empty);
            }

            let mut chars = k.chars();

            if !chars.next().unwrap().is_ascii_alphabetic() {
                return Err(InvalidStart);
            }

            if chars.all(valid_char) {
                let _ = me.extensions.insert(k.into(), v.into());
                Ok(())
            } else {
                Err(InvalidCharacter)
            }
        }

        add_ext(self, key.as_ref(), value.as_ref())
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[inline]
        fn add_attr(builder: &mut Formatter<'_>, name: &str, value: &str) -> fmt::Result {
            builder.write_str("; ")?;
            builder.write_str(name)?;
            builder.write_str("=\"")?;

            // quoted-pair = "\" ( HTAB / SP / VCHAR / obs-text ); only '"' and '\' must be escaped
            let mut start = 0;

            for (i, ch) in value.char_indices() {
                if ch == '"' || ch == '\\' {
                    builder.write_str(&value[start..i])?;
                    builder.write_char('\\')?;
                    start = i;
                }
            }

            builder.write_str(&value[start..])?;
            builder.write_char('"')
        }

        // ext-value = charset "'" [ language ] "'" value-chars
        #[inline]
        fn add_ext_attr(builder: &mut Formatter<'_>, name: &str, value: &str) -> fmt::Result {
            builder.write_str("; ")?;
            builder.write_str(name)?;
            builder.write_str("*=UTF-8''")?;

            for byte in value.bytes() {
                if is_attr_char(byte) {
                    builder.write_char(byte as char)?;
                } else {
                    write!(builder, "%{byte:02X}")?;
                }
            }

            Ok(())
        }

        f.write_char('<')?;
        self.target.fmt(f)?;
        f.write_char('>')?;

        // rel is required by spec
        if self.rel.is_empty() {
            add_attr(f, "rel", "*")?;
        } else {
            add_attr(f, "rel", &self.rel)?;
        }

        if !self.languages.is_empty() {
            for lang in &self.languages {
                add_attr(f, "hreflang", lang)?;
            }
        }

        if !self.media.is_empty() {
            add_attr(f, "media", &self.media)?;
        }

        // a quoted-string cannot represent control characters and HeaderValue::to_str only accepts visible ASCII, so
        // any other title is encoded as an RFC 8187 extended value, which can represent any character losslessly
        if self.title.chars().all(is_quotable) {
            if !self.title.is_empty() {
                add_attr(f, "title", &self.title)?;
            }
        } else {
            add_ext_attr(f, "title", &self.title)?;
        }

        if !self.media_type.is_empty() {
            add_attr(f, "type", &self.media_type)?;
        }

        if !self.extensions.is_empty() {
            for (key, value) in &self.extensions {
                add_attr(f, key, value)?;
            }
        }

        Ok(())
    }
}

impl From<Link> for HeaderValue {
    /// Converts the [Link] into a [HeaderValue].
    ///
    /// # Remarks
    ///
    /// Any title can be represented because a title that is not visible ASCII is encoded as `title*`. Values of the
    /// other parameters should be visible ASCII. Non-ASCII characters in those values are written verbatim as UTF-8,
    /// which [HeaderValue] permits, but [HeaderValue::to_str] rejects, so the link cannot be parsed from the header.
    ///
    /// # Panics
    ///
    /// Panics if the relation type, a language, the media, the media type, or an extension value contains a control
    /// character other than horizontal tab (e.g. `\r` or `\n`). HTTP field values cannot contain these characters, nor
    /// can they be escaped in a quoted string.
    fn from(value: Link) -> Self {
        Self::try_from(value.to_string()).unwrap()
    }
}

impl FromStr for Link {
    type Err = LinkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut links = Self::parse_all(s)?;

        if links.len() == 1 {
            Ok(links.remove(0))
        } else {
            Err(LinkParseError::MultipleLinks(links.len()))
        }
    }
}

impl TryFrom<HeaderValue> for Link {
    type Error = Box<dyn Error + Send + Sync + 'static>;

    #[inline]
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&HeaderValue> for Link {
    type Error = Box<dyn Error + Send + Sync + 'static>;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let text = value.to_str().map_err(Box::new)?;
        Ok(text.parse::<Self>()?)
    }
}

/// Represents a parser for the `link` header grammar defined in [RFC 8288].
///
/// [RFC 8288]: https://datatracker.ietf.org/doc/html/rfc8288#section-3
struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn eat(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    // link-value = "<" URI-Reference ">" *( OWS ";" OWS link-param )
    fn link(&mut self) -> Result<Link, LinkParseError> {
        if !self.eat(b'<') {
            return Err(LinkParseError::ExpectedTarget(self.pos));
        }

        let start = self.pos;
        let end = match self.text[start..].find('>') {
            Some(offset) => start + offset,
            None => return Err(LinkParseError::UnterminatedTarget),
        };
        let target = Uri::try_from(&self.text[start..end]).map_err(|_| LinkParseError::InvalidTarget)?;
        let mut link = Link::new(target, String::new());
        let mut rel = None;
        let mut title = None;

        self.pos = end + 1;

        loop {
            self.skip_whitespace();

            if !self.eat(b';') {
                break;
            }

            self.skip_whitespace();

            let start = self.pos;
            let (name, value) = self.param()?;

            if value.is_empty() {
                continue;
            }

            match name.as_str() {
                "rel" => {
                    if rel.is_none() {
                        rel = Some(value);
                    }
                }
                "hreflang" => link.add_language(value),
                "media" => {
                    if link.media.is_empty() {
                        link.media = value;
                    }
                }
                "title" => {
                    if link.title.is_empty() {
                        link.title = value;
                    }
                }
                "title*" => {
                    if title.is_none() {
                        title = Some(decode_ext_value(&value).ok_or(LinkParseError::InvalidExtendedValue(start))?);
                    }
                }
                "type" => {
                    if link.media_type.is_empty() {
                        link.media_type = value;
                    }
                }
                _ => {
                    let _ = link.extensions.entry(name).or_insert(value);
                }
            }
        }

        // RFC 8288 §3.4.1: title* is preferred over title when both are present
        if let Some(title) = title
            && !title.is_empty()
        {
            link.title = title;
        }

        match rel {
            Some(rel) => {
                link.rel = rel;
                Ok(link)
            }
            None => Err(LinkParseError::MissingRelation),
        }
    }

    // link-param = token BWS [ "=" BWS ( token / quoted-string ) ]
    fn param(&mut self) -> Result<(String, String), LinkParseError> {
        self.skip_whitespace();

        let name = self.token();

        if name.is_empty() {
            return Err(LinkParseError::ExpectedParameter(self.pos));
        }

        let name = name.to_ascii_lowercase();

        self.skip_whitespace();

        if !self.eat(b'=') {
            return Ok((name, String::new()));
        }

        self.skip_whitespace();

        if self.peek() == Some(b'"') {
            return Ok((name, self.quoted_string()?));
        }

        let value = self.token();

        if value.is_empty() {
            Err(LinkParseError::ExpectedValue(self.pos))
        } else {
            Ok((name, value.into()))
        }
    }

    // token = 1*tchar
    fn token(&mut self) -> &'a str {
        let start = self.pos;

        while self.peek().is_some_and(is_tchar) {
            self.pos += 1;
        }

        &self.text[start..self.pos]
    }

    // quoted-string = DQUOTE *( qdtext / quoted-pair ) DQUOTE
    fn quoted_string(&mut self) -> Result<String, LinkParseError> {
        let start = self.pos;
        let content = start + 1;
        let mut value = String::new();
        let mut chars = self.text[content..].char_indices();

        while let Some((offset, ch)) = chars.next() {
            match ch {
                '"' => {
                    self.pos = content + offset + 1;
                    return Ok(value);
                }
                '\\' => match chars.next() {
                    Some((_, escaped)) => value.push(escaped),
                    None => break,
                },
                _ => value.push(ch),
            }
        }

        Err(LinkParseError::UnterminatedQuotedString(start))
    }
}

// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA
const fn is_tchar(ch: u8) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
        )
}

// attr-char = ALPHA / DIGIT / "!" / "#" / "$" / "&" / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
const fn is_attr_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
        )
}

// characters that can appear in a quoted-string and be read back with HeaderValue::to_str
const fn is_quotable(ch: char) -> bool {
    ch == '\t' || matches!(ch, ' '..='~')
}

// ext-value = charset "'" [ language ] "'" value-chars
fn decode_ext_value(value: &str) -> Option<String> {
    fn hex(digit: u8) -> Option<u8> {
        (digit as char).to_digit(16).map(|d| d as u8)
    }

    let mut parts = value.splitn(3, '\'');
    let charset = parts.next()?;
    let _language = parts.next()?;
    let mut encoded = parts.next()?.bytes();
    let mut bytes = Vec::new();

    while let Some(byte) = encoded.next() {
        if byte == b'%' {
            let high = hex(encoded.next()?)?;
            let low = hex(encoded.next()?)?;
            bytes.push(high << 4 | low);
        } else if is_attr_char(byte) {
            bytes.push(byte);
        } else {
            return None;
        }
    }

    if charset.eq_ignore_ascii_case("UTF-8") {
        String::from_utf8(bytes).ok()
    } else if charset.eq_ignore_ascii_case("ISO-8859-1") {
        Some(bytes.into_iter().map(char::from).collect())
    } else {
        None
    }
}

/// Represents a link builder.
pub struct LinkBuilder(Link);

impl LinkBuilder {
    /// Initializes a new [LinkBuilder].
    ///
    /// # Arguments
    ///
    /// * `target` - the link target URL
    /// * `rel` - the link relation type that identifies the semantics of the link
    pub fn new<S: Into<String>>(target: Uri, rel: S) -> Self {
        Self(Link::new(target, rel))
    }

    /// Adds a language to the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the language to add
    pub fn add_language<S: AsRef<str>>(mut self, value: S) -> Self {
        self.0.add_language(value);
        self
    }

    /// Sets the title of the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the link title
    pub fn title<S: Into<String>>(mut self, value: S) -> Self {
        self.0.title = value.into();
        self
    }

    /// Sets the media of the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the link media
    pub fn media<S: Into<String>>(mut self, value: S) -> Self {
        self.0.media = value.into();
        self
    }

    /// Sets the media type of the link being built.
    ///
    /// # Arguments
    ///
    /// * `value` - the link media type
    pub fn media_type<S: Into<String>>(mut self, value: S) -> Self {
        self.0.media_type = value.into();
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
    pub fn add_extension<S: AsRef<str>>(mut self, key: S, value: S) -> Result<Self, AttributeNameError> {
        self.0.add_extension(key, value).map(|_| self)
    }

    /// Builds the completed link.
    pub fn build(self) -> Link {
        self.0
    }
}

impl From<LinkBuilder> for Link {
    #[inline]
    fn from(builder: LinkBuilder) -> Self {
        builder.0
    }
}

impl From<Link> for LinkBuilder {
    #[inline]
    fn from(link: Link) -> Self {
        Self(link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn into_header_should_use_absolute_url() {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let link = Link::new(url, "test");
        let expected = HeaderValue::from_static(r#"<http://tempuri.org/test>; rel="test""#);

        // act
        let header = HeaderValue::from(link);

        // assert
        assert_eq!(header, expected);
    }

    #[test]
    fn into_header_should_use_all_target_attributes() {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let link: Link = LinkBuilder::new(url, "test")
            .add_language("en")
            .media("screen")
            .title("Test Case")
            .media_type("text/plain")
            .add_extension("ext", "42")
            .unwrap()
            .into();
        let expected = HeaderValue::from_static(concat!(
            r#"<http://tempuri.org/test>; rel="test"; hreflang="en"; media="screen"; "#,
            r#"title="Test Case"; type="text/plain"; ext="42""#
        ));

        // act
        let header = HeaderValue::from(link);

        // assert
        assert_eq!(header, expected);
    }

    #[test]
    fn into_header_should_add_multiple_languages() {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let link: Link = LinkBuilder::new(url, "test")
            .add_language("en")
            .add_language("es")
            .into();
        let expected =
            HeaderValue::from_static(r#"<http://tempuri.org/test>; rel="test"; hreflang="en"; hreflang="es""#);

        // act
        let header = HeaderValue::from(link);

        // assert
        assert_eq!(header, expected);
    }

    #[rstest]
    #[case("Test Case", r#"title="Test Case""#)]
    #[case("", r#"rel="test""#)]
    #[case(r#"say "hello""#, r#"title="say \"hello\"""#)]
    #[case(r#"C:\temp\"#, r#"title="C:\\temp\\""#)]
    #[case(r#""\"#, r#"title="\"\\""#)]
    fn to_string_should_escape_quoted_values(#[case] title: &str, #[case] expected: &str) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let link = LinkBuilder::new(url, "test").title(title).build();

        // act
        let text = link.to_string();

        // assert
        assert!(text.ends_with(expected), "{text}");
    }

    #[test]
    fn into_header_should_escape_all_quoted_values() {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let link: Link = LinkBuilder::new(url, r#"a"b"#)
            .add_language(r#"e\n"#)
            .media(r#"m"m"#)
            .title(r#"t\t"#)
            .media_type(r#"x"y"#)
            .add_extension("ext", r#"4"2"#)
            .unwrap()
            .into();
        let expected = HeaderValue::from_static(concat!(
            r#"<http://tempuri.org/test>; rel="a\"b"; hreflang="e\\n"; media="m\"m"; title="t\\t"; "#,
            r#"type="x\"y"; ext="4\"2""#,
        ));

        // act
        let header = HeaderValue::from(link);

        // assert
        assert_eq!(header, expected);
    }

    #[rstest]
    #[case("ext")]
    #[case("ext42")]
    #[case("ext-val")]
    #[case("ext-val-42")]
    #[case("ext_val")]
    #[case("ext_val_42")]
    #[case("ext.val")]
    #[case("ext.val.42")]
    fn add_extension_should_allow_valid_key(#[case] key: &str) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let mut link = Link::new(url, "test");

        // act
        let result = link.add_extension(key, "test");

        // assert
        assert!(result.is_ok());
        assert_eq!(link.extensions().len(), 1);
    }

    #[rstest]
    #[case("", Empty)]
    #[case("42", InvalidStart)]
    #[case("extra key", InvalidCharacter)]
    fn add_extension_should_not_allow_key(#[case] key: &str, #[case] expected: AttributeNameError) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let mut link = Link::new(url, "test");

        // act
        let actual = link.add_extension(key, "test").unwrap_err();

        // assert
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case("ext")]
    #[case("")]
    #[case("42")]
    fn add_extension_should_ignore_empty_value(#[case] key: &str) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let mut link = Link::new(url, "test");

        // act
        let result = link.add_extension(key, "");

        // assert
        assert!(result.is_ok());
        assert!(link.extensions().is_empty());
    }

    #[test]
    fn to_string_should_use_wildcard_when_relation_type_is_empty() {
        // arrange
        let link = Link::new("http://tempuri.org/test".parse().unwrap(), "");

        // act
        let text = link.to_string();

        // assert
        assert_eq!(text, r#"<http://tempuri.org/test>; rel="*""#);
    }

    #[test]
    fn builder_should_create_link_with_target_and_relation_type() {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();

        // act
        let link = Link::builder(url.clone(), "test").build();

        // assert
        assert_eq!(link, Link::new(url, "test"));
    }

    #[test]
    fn builder_should_initialize_from_existing_link() {
        // arrange
        let mut link = Link::new("http://tempuri.org/test".parse().unwrap(), "test");

        link.title = "Test".into();

        // act
        let rebuilt = LinkBuilder::from(link.clone()).media("screen").build();

        // assert
        assert_eq!(rebuilt.title, "Test");
        assert_eq!(rebuilt.media, "screen");
        assert_eq!(rebuilt.target(), link.target());
        assert_eq!(rebuilt.rel(), link.rel());
    }

    #[test]
    fn add_language_should_ignore_empty_value() {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let mut link = Link::new(url, "test");

        // act
        link.add_language("");

        // assert
        assert!(link.languages().is_empty());
    }

    #[test]
    fn parse_should_read_target_and_rel() {
        // arrange
        let text = r#"<http://tempuri.org/test>; rel="test""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.target(), "http://tempuri.org/test");
        assert_eq!(link.rel(), "test");
        assert!(link.languages().is_empty());
        assert!(link.extensions().is_empty());
        assert!(link.title.is_empty());
        assert!(link.media.is_empty());
        assert!(link.media_type.is_empty());
    }

    #[test]
    fn parse_should_read_all_target_attributes() {
        // arrange
        let text = concat!(
            r#"<http://tempuri.org/test>; rel="test"; hreflang="en"; hreflang="es"; "#,
            r#"media="screen"; title="Test Case"; type="text/plain"; ext="42""#,
        );

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.rel(), "test");
        assert_eq!(link.languages(), ["en", "es"]);
        assert_eq!(link.media, "screen");
        assert_eq!(link.title, "Test Case");
        assert_eq!(link.media_type, "text/plain");
        assert_eq!(link.extensions().get("ext").map(String::as_str), Some("42"));
    }

    #[rstest]
    #[case(r#"<http://tempuri.org/test>;rel=test"#)]
    #[case(r#"<http://tempuri.org/test>; rel=test"#)]
    #[case(r#"<http://tempuri.org/test> ; rel = "test" "#)]
    #[case("\t<http://tempuri.org/test>\t;\trel=\"test\"\t")]
    #[case(r#"<http://tempuri.org/test>; REL="test""#)]
    #[case(r#"<http://tempuri.org/test>; Rel=test"#)]
    fn parse_should_allow_whitespace_case_and_token_variations(#[case] text: &str) {
        // arrange
        let expected = Link::new("http://tempuri.org/test".parse().unwrap(), "test");

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link, expected);
    }

    #[test]
    fn parse_should_unescape_quoted_string() {
        // arrange
        let text = r#"<http://tempuri.org/test>; rel="test"; title="say \"hello\", \\world\\; ok""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.title, r#"say "hello", \world\; ok"#);
    }

    #[test]
    fn parse_should_allow_non_ascii_quoted_string() {
        // arrange
        let text = r#"<http://tempuri.org/test>; rel="test"; title="café ☕""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.title, "café ☕");
    }

    #[test]
    fn parse_should_allow_relative_target() {
        // arrange
        let text = r#"</api/test?api-version=1.0>; rel="sunset""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.target(), "/api/test?api-version=1.0");
        assert_eq!(link.rel(), "sunset");
    }

    #[test]
    fn parse_should_preserve_multiple_relation_types() {
        // arrange
        let text = r#"<http://tempuri.org/test>; rel="start http://tempuri.org/rel/test""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.rel(), "start http://tempuri.org/rel/test");
    }

    #[test]
    fn parse_should_ignore_subsequent_occurrences_of_singular_parameters() {
        // arrange
        let text = concat!(
            r#"<http://tempuri.org/test>; rel="first"; rel="second"; media="screen"; media="print"; "#,
            r#"title="One"; title="Two"; type="text/plain"; type="text/html"; ext="1"; ext="2""#,
        );

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.rel(), "first");
        assert_eq!(link.media, "screen");
        assert_eq!(link.title, "One");
        assert_eq!(link.media_type, "text/plain");
        assert_eq!(link.extensions().get("ext").map(String::as_str), Some("1"));
    }

    #[test]
    fn parse_should_normalize_extension_names_to_lowercase() {
        // arrange
        let text = r#"<http://tempuri.org/test>; rel="test"; Label*=UTF-8'en'Test; X-Ext="42""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(
            link.extensions().get("label*").map(String::as_str),
            Some("UTF-8'en'Test")
        );
        assert_eq!(link.extensions().get("x-ext").map(String::as_str), Some("42"));
    }

    #[rstest]
    #[case(r#"<http://tempuri.org/test>; rel="test"; flag"#)]
    #[case(r#"<http://tempuri.org/test>; rel="test"; title="""#)]
    fn parse_should_ignore_parameters_without_value(#[case] text: &str) {
        // arrange
        let expected = Link::new("http://tempuri.org/test".parse().unwrap(), "test");

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link, expected);
    }

    #[rstest]
    #[case("", LinkParseError::Empty)]
    #[case(" , ,", LinkParseError::Empty)]
    #[case(r#"http://tempuri.org/test; rel="test""#, LinkParseError::ExpectedTarget(0))]
    #[case(r#"<http://tempuri.org/test; rel="test""#, LinkParseError::UnterminatedTarget)]
    #[case(r#"<>; rel="test""#, LinkParseError::InvalidTarget)]
    #[case(r#"<http://tempuri.org/te st>; rel="test""#, LinkParseError::InvalidTarget)]
    #[case(r#"<http://tempuri.org/test>"#, LinkParseError::MissingRelation)]
    #[case(r#"<http://tempuri.org/test>; title="Test""#, LinkParseError::MissingRelation)]
    #[case(r#"<http://tempuri.org/test>; rel="""#, LinkParseError::MissingRelation)]
    #[case(r#"<http://tempuri.org/test>; ="test""#, LinkParseError::ExpectedParameter(27))]
    #[case(r#"<http://tempuri.org/test>; rel="test";"#, LinkParseError::ExpectedParameter(38))]
    #[case(r#"<http://tempuri.org/test>; rel="#, LinkParseError::ExpectedValue(31))]
    #[case(r#"<http://tempuri.org/test>; rel=;"#, LinkParseError::ExpectedValue(31))]
    #[case(
        r#"<http://tempuri.org/test>; rel="test"#,
        LinkParseError::UnterminatedQuotedString(31)
    )]
    #[case(
        r#"<http://tempuri.org/test>; rel="test\""#,
        LinkParseError::UnterminatedQuotedString(31)
    )]
    #[case(
        r#"<http://tempuri.org/test>; rel="test\"#,
        LinkParseError::UnterminatedQuotedString(31)
    )]
    #[case(
        r#"<http://tempuri.org/test>; rel="test" x"#,
        LinkParseError::UnexpectedCharacter(38)
    )]
    #[case(r#"<http://tempuri.org/test>; rel=test@"#, LinkParseError::UnexpectedCharacter(35))]
    fn parse_should_fail_for_invalid_value(#[case] text: &str, #[case] expected: LinkParseError) {
        // arrange

        // act
        let error = text.parse::<Link>().unwrap_err();

        // assert
        assert_eq!(error, expected);
    }

    #[test]
    fn parse_should_fail_for_multiple_links() {
        // arrange
        let text = r#"<http://tempuri.org/1>; rel="first", <http://tempuri.org/2>; rel="second""#;

        // act
        let error = text.parse::<Link>().unwrap_err();

        // assert
        assert_eq!(error, LinkParseError::MultipleLinks(2));
    }

    #[test]
    fn parse_all_should_read_multiple_links() {
        // arrange
        let text =
            r#"<http://tempuri.org/1>; rel="first"; title="One, Two", , <http://tempuri.org/2?a=1,2>; rel=second"#;

        // act
        let links = Link::parse_all(text).unwrap();

        // assert
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target(), "http://tempuri.org/1");
        assert_eq!(links[0].rel(), "first");
        assert_eq!(links[0].title, "One, Two");
        assert_eq!(links[1].target(), "http://tempuri.org/2?a=1,2");
        assert_eq!(links[1].rel(), "second");
    }

    #[test]
    fn parse_all_should_fail_if_any_link_is_invalid() {
        // arrange
        let text = r#"<http://tempuri.org/1>; rel="first", <http://tempuri.org/2>"#;

        // act
        let error = Link::parse_all(text).unwrap_err();

        // assert
        assert_eq!(error, LinkParseError::MissingRelation);
    }

    #[test]
    fn try_from_header_should_parse_link() {
        // arrange
        let header = HeaderValue::from_static(r#"<http://tempuri.org/test>; rel="test"; hreflang="en""#);
        let expected = LinkBuilder::new("http://tempuri.org/test".parse().unwrap(), "test")
            .add_language("en")
            .build();

        // act
        let link = Link::try_from(header).unwrap();

        // assert
        assert_eq!(link, expected);
    }

    #[test]
    fn try_from_header_should_fail_for_invalid_link() {
        // arrange
        let header = HeaderValue::from_static("<http://tempuri.org/test>");

        // act
        let error = Link::try_from(&header).unwrap_err();

        // assert
        assert_eq!(
            error.downcast_ref::<LinkParseError>(),
            Some(&LinkParseError::MissingRelation)
        );
    }

    #[test]
    fn try_from_header_should_fail_for_non_visible_ascii() {
        // arrange
        let header = HeaderValue::from_bytes(b"<http://tempuri.org/test>; rel=\"t\xe9st\"").unwrap();

        // act
        let result = Link::try_from(&header);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn link_should_round_trip_through_header() {
        // arrange
        let expected = LinkBuilder::new("http://tempuri.org/test".parse().unwrap(), "test")
            .add_language("en")
            .add_language("es")
            .media("screen")
            .title("Test Case")
            .media_type("text/plain")
            .add_extension("ext", "42")
            .unwrap()
            .add_extension("other", "value")
            .unwrap()
            .build();
        let header = HeaderValue::from(expected.clone());

        // act
        let link = Link::try_from(header).unwrap();

        // assert
        assert_eq!(link, expected);
    }

    #[test]
    fn link_with_escaped_values_should_round_trip_through_header() {
        // arrange
        let expected = LinkBuilder::new("http://tempuri.org/test".parse().unwrap(), r#"te"st"#)
            .add_language(r#"e\n"#)
            .media(r#""screen""#)
            .title(r#"say "hello", \world\; ok"#)
            .media_type(r#"text\"plain"#)
            .add_extension("ext", r#"\"#)
            .unwrap()
            .build();
        let header = HeaderValue::from(expected.clone());

        // act
        let link = Link::try_from(header).unwrap();

        // assert
        assert_eq!(link, expected);
    }

    #[rstest]
    #[case("a\tb", "; title=\"a\tb\"")]
    #[case("café ☕", "; title*=UTF-8''caf%C3%A9%20%E2%98%95")]
    #[case("line 1\nline 2", "; title*=UTF-8''line%201%0Aline%202")]
    #[case("\r\n\0\x7f", "; title*=UTF-8''%0D%0A%00%7F")]
    #[case("!#$&+-.^_`|~\n", "; title*=UTF-8''!#$&+-.^_`|~%0A")]
    #[case("'%*\"\\\n", "; title*=UTF-8''%27%25%2A%22%5C%0A")]
    fn to_string_should_encode_title_that_cannot_be_quoted(#[case] title: &str, #[case] expected: &str) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let link = LinkBuilder::new(url, "test").title(title).build();

        // act
        let text = link.to_string();

        // assert
        assert_eq!(text, format!(r#"<http://tempuri.org/test>; rel="test"{expected}"#));
    }

    #[rstest]
    #[case("title*=UTF-8''caf%C3%A9%20%E2%98%95", "café ☕")]
    #[case("title*=utf-8'en'caf%c3%a9", "café")]
    #[case("title*=UTF-8''line%201%0Aline%202", "line 1\nline 2")]
    #[case("title*=iso-8859-1'en'%A3%20rates", "£ rates")]
    #[case("title*=UTF-8'de'Test", "Test")]
    #[case("title*=\"UTF-8''Test\"", "Test")]
    fn parse_should_decode_extended_title(#[case] param: &str, #[case] expected: &str) {
        // arrange
        let text = format!(r#"<http://tempuri.org/test>; rel="test"; {param}"#);

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.title, expected);
        assert!(link.extensions().is_empty());
    }

    #[rstest]
    #[case(r#"title="Plain"; title*=UTF-8''Extended"#)]
    #[case(r#"title*=UTF-8''Extended; title="Plain""#)]
    #[case(r#"title*=UTF-8''Extended; title*=UTF-8''Ignored"#)]
    #[case(r#"title*=UTF-8''Extended; title*=UTF-8''%ZZ"#)]
    fn parse_should_prefer_first_extended_title(#[case] params: &str) {
        // arrange
        let text = format!(r#"<http://tempuri.org/test>; rel="test"; {params}"#);

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.title, "Extended");
    }

    #[test]
    fn parse_should_use_title_when_extended_title_is_empty() {
        // arrange
        let text = r#"<http://tempuri.org/test>; rel="test"; title*=UTF-8''; title="Plain""#;

        // act
        let link: Link = text.parse().unwrap();

        // assert
        assert_eq!(link.title, "Plain");
    }

    #[rstest]
    #[case("UTF-8Test")]
    #[case("UTF-8'Test")]
    #[case("UTF-16''Test")]
    #[case("''Test")]
    #[case("UTF-8''%C3")]
    #[case("UTF-8''%G1")]
    #[case("UTF-8''%4")]
    #[case("UTF-8''a*b")]
    #[case("UTF-8''a'b")]
    fn parse_should_fail_for_invalid_extended_title(#[case] value: &str) {
        // arrange
        let text = format!(r#"<http://tempuri.org/test>; rel="test"; title*={value}"#);

        // act
        let error = text.parse::<Link>().unwrap_err();

        // assert
        assert_eq!(error, LinkParseError::InvalidExtendedValue(39));
    }

    #[rstest]
    #[case("café ☕")]
    #[case("line 1\r\nline 2")]
    #[case("tab\there")]
    #[case("\0\x07\x1b\x7f")]
    #[case("say \"hello\" \\ 'world' 100%")]
    #[case("日本語のタイトル")]
    fn link_with_any_title_should_round_trip_through_header(#[case] title: &str) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let expected = LinkBuilder::new(url, "test").title(title).build();
        let header = HeaderValue::from(expected.clone());

        // act
        let link = Link::try_from(&header).unwrap();

        // assert
        assert!(header.to_str().is_ok());
        assert_eq!(link, expected);
    }

    #[rstest]
    #[case("rel")]
    #[case("hreflang")]
    #[case("media")]
    #[case("type")]
    #[case("ext")]
    #[should_panic(expected = "InvalidHeaderValue")]
    fn into_header_should_panic_for_control_character(#[case] param: &str) {
        // arrange
        let url: Uri = "http://tempuri.org/test".parse().unwrap();
        let value = "line 1\nline 2";
        let mut link = Link::new(url, if param == "rel" { value } else { "test" });

        match param {
            "hreflang" => link.add_language(value),
            "media" => link.media = value.into(),
            "type" => link.media_type = value.into(),
            "ext" => link.add_extension("ext", value).unwrap(),
            _ => {}
        }

        // act
        let _ = HeaderValue::from(link);

        // assert
    }
}
