use crate::ApiDescription;
use crate::dispatch::{Dispatch, VersionedEndpoint};
use crate::endpoints::{Endpoint, Endpoints, Scope, specific};
use crate::router::join;
use crate::versioning::ApiVersioning;
use api_versioning::{ApiVersion, Metadata, Model, Variadic};
use axum::routing::{MethodFilter, MethodRouter};
use http::Method;
use std::sync::Arc;

/// Represents a versioned API made up of a group of routes.
///
/// # Remarks
///
/// A versioned API is added to a [Router](crate::Router) with [api](crate::Router::api). API version information is
/// collected while the API is defined and is only resolved when the router is built. The order in which API versions
/// and routes are defined does not matter.
pub struct VersionedApi<S = ()> {
    name: String,
    neutral: bool,
    report: bool,
    supported: Vec<ApiVersion<'static>>,
    deprecated: Vec<ApiVersion<'static>>,
    advertised: Vec<ApiVersion<'static>>,
    deprecated_advertised: Vec<ApiVersion<'static>>,
    routes: Vec<(String, Vec<Endpoint<S>>)>,
}

impl<S> Default for VersionedApi<S> {
    fn default() -> Self {
        Self {
            name: String::new(),
            neutral: false,
            report: false,
            supported: Vec::new(),
            deprecated: Vec::new(),
            advertised: Vec::new(),
            deprecated_advertised: Vec::new(),
            routes: Vec::new(),
        }
    }
}

impl<S: Clone + Send + Sync + 'static> VersionedApi<S> {
    /// Sets the logical name of the API.
    ///
    /// # Arguments
    ///
    /// * `name` - the logical name of the API
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Indicates the API has the specified, supported API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - one or more supported [API versions](ApiVersion)
    ///
    /// # Panics
    ///
    /// Panics if any of the API versions are [neutral](ApiVersion::NEUTRAL). Use [neutral](Self::neutral) instead.
    #[track_caller]
    pub fn has(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        self.supported.extend(specific(versions));
        self
    }

    /// Indicates the API has the specified, deprecated API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - one or more deprecated [API versions](ApiVersion)
    ///
    /// # Panics
    ///
    /// Panics if any of the API versions are [neutral](ApiVersion::NEUTRAL). Use [neutral](Self::neutral) instead.
    #[track_caller]
    pub fn deprecates(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        self.deprecated.extend(specific(versions));
        self
    }

    /// Indicates the API advertises the specified, supported API versions implemented elsewhere.
    ///
    /// # Arguments
    ///
    /// * `versions` - one or more advertised [API versions](ApiVersion)
    ///
    /// # Panics
    ///
    /// Panics if any of the API versions are [neutral](ApiVersion::NEUTRAL). Use [neutral](Self::neutral) instead.
    #[track_caller]
    pub fn advertises(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        self.advertised.extend(specific(versions));
        self
    }

    /// Indicates the API advertises the specified, deprecated API versions implemented elsewhere.
    ///
    /// # Arguments
    ///
    /// * `versions` - one or more advertised, deprecated [API versions](ApiVersion)
    ///
    /// # Panics
    ///
    /// Panics if any of the API versions are [neutral](ApiVersion::NEUTRAL). Use [neutral](Self::neutral) instead.
    #[track_caller]
    pub fn advertises_deprecated(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        self.deprecated_advertised.extend(specific(versions));
        self
    }

    /// Indicates the entire API is version-neutral.
    pub fn neutral(mut self) -> Self {
        self.neutral = true;
        self
    }

    /// Indicates the API reports its API versions, even if reporting is not enabled globally.
    pub fn report_api_versions(mut self) -> Self {
        self.report = true;
        self
    }

    /// Adds a route to the API.
    ///
    /// # Arguments
    ///
    /// * `path` - the path of the route, relative to where the API is nested
    /// * `endpoints` - the function used to define the [endpoints](Endpoints) of the route
    pub fn route(mut self, path: &str, endpoints: impl FnOnce(Endpoints<S>) -> Endpoints<S>) -> Self {
        let items = endpoints(Endpoints::default()).items;

        match self.routes.iter_mut().find(|(p, _)| p == path) {
            Some((_, existing)) => existing.extend(items),
            None => self.routes.push((path.into(), items)),
        }

        self
    }

    fn display_name(&self) -> String {
        if self.name.is_empty() {
            "the API".into()
        } else {
            format!("the {} API", self.name)
        }
    }

    // declares each inferred API version as supported unless it is explicitly declared by the API
    fn declare_inferred(&mut self) {
        let inferred = self
            .routes
            .iter()
            .flat_map(|(_, endpoints)| endpoints)
            .filter(|endpoint| endpoint.inferred)
            .filter_map(|endpoint| match &endpoint.scope {
                Scope::Mapped(versions) => Some(versions),
                _ => None,
            })
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        for version in inferred {
            let explicit = [
                &self.supported,
                &self.deprecated,
                &self.advertised,
                &self.deprecated_advertised,
            ];

            if !explicit.iter().any(|versions| versions.contains(&version)) {
                self.supported.push(version);
            }
        }
    }

    fn api_model(&self, default_version: &ApiVersion<'static>) -> Model {
        if self.neutral {
            return Model::neutral();
        }

        let unversioned = self.supported.is_empty()
            && self.deprecated.is_empty()
            && self.advertised.is_empty()
            && self.deprecated_advertised.is_empty();

        if unversioned {
            let version = default_version.clone();
            return Model::builder()
                .add_declared(version.clone())
                .add_supported(version)
                .build();
        }

        Model::builder()
            .add_declared(self.supported.iter().chain(&self.deprecated).cloned())
            .add_supported(self.supported.clone())
            .add_deprecated(self.deprecated.clone())
            .add_advertised(self.advertised.clone())
            .add_deprecated_advertised(self.deprecated_advertised.clone())
            .build()
    }

    fn endpoint_model(&self, api: &Model, path: &str, method: &Method, scope: &Scope) -> Model {
        let inherited = || {
            let mut builder = Model::builder();

            for version in api.supported() {
                builder = if api.implemented().contains(version) {
                    builder.add_supported(version.to_owned())
                } else {
                    builder.add_advertised(version.to_owned())
                };
            }

            for version in api.deprecated() {
                builder = if api.implemented().contains(version) {
                    builder.add_deprecated(version.to_owned())
                } else {
                    builder.add_deprecated_advertised(version.to_owned())
                };
            }

            builder
        };

        match scope {
            Scope::Neutral => Model::neutral(),
            _ if api.is_neutral() => Model::neutral(),
            Scope::Derived => inherited().build(),
            Scope::Mapped(versions) => {
                if let Some(version) = versions.iter().find(|version| !api.declared().contains(version)) {
                    panic!(
                        "{method} {path} maps to API version {version}, which is not declared by {}.",
                        self.display_name()
                    );
                }

                inherited().add_declared(versions.clone()).build()
            }
        }
    }
}

fn overlaps(left: &Scope, right: &Scope) -> bool {
    match (left, right) {
        (Scope::Neutral, _) | (_, Scope::Neutral) | (Scope::Derived, Scope::Derived) => true,
        (Scope::Mapped(left), Scope::Mapped(right)) => left.iter().any(|version| right.contains(version)),
        _ => false,
    }
}

impl<S: Clone + Send + Sync + 'static> VersionedApi<S> {
    /// Registers the routes of the API with the specified router.
    ///
    /// # Arguments
    ///
    /// * `router` - the router the routes are added to
    /// * `versioning` - the shared API versioning configuration
    /// * `path` - the path of the API, relative to `router`
    /// * `prefix` - the path `router` is nested at, relative to the root router
    /// * `descriptions` - the descriptions each endpoint of the API is added to
    pub(crate) fn register(
        mut self,
        mut router: axum::Router<S>,
        versioning: &ApiVersioning,
        path: &str,
        prefix: &str,
        descriptions: &mut Vec<ApiDescription>,
    ) -> axum::Router<S> {
        self.declare_inferred();

        let api_model = self.api_model(&versioning.options().default_version);
        let api_metadata = Arc::new(Metadata::new(self.name.clone(), api_model.clone(), api_model.clone()));

        versioning.declare(api_model.declared());

        for (route, endpoints) in std::mem::take(&mut self.routes) {
            let route = join(path, &route);
            let full = join(prefix, &route);
            let mut methods: Vec<(Method, Vec<VersionedEndpoint<S>>)> = Vec::new();
            let mut scopes: Vec<(Method, Scope)> = Vec::new();

            for Endpoint {
                method,
                scope,
                handler,
                extensions,
                ..
            } in endpoints
            {
                if scopes.iter().any(|(m, s)| *m == method && overlaps(s, &scope)) {
                    panic!(
                        "{method} {full} has more than one handler for the same API versions in {}.",
                        self.display_name()
                    );
                }

                let model = self.endpoint_model(&api_model, &full, &method, &scope);
                let metadata = Arc::new(Metadata::new(self.name.clone(), api_model.clone(), model));

                descriptions.push(ApiDescription::new(
                    method.clone(),
                    full.clone(),
                    metadata.clone(),
                    extensions,
                ));
                scopes.push((method.clone(), scope));

                let endpoint = VersionedEndpoint { metadata, handler };

                match methods.iter_mut().find(|(m, _)| *m == method) {
                    Some((_, items)) => items.push(endpoint),
                    None => methods.push((method, vec![endpoint])),
                }
            }

            let mut method_router = MethodRouter::new();

            for (method, endpoints) in methods {
                let filter = MethodFilter::try_from(method.clone())
                    .unwrap_or_else(|_| panic!("The {method} method is not supported by axum."));
                let dispatch = Dispatch::new(versioning.clone(), endpoints, api_metadata.clone(), self.report);

                method_router = method_router.on(filter, dispatch);
            }

            router = router.route(&route, method_router);
        }

        router
    }
}
