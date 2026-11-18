use crate::error::{ProblemDetailsResponder, Responder};
use crate::versioning::ApiVersioning;
use crate::{ApiDescription, ApiDescriptions, VersionedApi};
use api_versioning::Options;
use axum::{
    extract::Request,
    handler::Handler,
    response::IntoResponse,
    routing::{MethodRouter, Route},
};
use std::convert::Infallible;
use tower_layer::Layer;
use tower_service::Service;

type Apply<S> = Box<dyn FnOnce(axum::Router<S>) -> axum::Router<S> + Send>;

/// Joins a path to the path it is relative to.
///
/// # Remarks
///
/// The paths are joined the same way Axum joins a nested path, so `/` relative to `/orders` is `/orders`.
pub(crate) fn join(prefix: &str, path: &str) -> String {
    if prefix.is_empty() || prefix == "/" {
        path.into()
    } else if path.is_empty() || path == "/" {
        prefix.into()
    } else {
        format!("{}{path}", prefix.trim_end_matches('/'))
    }
}

/// Represents the API versioning configuration of the root router.
struct Config {
    options: Options,
    responder: Box<dyn Responder>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            options: Options::default(),
            responder: Box::new(ProblemDetailsResponder),
        }
    }
}

/// Represents an item added to a router, which is applied in order when the router is built.
enum Item<S> {
    Api(String, VersionedApi<S>),
    Nest(String, Router<S>),
    Merge(Router<S>),
    Apply(Apply<S>),
}

/// Represents a router of versioned APIs.
///
/// # Remarks
///
/// A router wraps an Axum [Router](axum::Router) so that the complete path and API version information of every
/// endpoint is known when the router is built. Routes, APIs, nested routers, and layers are recorded in the order they
/// are added and the Axum router is only built when the router is [split](Self::split) or converted. The semantics of
/// each operation are the same as the Axum operation of the same name; for example, a [layer](Self::layer) only applies
/// to the routes that were added before it.
///
/// The root router owns the API versioning configuration, which is shared by all of the APIs in the router, including
/// the APIs in nested and merged routers. A router created with [new](Self::new) is configured and can only be the root.
/// A router created with [default](Self::default) is unconfigured; it uses the configuration of the router it is nested
/// in or merged with, or the default configuration if it is the root.
///
/// # Examples
///
/// ```
/// use api_versioning::{Options, api_version};
/// use api_versioning_axum::Router;
///
/// async fn get_orders() -> &'static str {
///     "orders"
/// }
///
/// async fn reports() -> &'static str {
///     "reports"
/// }
///
/// fn admin() -> Router {
///     Router::default().api("/reports", |api| api.name("Reports").has(api_version!(1.0)).route("/", |e| e.get(reports)))
/// }
///
/// let (app, apis): (axum::Router, _) = Router::new(Options::default())
///     .api("/orders", |api| api.name("Orders").has(api_version!(1.0)).route("/", |e| e.get(get_orders)))
///     .nest("/admin", admin())
///     .split();
///
/// let paths: Vec<_> = apis.iter().map(|api| api.path()).collect();
///
/// assert_eq!(paths, ["/orders", "/admin/reports"]);
/// ```
pub struct Router<S = ()> {
    config: Option<Config>,
    items: Vec<Item<S>>,
}

impl<S> Default for Router<S> {
    fn default() -> Self {
        Self {
            config: None,
            items: Vec::new(),
        }
    }
}

impl<S: Clone + Send + Sync + 'static> Router<S> {
    /// Initializes a new, configured root [Router].
    ///
    /// # Arguments
    ///
    /// * `options` - the API versioning [options](Options) shared by all of the APIs in the router
    ///
    /// # Panics
    ///
    /// Panics if the [options are invalid](Options::validate).
    #[track_caller]
    pub fn new(options: Options) -> Self {
        options.validate();

        Self {
            config: Some(Config {
                options,
                ..Default::default()
            }),
            items: Vec::new(),
        }
    }

    /// Configures the [responder](Responder) that creates the response for an API versioning error.
    ///
    /// # Arguments
    ///
    /// * `responder` - the [responder](Responder) to use
    ///
    /// # Remarks
    ///
    /// The default responder is the [ProblemDetailsResponder]. Configuring the responder of an unconfigured router also
    /// configures it with the default [options](Options), so it can only be the root.
    pub fn with_error_responder(mut self, responder: impl Responder + 'static) -> Self {
        self.config.get_or_insert_default().responder = Box::new(responder);
        self
    }

    /// Adds a versioned API.
    ///
    /// # Arguments
    ///
    /// * `path` - the path of the API, which each of its routes is relative to
    /// * `api` - the function used to define the [API](VersionedApi)
    pub fn api(mut self, path: &str, api: impl FnOnce(VersionedApi<S>) -> VersionedApi<S>) -> Self {
        self.items.push(Item::Api(path.into(), api(VersionedApi::default())));
        self
    }

    /// Nests a router at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - the path the router is nested at
    /// * `router` - the router to nest, which can also be an Axum [Router](axum::Router)
    ///
    /// # Panics
    ///
    /// Panics if the nested router is configured. Only the root router can be configured.
    #[track_caller]
    pub fn nest(mut self, path: &str, router: impl Into<Self>) -> Self {
        let router = unconfigured(router.into());
        self.items.push(Item::Nest(path.into(), router));
        self
    }

    /// Merges the routes of another router into this router.
    ///
    /// # Arguments
    ///
    /// * `router` - the router to merge, which can also be an Axum [Router](axum::Router)
    ///
    /// # Panics
    ///
    /// Panics if the merged router is configured. Only the root router can be configured.
    #[track_caller]
    pub fn merge(mut self, router: impl Into<Self>) -> Self {
        let router = unconfigured(router.into());
        self.items.push(Item::Merge(router));
        self
    }

    /// Adds an unversioned route. See [Router::route](axum::Router::route).
    ///
    /// # Arguments
    ///
    /// * `path` - the path of the route
    /// * `method_router` - the [MethodRouter] for the route
    pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
        let path = path.to_owned();
        self.apply(move |router| router.route(&path, method_router))
    }

    /// Adds an unversioned route to a service. See [Router::route_service](axum::Router::route_service).
    ///
    /// # Arguments
    ///
    /// * `path` - the path of the route
    /// * `service` - the service to route to
    pub fn route_service<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        let path = path.to_owned();
        self.apply(move |router| router.route_service(&path, service))
    }

    /// Nests a service at the specified path. See [Router::nest_service](axum::Router::nest_service).
    ///
    /// # Arguments
    ///
    /// * `path` - the path the service is nested at
    /// * `service` - the service to nest
    pub fn nest_service<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        let path = path.to_owned();
        self.apply(move |router| router.nest_service(&path, service))
    }

    /// Adds a fallback handler. See [Router::fallback](axum::Router::fallback).
    ///
    /// # Arguments
    ///
    /// * `handler` - the fallback handler
    pub fn fallback<H, T>(self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.apply(move |router| router.fallback(handler))
    }

    /// Applies a layer to the routes added so far. See [Router::layer](axum::Router::layer).
    ///
    /// # Arguments
    ///
    /// * `layer` - the layer to apply
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.apply(move |router| router.layer(layer))
    }

    /// Applies a layer to the routes added so far, but only if a route matches. See
    /// [Router::route_layer](axum::Router::route_layer).
    ///
    /// # Arguments
    ///
    /// * `layer` - the layer to apply
    pub fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.apply(move |router| router.route_layer(layer))
    }

    /// Builds the router and splits it into an Axum [Router](axum::Router) and the [descriptions](ApiDescriptions) of
    /// its versioned APIs.
    ///
    /// # Panics
    ///
    /// Panics if any API is invalid, such as an endpoint that maps to an API version its API does not declare, or if
    /// Axum rejects any route.
    pub fn split(mut self) -> (axum::Router<S>, ApiDescriptions) {
        let Config { options, responder } = self.config.take().unwrap_or_default();
        let versioning = ApiVersioning::new(options, responder);
        let mut descriptions = Vec::new();
        let router = self.build(&versioning, "", &mut descriptions);

        (router, ApiDescriptions::new(versioning, descriptions))
    }

    fn apply(mut self, apply: impl FnOnce(axum::Router<S>) -> axum::Router<S> + Send + 'static) -> Self {
        self.items.push(Item::Apply(Box::new(apply)));
        self
    }

    fn build(
        self,
        versioning: &ApiVersioning,
        prefix: &str,
        descriptions: &mut Vec<ApiDescription>,
    ) -> axum::Router<S> {
        let mut router = axum::Router::new();

        for item in self.items {
            router = match item {
                Item::Api(path, api) => api.register(router, versioning, &path, prefix, descriptions),
                Item::Nest(path, nested) => {
                    let nested = nested.build(versioning, &join(prefix, &path), descriptions);
                    router.nest(&path, nested)
                }
                Item::Merge(merged) => router.merge(merged.build(versioning, prefix, descriptions)),
                Item::Apply(apply) => apply(router),
            };
        }

        router
    }
}

#[track_caller]
fn unconfigured<S>(router: Router<S>) -> Router<S> {
    assert!(
        router.config.is_none(),
        "Only the root router can be configured. Use Router::default() to create a router that is nested or merged."
    );
    router
}

impl<S: Clone + Send + Sync + 'static> From<axum::Router<S>> for Router<S> {
    fn from(router: axum::Router<S>) -> Self {
        Self::default().apply(move |this| this.merge(router))
    }
}

impl<S: Clone + Send + Sync + 'static> From<Router<S>> for axum::Router<S> {
    fn from(router: Router<S>) -> Self {
        router.split().0
    }
}

impl<S: Clone + Send + Sync + 'static> From<Options> for Router<S> {
    #[track_caller]
    fn from(options: Options) -> Self {
        Self::new(options)
    }
}
