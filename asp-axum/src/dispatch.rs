use crate::endpoints::{BoxFuture, BoxHandler};
use crate::error::Context;
use crate::versioning::ApiVersioning;
use crate::{ApiVersionError, EndpointMetadata, RequestedApiVersion};
use api_versioning::{ApiVersion, Mapping, Metadata, Model, RequestInfo, visibility::Scoped};
use axum::{
    extract::{FromRequestParts, OriginalUri, RawPathParams, Request},
    handler::Handler,
};
use http::{HeaderMap, Uri, request::Parts};
use smallvec::SmallVec;
use std::sync::Arc;

/// Represents the information about a request that was matched by the router.
struct RoutedRequest<'a> {
    parts: &'a Parts,
    parameters: Option<RawPathParams>,
}

impl RequestInfo for RoutedRequest<'_> {
    fn uri(&self) -> &Uri {
        &self.parts.uri
    }

    fn headers(&self) -> &HeaderMap {
        &self.parts.headers
    }

    fn route_parameter(&self, name: &str) -> Option<&str> {
        self.parameters
            .as_ref()?
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value)
    }
}

pub(crate) struct VersionedEndpoint<S> {
    pub metadata: Arc<Metadata>,
    pub handler: BoxHandler<S>,
}

/// Represents the handler that dispatches a request to the endpoint matching the requested API version.
///
/// # Remarks
///
/// Axum only allows a single handler per path and method so the dispatcher is registered in place of all of the
/// versioned handlers that share a path and method.
pub(crate) struct Dispatch<S> {
    versioning: ApiVersioning,
    endpoints: Arc<[VersionedEndpoint<S>]>,
    api: Arc<Metadata>,
    implemented: Arc<Model>,
    report: bool,
}

/// Creates the model of the API versions that can be dispatched to the specified endpoints.
///
/// # Remarks
///
/// An API might implement versions that a specific path and method do not, in which case those versions must not be
/// selected for them.
fn implemented_by<S>(api: &Metadata, endpoints: &[VersionedEndpoint<S>]) -> Model {
    let api = api.map(Mapping::Implicit);
    let mut builder = Model::builder();

    for version in api.implemented() {
        let endpoint = |endpoint: &VersionedEndpoint<S>| {
            !endpoint.metadata.is_neutral() && endpoint.metadata.is_mapped_to(version)
        };

        if !endpoints.iter().any(endpoint) {
            continue;
        }

        builder = if api.deprecated().contains(version) {
            builder.add_deprecated(version.to_owned())
        } else {
            builder.add_supported(version.to_owned())
        };
    }

    builder.build()
}

impl<S> Dispatch<S> {
    pub fn new(
        versioning: ApiVersioning,
        endpoints: Vec<VersionedEndpoint<S>>,
        api: Arc<Metadata>,
        report: bool,
    ) -> Self {
        let implemented = Arc::new(implemented_by(&api, &endpoints));

        Self {
            versioning,
            endpoints: endpoints.into(),
            api,
            implemented,
            report,
        }
    }
}

impl<S> Clone for Dispatch<S> {
    fn clone(&self) -> Self {
        Self {
            versioning: self.versioning.clone(),
            endpoints: self.endpoints.clone(),
            api: self.api.clone(),
            implemented: self.implemented.clone(),
            report: self.report,
        }
    }
}

/// Selects the endpoint matching the requested API version.
///
/// # Remarks
///
/// A match only occurs if exactly one distinct, valid API version is requested and it maps to a registered endpoint. An
/// explicit mapping is always preferred over a derived (implicit) mapping, which means a derived endpoint never matches
/// an API version explicitly mapped by another endpoint. A version-neutral endpoint matches no API version or any API
/// version declared by any aggregated API. If no API version is requested and one is assumed, the configured selector
/// chooses it from the versions implemented by the endpoints. The path of the request is only used to describe an error.
async fn select<'a, S>(
    versioning: &'a ApiVersioning,
    request: &'a (dyn RequestInfo + Sync),
    path: &str,
    endpoints: &[VersionedEndpoint<S>],
    implemented: &'a Model,
) -> Result<(usize, Option<RequestedApiVersion>), ApiVersionError> {
    let options = versioning.options();
    let mut requested = SmallVec::<[(&'a str, ApiVersion<'a>); 1]>::new();

    for text in options.reader.read(request) {
        let version = ApiVersion::parse(text).map_err(|_| ApiVersionError::Invalid {
            path: path.into(),
            version: text.into(),
        })?;

        if !requested.iter().any(|(_, other)| *other == version) {
            requested.push((text, version));
        }
    }

    if requested.len() > 1 {
        requested.sort_by(|(_, left), (_, right)| left.total_cmp(right));

        let versions = requested.iter().map(|(raw, _)| (*raw).to_owned()).collect();
        return Err(ApiVersionError::Ambiguous(versions));
    }

    if requested.is_empty()
        && let Some(index) = endpoints.iter().position(|endpoint| endpoint.metadata.is_neutral())
    {
        return Ok((index, None));
    }

    let (raw, version) = match requested.pop() {
        Some((raw, version)) => (Some(raw), version),
        None if options.assume_default_version_when_unspecified => {
            let selected = versioning.selector().select(request, implemented).await;
            (None, selected.ok_or(ApiVersionError::Unspecified)?)
        }
        None => return Err(ApiVersionError::Unspecified),
    };
    let mut fallback = None;

    for (index, endpoint) in endpoints.iter().enumerate() {
        if endpoint.metadata.is_neutral() {
            if fallback.is_none() && versioning.is_declared(&version) {
                fallback = Some(index);
            }

            continue;
        }

        match endpoint.metadata.mapping_to(&version) {
            Mapping::Explicit => return Ok((index, Some(RequestedApiVersion::new(raw, &version)))),
            Mapping::Implicit if fallback.is_none() => fallback = Some(index),
            _ => {}
        }
    }

    match fallback {
        Some(index) => Ok((index, Some(RequestedApiVersion::new(raw, &version)))),
        None => Err(ApiVersionError::Unsupported {
            path: path.into(),
            version: raw.map_or_else(|| version.to_string(), Into::into),
        }),
    }
}

impl<S: Clone + Send + Sync + 'static> Handler<(), S> for Dispatch<S> {
    type Future = BoxFuture;

    fn call(self, request: Request, state: S) -> Self::Future {
        Box::pin(async move {
            let versioning = &self.versioning;
            let options = versioning.options();
            let report = self.report || options.report_versions.enabled;
            let (mut parts, body) = request.into_parts();
            let parameters = if versioning.versions_by_url() {
                // path parameters with invalid UTF-8 are never API versions; the rejection is left to the handler
                RawPathParams::from_request_parts(&mut parts, &()).await.ok()
            } else {
                None
            };
            let selected = {
                let routed = RoutedRequest {
                    parts: &parts,
                    parameters,
                };

                // a nested router strips its prefix from the request URI, but an error describes the entire path
                let path = parts
                    .extensions
                    .get::<OriginalUri>()
                    .map_or(&parts.uri, |uri| &uri.0)
                    .path();

                select(versioning, &routed, path, &self.endpoints, &self.implemented).await
            };
            let (index, requested) = match selected {
                Ok(selected) => selected,
                Err(error) => {
                    let context = Context::new(&error, error.status(versioning), Some(&parts));
                    let mut response = versioning.error_responder().respond(&context);

                    if report {
                        versioning.reporter().report(response.headers_mut(), None, &self.api);
                    }

                    return response;
                }
            };
            let mut request = Request::from_parts(parts, body);

            let endpoint = &self.endpoints[index];
            let current = requested.as_ref().map(|requested| requested.version().clone());
            let reported = current.as_ref().filter(|_| report).cloned();

            if let Some(requested) = requested {
                request.extensions_mut().insert(requested);
            }

            request
                .extensions_mut()
                .insert(EndpointMetadata(endpoint.metadata.clone()));

            // the version is current while the handler runs so that serde can filter members by visibility
            let mut response = Scoped::new(current, (endpoint.handler)(request, state)).await;

            if report {
                versioning
                    .reporter()
                    .report(response.headers_mut(), reported.as_ref(), &endpoint.metadata);
            }

            response
        })
    }
}
