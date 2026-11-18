use api_versioning::{ApiVersion, Variadic};
use axum::{extract::Request, handler::Handler, response::Response};
use http::{Extensions, Method};
use smallvec::SmallVec;
use std::{future::Future, pin::Pin, sync::Arc};

/// Ensures the specified API versions are specific API versions that can be declared by an API or endpoint.
#[track_caller]
pub(crate) fn specific(versions: impl Into<Variadic<ApiVersion<'static>>>) -> SmallVec<[ApiVersion<'static>; 1]> {
    let versions = versions.into().to_vec();

    assert!(
        !versions.iter().any(ApiVersion::is_neutral),
        "ApiVersion::NEUTRAL cannot be declared as an API version; use neutral() to make an API or endpoint \
         version-neutral."
    );

    versions
}

pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = Response> + Send>>;
pub(crate) type BoxHandler<S> = Arc<dyn Fn(Request, S) -> BoxFuture + Send + Sync>;

/// Represents how an endpoint maps to the API versions of its API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    /// Indicates the endpoint derives its API versions from the API.
    Derived,

    /// Indicates the endpoint explicitly maps to specific API versions.
    Mapped(SmallVec<[ApiVersion<'static>; 1]>),

    /// Indicates the endpoint is version-neutral.
    Neutral,
}

pub(crate) struct Endpoint<S> {
    pub method: Method,
    pub scope: Scope,

    /// Indicates whether the API versions of a mapped scope were inferred from the handler.
    pub inferred: bool,
    pub handler: BoxHandler<S>,

    /// The additional metadata attached to the handler.
    pub extensions: Extensions,
}

fn endpoint<S, H, T>(method: Method, scope: Scope, inferred: bool, handler: H) -> Endpoint<S>
where
    H: IntoEndpoint<T, S>,
{
    let (handler, extensions) = handler.into_parts();

    Endpoint {
        method,
        scope,
        inferred,
        handler,
        extensions,
    }
}

macro_rules! methods {
    () => {
        methods!(get, GET);
        methods!(post, POST);
        methods!(put, PUT);
        methods!(patch, PATCH);
        methods!(delete, DELETE);
    };
    ($name:ident, $method:ident) => {
        #[doc = concat!("Adds a handler for `", stringify!($method), "` requests.")]
        ///
        /// # Arguments
        ///
        /// * `handler` - the handler to add
        #[track_caller]
        pub fn $name<H, T>(self, handler: H) -> Self
        where
            H: IntoEndpoint<T, S>,
        {
            self.on(Method::$method, handler)
        }
    };
}

/// Represents the endpoints of a single route in a versioned API.
///
/// # Remarks
///
/// Endpoints added directly derive their API versions from the API. Endpoints added within [map](Self::map) explicitly
/// map to specific API versions, endpoints added within [infer](Self::infer) map to the API version inferred from their
/// handlers, and endpoints added within [neutral](Self::neutral) are version-neutral.
pub struct Endpoints<S = ()> {
    pub(crate) items: Vec<Endpoint<S>>,
}

impl<S> Default for Endpoints<S> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<S: Clone + Send + Sync + 'static> Endpoints<S> {
    /// Adds handlers that explicitly map to the specified API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - one or more [API versions](ApiVersion) declared by the API
    /// * `handlers` - the function used to add the mapped [handlers](Handlers)
    ///
    /// # Remarks
    ///
    /// An endpoint that explicitly maps to an API version only matches that version. An endpoint that derives its API
    /// versions from the API never matches a version that is explicitly mapped by another endpoint.
    ///
    /// # Panics
    ///
    /// Panics if any of the API versions are [neutral](ApiVersion::NEUTRAL). Use [neutral](Self::neutral) instead.
    #[track_caller]
    pub fn map(
        self,
        versions: impl Into<Variadic<ApiVersion<'static>>>,
        handlers: impl FnOnce(Handlers<S>) -> Handlers<S>,
    ) -> Self {
        self.scoped(Scope::Mapped(specific(versions)), handlers)
    }

    /// Adds handlers that are version-neutral.
    ///
    /// # Arguments
    ///
    /// * `handlers` - the function used to add the version-neutral [handlers](Handlers)
    ///
    /// # Remarks
    ///
    /// A version-neutral endpoint matches a request without an API version or with any declared API version.
    pub fn neutral(self, handlers: impl FnOnce(Handlers<S>) -> Handlers<S>) -> Self {
        self.scoped(Scope::Neutral, handlers)
    }

    /// Adds handlers that map to the API version inferred from each handler's module path and function name.
    ///
    /// # Arguments
    ///
    /// * `handlers` - the function used to add the inferred [handlers](Handlers)
    ///
    /// # Remarks
    ///
    /// Each handler explicitly maps to the API version inferred by [ApiVersion::infer_from_val], as if it was added
    /// within [map](Self::map). For example, `v1::orders::get` and `hello_v1` both map to API version 1. The inferred
    /// API version follows the handler when its module or function is renamed.
    ///
    /// An inferred API version is also declared as a supported API version of the API unless the API explicitly
    /// declares it, such as with [has](crate::VersionedApi::has) or
    /// [deprecates](crate::VersionedApi::deprecates). To deprecate an inferred API version, declare it with
    /// `deprecates(api_version!(use path))`.
    ///
    /// # Panics
    ///
    /// Panics when a handler is added if exactly one API version cannot be inferred from it or the inferred API version
    /// is [neutral](ApiVersion::NEUTRAL). A closure or a handler wrapped by another type, such as a layered handler,
    /// generally has no function name to infer from. Use [map](Self::map) instead.
    pub fn infer(self, handlers: impl FnOnce(Handlers<S>) -> Handlers<S>) -> Self {
        let handlers = handlers(Handlers {
            mapping: Mapping::Inferred,
            items: Vec::new(),
        });

        self.append(handlers)
    }

    /// Adds a handler for the specified method that derives its API versions from the API.
    ///
    /// # Arguments
    ///
    /// * `method` - the HTTP [method](Method) the handler applies to
    /// * `handler` - the handler to add
    pub fn on<H, T>(mut self, method: Method, handler: H) -> Self
    where
        H: IntoEndpoint<T, S>,
    {
        self.items.push(endpoint(method, Scope::Derived, false, handler));
        self
    }

    methods!();

    fn scoped(self, scope: Scope, handlers: impl FnOnce(Handlers<S>) -> Handlers<S>) -> Self {
        let handlers = handlers(Handlers {
            mapping: Mapping::Scoped(scope),
            items: Vec::new(),
        });

        self.append(handlers)
    }

    fn append(mut self, handlers: Handlers<S>) -> Self {
        self.items.extend(handlers.items);
        self
    }
}

/// Represents how a set of handlers maps to API versions.
enum Mapping {
    /// Indicates every handler has the same scope.
    Scoped(Scope),

    /// Indicates each handler maps to the API version inferred from the handler.
    Inferred,
}

/// Represents a set of handlers that share the same API version mapping.
pub struct Handlers<S = ()> {
    mapping: Mapping,
    items: Vec<Endpoint<S>>,
}

impl<S: Clone + Send + Sync + 'static> Handlers<S> {
    /// Adds a handler for the specified method.
    ///
    /// # Arguments
    ///
    /// * `method` - the HTTP [method](Method) the handler applies to
    /// * `handler` - the handler to add
    ///
    /// # Panics
    ///
    /// Panics if the handlers are [inferred](Endpoints::infer) and exactly one API version cannot be inferred from the
    /// handler.
    #[track_caller]
    pub fn on<H, T>(mut self, method: Method, handler: H) -> Self
    where
        H: IntoEndpoint<T, S>,
    {
        let (scope, inferred) = match &self.mapping {
            Mapping::Scoped(scope) => (scope.clone(), false),
            Mapping::Inferred => (Scope::Mapped(handler.infer()), true),
        };

        self.items.push(endpoint(method, scope, inferred, handler));
        self
    }

    methods!();
}

mod sealed {
    pub trait Sealed<T, S> {}
}

/// Represents the marker that distinguishes a [Described] handler from any other handler.
#[doc(hidden)]
pub struct DescribedMarker;

/// Defines the behavior of a type that can be added as the handler of an endpoint.
///
/// # Remarks
///
/// This trait is implemented for every Axum [Handler] and for every [Described] handler. It cannot be implemented
/// outside of this crate.
pub trait IntoEndpoint<T, S>: sealed::Sealed<T, S> {
    /// Infers the API version of the handler from its module path and function name.
    #[doc(hidden)]
    #[track_caller]
    fn infer(&self) -> SmallVec<[ApiVersion<'static>; 1]>;

    /// Converts the handler into a boxed handler and its attached metadata.
    #[doc(hidden)]
    fn into_parts(self) -> (BoxHandler<S>, Extensions);
}

fn boxed<H, T, S>(handler: H) -> BoxHandler<S>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    Arc::new(move |request, state| Box::pin(handler.clone().call(request, state)))
}

impl<H, T, S> sealed::Sealed<T, S> for H
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
}

impl<H, T, S> IntoEndpoint<T, S> for H
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    #[track_caller]
    fn infer(&self) -> SmallVec<[ApiVersion<'static>; 1]> {
        specific(ApiVersion::infer_from_val(self))
    }

    fn into_parts(self) -> (BoxHandler<S>, Extensions) {
        (boxed(self), Extensions::new())
    }
}

/// Represents a handler with additional metadata attached to it.
///
/// # Remarks
///
/// The metadata is attached to the endpoint the handler is added to and is available from its
/// [description](crate::ApiDescription::extensions). It is typically used by components that describe APIs, such as
/// OpenAPI document generators, which provide their own, more convenient way to describe a handler.
///
/// # Examples
///
/// ```
/// use api_versioning::{Options, api_version};
/// use api_versioning_axum::{Described, Router};
///
/// #[derive(Clone)]
/// struct Summary(&'static str);
///
/// async fn get_orders() -> &'static str {
///     "orders"
/// }
///
/// let (_, apis): (axum::Router, _) = Router::new(Options::default())
///     .api("/orders", |api| {
///         api.has(api_version!(1.0))
///             .route("/", |e| e.get(Described::new(get_orders).with(Summary("Gets the orders"))))
///     })
///     .split();
///
/// let summary = apis.iter().next().unwrap().extensions().get::<Summary>().unwrap();
///
/// assert_eq!(summary.0, "Gets the orders");
/// ```
pub struct Described<H> {
    handler: H,
    extensions: Extensions,
}

impl<H> Described<H> {
    /// Initializes a new [Described] handler without any metadata.
    ///
    /// # Arguments
    ///
    /// * `handler` - the handler to describe
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            extensions: Extensions::new(),
        }
    }

    /// Attaches the specified metadata to the handler, replacing any metadata of the same type.
    ///
    /// # Arguments
    ///
    /// * `value` - the metadata to attach
    pub fn with<V: Clone + Send + Sync + 'static>(mut self, value: V) -> Self {
        self.extensions.insert(value);
        self
    }

    /// Gets the described handler.
    #[inline]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Gets the metadata attached to the handler.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }
}

impl<H, T, S> sealed::Sealed<(DescribedMarker, T), S> for Described<H>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
}

impl<H, T, S> IntoEndpoint<(DescribedMarker, T), S> for Described<H>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    #[track_caller]
    fn infer(&self) -> SmallVec<[ApiVersion<'static>; 1]> {
        specific(ApiVersion::infer_from_val(&self.handler))
    }

    fn into_parts(self) -> (BoxHandler<S>, Extensions) {
        (boxed(self.handler), self.extensions)
    }
}
