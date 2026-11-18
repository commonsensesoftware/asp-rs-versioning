use crate::parameters::VersionParameters;
use crate::{ApiDocument, DocumentOptions, markdown, operation::Operation};
use api_versioning::{
    ApiVersion, Mapping,
    parameter::Descriptor,
    policy::{Manager, Policy},
};
use api_versioning_axum::{ApiDescription, ApiDescriptions};
use http::Method;
use utoipa::openapi::{Components, Deprecated, OpenApi, extensions::Extensions, path::HttpMethod, tag::Tag};

/// The name of the OpenAPI extension that contains the links of API versioning policies.
const EXTENSION: &str = "x-api-versioning";

/// Creates an OpenAPI document for each API version using the default [options](DocumentOptions).
///
/// # Arguments
///
/// * `apis` - the [descriptions](ApiDescriptions) of the versioned APIs
/// * `base` - the [OpenAPI](OpenApi) document each document starts from, such as one derived with `utoipa::OpenApi`
///
/// # Remarks
///
/// A document is created for each API version declared by the APIs, in ascending order. API versions that are only
/// advertised are implemented elsewhere, so they do not have a document.
///
/// The operations and the schemas they reference are created in the API version of each document, so they only have
/// the members that are visible in it. They replace the schemas of the base document, whose other schemas are unchanged.
///
/// # Panics
///
/// Panics if a handler is added for an HTTP method that its `#[utoipa::path]` operation is not declared for.
pub fn documents(apis: &ApiDescriptions, base: OpenApi) -> Vec<ApiDocument> {
    documents_with(apis, base, &DocumentOptions::default())
}

/// Creates an OpenAPI document for each API version.
///
/// # Arguments
///
/// * `apis` - the [descriptions](ApiDescriptions) of the versioned APIs
/// * `base` - the [OpenAPI](OpenApi) document each document starts from, such as one derived with `utoipa::OpenApi`
/// * `options` - the [options](DocumentOptions) used to create the documents
///
/// # Remarks
///
/// A document is created for each API version declared by the APIs, in ascending order. API versions that are only
/// advertised are implemented elsewhere, so they do not have a document.
///
/// The operations and the schemas they reference are created in the API version of each document, so they only have
/// the members that are visible in it. They replace the schemas of the base document, whose other schemas are unchanged.
///
/// # Panics
///
/// Panics if a handler is added for an HTTP method that its `#[utoipa::path]` operation is not declared for.
pub fn documents_with(apis: &ApiDescriptions, base: OpenApi, options: &DocumentOptions) -> Vec<ApiDocument> {
    apis.versions()
        .iter()
        .map(|version| document(apis, &base, version, options))
        .collect()
}

/// Determines whether an API version is deprecated by every API that declares it.
fn is_deprecated<'a>(apis: impl Iterator<Item = &'a ApiDescription> + Clone, version: &ApiVersion) -> bool {
    let models = || apis.clone().map(|api| api.metadata().as_parts().0);

    !models().any(|model| model.supported().contains(version))
        && models().any(|model| model.deprecated().contains(version))
}

/// Selects the endpoints that a request for the specified API version is dispatched to.
///
/// # Remarks
///
/// Endpoints with the same path and method are selected the same way requests are dispatched: an endpoint that
/// explicitly maps to the API version is preferred over the first endpoint that is version-neutral or that derives the
/// API version from its API.
fn select<'a>(apis: &'a ApiDescriptions, version: &ApiVersion) -> Vec<&'a ApiDescription> {
    let mut selected: Vec<&ApiDescription> = Vec::new();
    let mut seen: Vec<(&str, &Method)> = Vec::new();

    for api in apis {
        let key = (api.path(), api.method());

        if seen.contains(&key) {
            continue;
        }

        seen.push(key);

        let group = || {
            apis.iter()
                .filter(|other| other.path() == api.path() && other.method() == api.method())
        };
        let explicit = group().find(|endpoint| {
            !endpoint.metadata().is_neutral() && endpoint.metadata().mapping_to(version) == Mapping::Explicit
        });
        let fallback = || {
            group().find(|endpoint| {
                endpoint.metadata().is_neutral() || endpoint.metadata().mapping_to(version) == Mapping::Implicit
            })
        };

        if let Some(endpoint) = explicit.or_else(fallback) {
            selected.push(endpoint);
        }
    }

    selected
}

fn http_method(method: &Method) -> Option<HttpMethod> {
    Some(match *method {
        Method::GET => HttpMethod::Get,
        Method::POST => HttpMethod::Post,
        Method::PUT => HttpMethod::Put,
        Method::DELETE => HttpMethod::Delete,
        Method::OPTIONS => HttpMethod::Options,
        Method::HEAD => HttpMethod::Head,
        Method::PATCH => HttpMethod::Patch,
        Method::TRACE => HttpMethod::Trace,
        _ => return None,
    })
}

fn method_name(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Options => "OPTIONS",
        HttpMethod::Head => "HEAD",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Trace => "TRACE",
    }
}

/// Converts an Axum route template into an OpenAPI path template, such as `/files/{*path}` into `/files/{path}`.
fn openapi_path(path: &str) -> String {
    path.replace("{*", "{")
}

fn document(
    apis: &ApiDescriptions,
    base: &OpenApi,
    version: &ApiVersion<'static>,
    options: &DocumentOptions,
) -> ApiDocument {
    let mut openapi = base.clone();
    let deprecated = is_deprecated(apis.iter(), version);
    let api_options = apis.options();
    let parameters = VersionParameters {
        version,
        neutral: false,
        assumed: (api_options.assume_default_version_when_unspecified && *version == api_options.default_version)
            .then_some(&api_options.default_version),
        descriptor: &*api_options.reader as &dyn Descriptor,
        options,
    };
    let mut names: Vec<&str> = Vec::new();

    for api in select(apis, version) {
        let Some(documented) = api.extensions().get::<Operation>() else {
            continue;
        };
        let Some(method) = http_method(api.method()) else {
            continue;
        };

        if !documented.methods.contains(&method) {
            let declared: Vec<_> = documented.methods.iter().map(method_name).collect();

            panic!(
                "{} {} is described by an OpenAPI operation for {}, but the methods must match.",
                api.method(),
                api.path(),
                declared.join(", ")
            );
        }

        let metadata = api.metadata();
        let neutral = metadata.is_neutral();
        let (mut operation, schemas) = documented.describe(version);

        if !api.name().is_empty() {
            operation.tags = Some(vec![api.name().to_owned()]);

            if !names.contains(&api.name()) {
                names.push(api.name());
            }
        }

        if !neutral && metadata.map_all().deprecated().contains(version) {
            operation.deprecated = Some(Deprecated::True);
        }

        let parameters = VersionParameters { neutral, ..parameters };
        let path = parameters.apply(openapi_path(api.path()), &mut operation);

        openapi.paths.add_path_operation(path, vec![method], operation);

        // the schemas of the API version replace the schemas of the base document, which have every member
        if !schemas.is_empty() {
            openapi
                .components
                .get_or_insert_with(Components::new)
                .schemas
                .extend(schemas);
        }
    }

    add_tags(&mut openapi, apis, version, deprecated, &names, options);
    describe(&mut openapi, apis, version, deprecated, options);

    ApiDocument::new(version.clone(), deprecated, openapi)
}

/// Adds a tag for each API name that is not already defined by the document and describes the API versioning policies
/// that are specific to the API, if any.
///
/// # Remarks
///
/// The policies of the API version apply to every API, so they are described by the document. A policy defined for an
/// API by name, with or without the API version, is described by the tag of the API instead. An API that deprecates the
/// API version is described as deprecated by its tag when the API version is not deprecated by every API.
fn add_tags(
    openapi: &mut OpenApi,
    apis: &ApiDescriptions,
    version: &ApiVersion<'static>,
    deprecated: bool,
    names: &[&str],
    options: &DocumentOptions,
) {
    if names.is_empty() {
        return;
    }

    let tags = openapi.tags.get_or_insert_with(Vec::new);

    for name in names {
        let index = match tags.iter().position(|tag| tag.name == *name) {
            Some(index) => index,
            None => {
                tags.push(Tag::new(*name));
                tags.len() - 1
            }
        };
        let deprecation = named(apis.deprecation_policies(), name, version);
        let sunset = named(apis.sunset_policies(), name, version);
        let api_deprecated = !deprecated && is_deprecated(apis.iter().filter(|api| api.name() == *name), version);

        if deprecation.is_none() && sunset.is_none() && !api_deprecated {
            continue;
        }

        let tag = &mut tags[index];

        tag.description =
            markdown::description(tag.description.as_deref(), api_deprecated, deprecation, sunset, options);

        if let Some(value) = markdown::extension(deprecation, sunset) {
            tag.extensions
                .get_or_insert_with(Extensions::default)
                .insert(EXTENSION.into(), value);
        }
    }
}

/// Gets the policy defined for an API by name, preferring the policy for the specified API version.
fn named<'a, T: Policy>(policies: &'a Manager<T>, name: &str, version: &ApiVersion) -> Option<&'a T> {
    policies.get(name, version).or_else(|| policies.for_api(name))
}

/// Describes the API version in the information and extensions of the document.
fn describe(
    openapi: &mut OpenApi,
    apis: &ApiDescriptions,
    version: &ApiVersion<'static>,
    deprecated: bool,
    options: &DocumentOptions,
) {
    let deprecation = apis.deprecation_policies().for_api_version(version);
    let sunset = apis.sunset_policies().for_api_version(version);
    let info = &mut openapi.info;

    if !info.title.is_empty() {
        info.title = format!("{} | v{version:#}", info.title);
    }

    info.version = version.to_string();
    info.description = markdown::description(info.description.as_deref(), deprecated, deprecation, sunset, options);

    if let Some(value) = markdown::extension(deprecation, sunset) {
        openapi
            .extensions
            .get_or_insert_with(Extensions::default)
            .insert(EXTENSION.into(), value);
    }
}
