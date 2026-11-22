use crate::DocumentOptions;
use api_versioning::{
    ApiVersion,
    parameter::{Context, Descriptor, Location},
};
use utoipa::openapi::{
    RefOr, Required,
    path::{Operation, Parameter, ParameterBuilder, ParameterIn},
    schema::{ObjectBuilder, Schema, Type},
};

/// Represents the context that collects the distinct API version parameters registered by a descriptor.
#[derive(Default)]
struct Registrations<'a>(Vec<(&'a str, Location)>);

impl<'a> Context<'a> for Registrations<'a> {
    fn register(&mut self, name: &'a str, location: Location) {
        let registered = self
            .0
            .iter()
            .any(|(other, at)| *at == location && other.eq_ignore_ascii_case(name));

        if !registered {
            self.0.push((name, location));
        }
    }
}

/// Represents the information used to add the API version parameters to an operation.
pub(crate) struct VersionParameters<'a> {
    /// The API version the operation is described for.
    pub version: &'a ApiVersion<'static>,

    /// Indicates whether the operation is version-neutral.
    pub neutral: bool,

    /// The default API version, if it is assumed when the API version is unspecified. When it is defined, the API
    /// version parameters are optional and their value defaults to it.
    pub assumed: Option<&'a ApiVersion<'static>>,

    /// The descriptor of the API version parameters.
    pub descriptor: &'a dyn Descriptor,

    /// The options used to describe the parameters.
    pub options: &'a DocumentOptions,
}

impl VersionParameters<'_> {
    /// Adds the API version parameters to the specified operation.
    ///
    /// # Arguments
    ///
    /// * `path` - the OpenAPI path template of the operation
    /// * `operation` - the [operation](Operation) to add the parameters to
    ///
    /// # Returns
    ///
    /// The path template of the operation, which no longer has the API version parameter if the API version was
    /// substituted into it.
    pub fn apply(&self, path: String, operation: &mut Operation) -> String {
        if self.neutral && !self.options.add_api_version_parameters_when_version_neutral {
            return path;
        }

        let mut registrations = Registrations::default();

        self.descriptor.describe(&mut registrations);

        let segment = registrations
            .0
            .iter()
            .filter(|(_, location)| *location == Location::Path)
            .map(|(name, _)| format!("{{{name}}}"))
            .find(|segment| path.contains(segment.as_str()));

        // an API version in the URL identifies the resource, so it replaces any other API version parameter
        if let Some(segment) = segment {
            let name = &segment[1..segment.len() - 1];

            if self.options.substitute_api_version_in_url && !self.neutral {
                remove(operation, name, ParameterIn::Path);

                // the short form omits a minor version of 0, such as 1 rather than 1.0, which is how the API version
                // usually appears in a URL
                return path.replace(&segment, &format!("{:#}", self.version));
            }

            self.upsert(operation, name, ParameterIn::Path, !self.neutral, None);
            return path;
        }

        let mut first = true;

        for (name, location) in registrations.0 {
            let parameter_in = match location {
                Location::Query => ParameterIn::Query,
                Location::Header => ParameterIn::Header,
                Location::MediaType => {
                    self.add_media_type_parameter(operation, name);
                    continue;
                }
                Location::Path => continue,
            };
            let required = first && !self.neutral && self.assumed.is_none();

            self.upsert(operation, name, parameter_in, required, self.assumed);
            first = false;
        }

        path
    }

    fn schema(&self, default: Option<&ApiVersion>) -> RefOr<Schema> {
        let schema = ObjectBuilder::new()
            .schema_type(Type::String)
            .enum_values(Some([self.version.to_string()]))
            .default(default.map(|version| version.to_string().into()))
            .build();

        RefOr::T(Schema::Object(schema))
    }

    /// Adds or updates an API version parameter. The description of an existing parameter is retained.
    fn upsert(
        &self,
        operation: &mut Operation,
        name: &str,
        parameter_in: ParameterIn,
        required: bool,
        default: Option<&ApiVersion>,
    ) {
        let required = if required { Required::True } else { Required::False };
        let description = self.options.api_version_parameter_description;
        let parameters = operation.parameters.get_or_insert_with(Vec::new);
        let existing = parameters.iter_mut().find_map(|parameter| match parameter {
            RefOr::T(parameter) if matches(parameter, name, &parameter_in) => Some(parameter),
            _ => None,
        });

        match existing {
            Some(parameter) => {
                parameter.required = required;
                parameter.schema = Some(self.schema(default));
                parameter.description.get_or_insert_with(|| description.into());
            }
            None => parameters.push(
                ParameterBuilder::new()
                    .name(name)
                    .parameter_in(parameter_in)
                    .description(Some(description))
                    .required(required)
                    .schema(Some(self.schema(default)))
                    .build()
                    .into(),
            ),
        }
    }

    /// Adds the API version as a parameter of each media type of the request and response content.
    fn add_media_type_parameter(&self, operation: &mut Operation, name: &str) {
        let add = |media_type: String| {
            let exists = media_type.split(';').skip(1).any(|parameter| {
                parameter
                    .split('=')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .eq_ignore_ascii_case(name)
            });

            if exists {
                media_type
            } else {
                format!("{media_type}; {name}={}", self.version)
            }
        };

        if let Some(RefOr::T(body)) = &mut operation.request_body {
            body.content = std::mem::take(&mut body.content)
                .into_iter()
                .map(|(media_type, content)| (add(media_type), content))
                .collect();
        }

        for response in operation.responses.responses.values_mut() {
            if let RefOr::T(response) = response {
                response.content = std::mem::take(&mut response.content)
                    .into_iter()
                    .map(|(media_type, content)| (add(media_type), content))
                    .collect();
            }
        }
    }
}

fn matches(parameter: &Parameter, name: &str, parameter_in: &ParameterIn) -> bool {
    let same_name = match parameter_in {
        ParameterIn::Header => parameter.name.eq_ignore_ascii_case(name),
        _ => parameter.name == name,
    };

    same_name && parameter.parameter_in == *parameter_in
}

fn remove(operation: &mut Operation, name: &str, parameter_in: ParameterIn) {
    if let Some(parameters) = &mut operation.parameters {
        parameters
            .retain(|parameter| !matches!(parameter, RefOr::T(parameter) if matches(parameter, name, &parameter_in)));

        if parameters.is_empty() {
            operation.parameters = None;
        }
    }
}
