use crate::{DocumentOptions, options::should_render};
use api_versioning::{
    headers::Link,
    policy::{DeprecationPolicy, SunsetPolicy},
};
use serde_json::{Map, Value, json};

/// Appends a sentence to the specified text, separating it from any existing sentence.
fn add_sentence(text: &mut String, sentence: &str) {
    if !text.is_empty() {
        if !text.ends_with('.') {
            text.push('.');
        }

        text.push(' ');
    }

    text.push_str(sentence);
}

/// Renders a link as a Markdown list item.
fn render(link: &Link) -> String {
    let url = link.target().to_string();

    if !link.title.is_empty() {
        format!("- [{}]({url})", link.title)
    } else if link.target().scheme().is_some() {
        format!("- {url}")
    } else {
        format!("- <a href=\"{url}\">{url}</a>")
    }
}

/// Creates the Markdown description of a document.
///
/// # Arguments
///
/// * `description` - the existing description, if any
/// * `deprecated` - indicates whether the API version is deprecated
/// * `deprecation` - the deprecation policy of the API version, if any
/// * `sunset` - the sunset policy of the API version, if any
/// * `options` - the [options](DocumentOptions) used to create the description
pub(crate) fn description(
    description: Option<&str>,
    deprecated: bool,
    deprecation: Option<&DeprecationPolicy>,
    sunset: Option<&SunsetPolicy>,
    options: &DocumentOptions,
) -> Option<String> {
    let mut text = description.unwrap_or_default().to_owned();
    let mut links = Vec::new();

    if let Some(policy) = deprecation {
        if let Some(notice) = (options.deprecation_notice)(policy).filter(|notice| !notice.is_empty()) {
            add_sentence(&mut text, &notice);
        }

        if !options.hide_policy_links {
            links.extend(policy.links().iter().filter(|link| should_render(link)).map(render));
        }
    } else if deprecated
        && let Some(notice) = (options.deprecation_notice)(&DeprecationPolicy::default()).filter(|n| !n.is_empty())
    {
        add_sentence(&mut text, &notice);
    }

    if let Some(policy) = sunset {
        if let Some(notice) = (options.sunset_notice)(policy).filter(|notice| !notice.is_empty()) {
            add_sentence(&mut text, &notice);
        }

        if !options.hide_policy_links {
            links.extend(policy.links().iter().filter(|link| should_render(link)).map(render));
        }
    }

    if !links.is_empty() {
        text.push_str("\n\n### Links\n\n");
        text.push_str(&links.join("\n"));
    }

    (!text.is_empty()).then_some(text)
}

/// Converts a link into the JSON of the `x-api-versioning` extension.
fn to_json(link: &Link) -> Value {
    let mut object = Map::new();

    if !link.title.is_empty() {
        object.insert("title".into(), link.title.clone().into());
    }

    if !link.media_type.is_empty() {
        object.insert("type".into(), link.media_type.clone().into());
    }

    object.insert("rel".into(), link.rel().into());
    object.insert("url".into(), link.target().to_string().into());

    if !link.media.is_empty() {
        object.insert("media".into(), link.media.clone().into());
    }

    if !link.languages().is_empty() {
        object.insert("lang".into(), link.languages().into());
    }

    let mut extensions: Vec<_> = link.extensions().iter().collect();

    extensions.sort();

    for (key, value) in extensions {
        object.insert(key.clone(), value.clone().into());
    }

    Value::Object(object)
}

/// Creates the value of the `x-api-versioning` extension of a document, if there are any policy links.
pub(crate) fn extension(deprecation: Option<&DeprecationPolicy>, sunset: Option<&SunsetPolicy>) -> Option<Value> {
    let links: Vec<_> = deprecation
        .map(DeprecationPolicy::links)
        .into_iter()
        .chain(sunset.map(SunsetPolicy::links))
        .flatten()
        .map(to_json)
        .collect();

    (!links.is_empty()).then(|| json!({ "links": links }))
}
