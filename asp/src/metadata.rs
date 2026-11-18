use crate::{
    ApiVersion,
    Mapping::{self, *},
    Model,
};
use std::borrow::Cow::{self, *};
use std::sync::OnceLock;

/// Represents the API version metadata applied to an endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    name: String,
    api_model: Model,
    endpoint_model: Model,
    merged_model: OnceLock<Model>,
}

impl Metadata {
    /// Initializes new [Metadata].
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the API the metadata belongs to, if any
    /// * `api_model` - the API version [model](Model) for the API as a whole
    /// * `endpoint_model` - the API version [model](Model) for a specific endpoint
    pub fn new(name: impl Into<String>, api_model: Model, endpoint_model: Model) -> Self {
        Self {
            name: name.into(),
            api_model,
            endpoint_model,
            merged_model: OnceLock::new(),
        }
    }

    /// Gets empty API version information.
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            api_model: Model::empty(),
            endpoint_model: Model::empty(),
            merged_model: OnceLock::new(),
        }
    }

    /// Gets version-neutral API version information.
    pub fn neutral() -> Self {
        Self {
            name: String::new(),
            api_model: Model::neutral(),
            endpoint_model: Model::neutral(),
            merged_model: OnceLock::new(),
        }
    }

    /// Gets the API name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets a value indicating whether the API is version-neutral.
    pub fn is_neutral(&self) -> bool {
        self.api_model.is_neutral() || self.endpoint_model.is_neutral()
    }

    /// Returns an API version model for the requested mapping.
    ///
    /// # Remarks
    ///
    /// * `mapping` - the [mapping](Mapping) to return the [model](Model) for
    pub fn map(&self, mapping: Mapping) -> Cow<'_, Model> {
        match mapping {
            Explicit => Borrowed(&self.endpoint_model),
            Implicit => Borrowed(&self.api_model),
            None => Owned(Model::empty()),
        }
    }

    /// Returns an API version model for all mappings.
    pub fn map_all(&self) -> &Model {
        self.merged_model.get_or_init(|| {
            if self.api_model.is_neutral() {
                self.api_model.clone()
            } else if self.endpoint_model.is_neutral() || !self.endpoint_model.declared().is_empty() {
                self.endpoint_model.clone()
            } else if self.endpoint_model.implemented().is_empty() {
                Model::neutral()
            } else {
                Model::builder()
                    .add_declared(self.api_model.declared().iter().map(ApiVersion::to_owned))
                    .add_supported(self.endpoint_model.supported().iter().map(ApiVersion::to_owned))
                    .add_deprecated(self.endpoint_model.deprecated().iter().map(ApiVersion::to_owned))
                    .build()
            }
        })
    }

    /// Returns a value indicating whether the defined metadata maps to the specified API version.
    ///
    /// # Arguments
    ///
    /// * `version` - the [API version](ApiVersion) to test the [mapping](Mapping) for
    pub fn mapping_to(&self, version: &ApiVersion) -> Mapping {
        if self.is_neutral() {
            return Explicit;
        }

        let mapped =
            self.endpoint_model.declared().contains(version) && self.endpoint_model.implemented().contains(version);

        if mapped {
            return Explicit;
        }

        let derived = self.endpoint_model.declared().is_empty() && self.api_model.declared().contains(version);

        if derived { Implicit } else { None }
    }

    /// Returns a value indicating whether the defined metadata maps to the specified API version.
    pub fn is_mapped_to(&self, version: &ApiVersion) -> bool {
        self.mapping_to(version) != None
    }

    /// Decomposes the metadata into references of its constituent parts.
    ///
    /// # Remarks
    ///
    /// The first part is the API model, the second part is the endpoint model, and the last part is the API name.
    pub fn as_parts(&self) -> (&Model, &Model, &str) {
        (&self.api_model, &self.endpoint_model, &self.name)
    }

    /// Consumes and decomposes the metadata into its constituent parts.
    ///
    /// # Remarks
    ///
    /// The first part is the API model, the second part is the endpoint model, and the last part is the API name.
    pub fn into_parts(self) -> (Model, Model, String) {
        (self.api_model, self.endpoint_model, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version;

    fn metadata(api_model: Model, endpoint_model: Model) -> Metadata {
        Metadata {
            name: "test".into(),
            api_model,
            endpoint_model,
            merged_model: OnceLock::new(),
        }
    }

    #[test]
    fn empty_should_not_map_to_any_version() {
        // arrange
        let metadata = Metadata::empty();

        // act
        let mapping = metadata.mapping_to(&ApiVersion::DEFAULT);

        // assert
        assert_eq!(metadata.name(), "");
        assert!(!metadata.is_neutral());
        assert_eq!(mapping, None);
        assert!(!metadata.is_mapped_to(&ApiVersion::DEFAULT));
    }

    #[test]
    fn neutral_should_map_explicitly_to_any_version() {
        // arrange
        let metadata = Metadata::neutral();

        // act
        let mapping = metadata.mapping_to(&api_version!(42.0));

        // assert
        assert!(metadata.is_neutral());
        assert_eq!(mapping, Explicit);
        assert!(metadata.is_mapped_to(&api_version!(42.0)));
    }

    #[test]
    fn is_neutral_should_be_true_when_either_model_is_neutral() {
        // arrange
        let api = metadata(Model::neutral(), Model::empty());
        let endpoint = metadata(Model::empty(), Model::neutral());

        // act
        let results = (api.is_neutral(), endpoint.is_neutral());

        // assert
        assert_eq!(results, (true, true));
    }

    #[test]
    fn mapping_to_should_be_explicit_when_endpoint_declares_and_implements_version() {
        // arrange
        let endpoint = Model::builder()
            .add_declared([api_version!(1.0)])
            .add_supported([api_version!(1.0)])
            .build();
        let metadata = metadata(Model::empty(), endpoint);

        // act
        let mapping = metadata.mapping_to(&api_version!(1.0));

        // assert
        assert_eq!(mapping, Explicit);
    }

    #[test]
    fn mapping_to_should_be_none_when_endpoint_declares_but_does_not_implement_version() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let endpoint = Model::builder().add_declared([api_version!(1.0)]).build();
        let metadata = metadata(api, endpoint);

        // act
        let mapping = metadata.mapping_to(&api_version!(1.0));

        // assert
        assert_eq!(mapping, None);
    }

    #[test]
    fn mapping_to_should_be_implicit_when_only_api_declares_version() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let metadata = metadata(api, Model::empty());

        // act
        let mapping = metadata.mapping_to(&api_version!(1.0));

        // assert
        assert_eq!(mapping, Implicit);
        assert!(metadata.is_mapped_to(&api_version!(1.0)));
    }

    #[test]
    fn mapping_to_should_not_be_implicit_when_endpoint_declares_other_versions() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let endpoint = Model::builder()
            .add_declared([api_version!(2.0)])
            .add_supported([api_version!(2.0)])
            .build();
        let metadata = metadata(api, endpoint);

        // act
        let mapping = metadata.mapping_to(&api_version!(1.0));

        // assert
        assert_eq!(mapping, None);
    }

    #[test]
    fn map_should_return_model_for_mapping() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let endpoint = Model::builder().add_declared([api_version!(2.0)]).build();
        let metadata = metadata(api.clone(), endpoint.clone());

        // act
        let explicit = metadata.map(Explicit);
        let implicit = metadata.map(Implicit);
        let none = metadata.map(None);

        // assert
        assert_eq!(*explicit, endpoint);
        assert_eq!(*implicit, api);
        assert_eq!(*none, Model::empty());
    }

    #[test]
    fn map_all_should_return_api_model_when_api_is_neutral() {
        // arrange
        let endpoint = Model::builder().add_declared([api_version!(1.0)]).build();
        let metadata = metadata(Model::neutral(), endpoint);

        // act
        let model = metadata.map_all();

        // assert
        assert_eq!(*model, Model::neutral());
    }

    #[test]
    fn map_all_should_return_endpoint_model_when_endpoint_is_neutral() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let metadata = metadata(api, Model::neutral());

        // act
        let model = metadata.map_all();

        // assert
        assert_eq!(*model, Model::neutral());
    }

    #[test]
    fn map_all_should_return_endpoint_model_when_endpoint_declares_versions() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let endpoint = Model::builder()
            .add_declared([api_version!(2.0)])
            .add_supported([api_version!(2.0)])
            .build();
        let metadata = metadata(api, endpoint.clone());

        // act
        let model = metadata.map_all();

        // assert
        assert_eq!(*model, endpoint);
    }

    #[test]
    fn map_all_should_merge_api_declared_with_endpoint_implemented_versions() {
        // arrange
        let api = Model::builder()
            .add_declared([api_version!(1.0), api_version!(2.0)])
            .add_supported([api_version!(3.0)])
            .build();
        let endpoint = Model::builder()
            .add_supported([api_version!(2.0)])
            .add_deprecated([api_version!(1.0)])
            .build();
        let metadata = metadata(api, endpoint);
        let expected = Model::builder()
            .add_declared([api_version!(1.0), api_version!(2.0)])
            .add_supported([api_version!(2.0)])
            .add_deprecated([api_version!(1.0)])
            .build();

        // act
        let model = metadata.map_all();

        // assert
        assert_eq!(*model, expected);
    }

    #[test]
    fn map_all_should_return_same_model_on_subsequent_calls() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let metadata = metadata(api, Model::empty());
        let first = metadata.map_all() as *const Model;

        // act
        let second = metadata.map_all() as *const Model;

        // assert
        assert_eq!(first, second);
    }

    #[test]
    fn as_parts_should_borrow_constituent_parts() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let endpoint = Model::builder().add_declared([api_version!(2.0)]).build();
        let metadata = metadata(api.clone(), endpoint.clone());

        // act
        let parts = metadata.as_parts();

        // assert
        assert_eq!(parts, (&api, &endpoint, "test"));
    }

    #[test]
    fn into_parts_should_return_constituent_parts() {
        // arrange
        let api = Model::builder().add_declared([api_version!(1.0)]).build();
        let endpoint = Model::builder().add_declared([api_version!(2.0)]).build();
        let metadata = metadata(api.clone(), endpoint.clone());

        // act
        let parts = metadata.into_parts();

        // assert
        assert_eq!(parts, (api, endpoint, String::from("test")));
    }
}
