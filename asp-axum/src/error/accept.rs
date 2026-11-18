use http::{HeaderMap, header::ACCEPT};

// the media types that problem details serialized as JSON can be written as
const MEDIA_TYPES: [(&str, &str); 2] = [("application", "problem+json"), ("application", "json")];

/// Represents a media range in an `Accept` header.
struct MediaRange<'a> {
    kind: &'a str,
    subtype: &'a str,
    quality: f32,
}

impl MediaRange<'_> {
    // gets the specificity of the range when it matches the media type: */* = 0, type/* = 1, type/subtype = 2
    fn specificity(&self, kind: &str, subtype: &str) -> Option<u8> {
        if self.kind == "*" && self.subtype == "*" {
            Some(0)
        } else if !self.kind.eq_ignore_ascii_case(kind) {
            None
        } else if self.subtype == "*" {
            Some(1)
        } else if self.subtype.eq_ignore_ascii_case(subtype) {
            Some(2)
        } else {
            None
        }
    }
}

fn parse(text: &str) -> Option<MediaRange<'_>> {
    let mut parameters = text.split(';');
    let (kind, subtype) = parameters.next()?.trim().split_once('/')?;
    let (kind, subtype) = (kind.trim(), subtype.trim());

    if kind.is_empty() || subtype.is_empty() || (kind == "*" && subtype != "*") {
        return None;
    }

    let mut quality = 1.0;

    for parameter in parameters {
        if let Some((name, value)) = parameter.split_once('=')
            && name.trim().eq_ignore_ascii_case("q")
        {
            quality = value.trim().parse::<f32>().ok().filter(|q| (0.0..=1.0).contains(q))?;
        }
    }

    Some(MediaRange { kind, subtype, quality })
}

/// Determines whether the specified request headers accept problem details serialized as JSON.
///
/// # Remarks
///
/// Problem details are accepted when there is no valid media range or when the most specific media range that matches
/// a supported media type has a nonzero quality.
pub(super) fn problem_details(headers: &HeaderMap) -> bool {
    let ranges = headers
        .get_all(ACCEPT)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .filter_map(parse)
        .collect::<Vec<_>>();

    if ranges.is_empty() {
        return true;
    }

    MEDIA_TYPES.iter().any(|(kind, subtype)| {
        let mut best: Option<(u8, f32)> = None;

        for range in &ranges {
            if let Some(specificity) = range.specificity(kind, subtype) {
                best = match best {
                    Some((most, quality))
                        if most > specificity || (most == specificity && quality >= range.quality) =>
                    {
                        best
                    }
                    _ => Some((specificity, range.quality)),
                };
            }
        }

        best.is_some_and(|(_, quality)| quality > 0.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;
    use rstest::rstest;

    fn headers(values: &[&'static str]) -> HeaderMap {
        let mut headers = HeaderMap::new();

        for value in values {
            headers.append(ACCEPT, HeaderValue::from_static(value));
        }

        headers
    }

    #[rstest]
    #[case(&[])]
    #[case(&[""])]
    #[case(&["application/problem+json"])]
    #[case(&["application/json"])]
    #[case(&["APPLICATION/JSON"])]
    #[case(&["application/*"])]
    #[case(&["*/*"])]
    #[case(&["text/html, application/json;q=0.5"])]
    #[case(&["text/html, */*;q=0.1"])]
    #[case(&["text/html", "application/json"])]
    #[case(&["application/json;q=0, application/problem+json"])]
    #[case(&["application/*;q=0, application/json"])]
    #[case(&["application/json; charset=utf-8"])]
    #[case(&["not a media range"])]
    fn problem_details_should_be_accepted(#[case] values: &[&'static str]) {
        // arrange
        let headers = headers(values);

        // act
        let accepted = problem_details(&headers);

        // assert
        assert!(accepted);
    }

    #[rstest]
    #[case(&["text/html"])]
    #[case(&["application/xml"])]
    #[case(&["application/problem+xml"])]
    #[case(&["text/*"])]
    #[case(&["application/json;q=0"])]
    #[case(&["application/*;q=0"])]
    #[case(&["*/*;q=0"])]
    #[case(&["application/json;q=0, application/problem+json;q=0, */*"])]
    #[case(&["text/html, application/json;q=invalid"])]
    #[case(&["text/html, application/json;q=2"])]
    fn problem_details_should_not_be_accepted(#[case] values: &[&'static str]) {
        // arrange
        let headers = headers(values);

        // act
        let accepted = problem_details(&headers);

        // assert
        assert!(!accepted);
    }
}
