use crate::{ApiVersion, Variadic};
use std::mem;

fn union(mut versions: Vec<ApiVersion<'static>>) -> Vec<ApiVersion<'static>> {
    let len = versions.len();

    if len < 2 {
        return versions;
    }

    let mut distinct = Vec::new();

    versions.sort_by(ApiVersion::total_cmp);
    distinct.push(versions.remove(0));

    for next in versions {
        if *distinct.last().unwrap() != next {
            distinct.push(next);
        }
    }

    distinct.shrink_to_fit();
    distinct
}

/// Represents an API version model builder.
#[derive(Clone)]
pub struct Builder {
    model: Model,
    advertised: Vec<ApiVersion<'static>>,
    deprecated_advertised: Vec<ApiVersion<'static>>,
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Self {
            model: Model::empty(),
            advertised: Vec::new(),
            deprecated_advertised: Vec::new(),
        }
    }
}

impl Builder {
    /// Initializes a new model [Builder].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Indicates the API is version-neutral.
    ///
    /// # Remarks
    ///
    /// This function clears any other previously specified versioning information. Attempting to add additional version
    /// information after this function has been called will have no effect.
    pub fn neutral(mut self) -> Self {
        self.model = Model::neutral();
        self.advertised.clear();
        self.deprecated_advertised.clear();
        self
    }

    /// Adds declared API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - the [API versions](ApiVersion) to add
    ///
    /// # Remarks
    ///
    /// The declared API versions are constrained to the versions declared explicitly by an implementation.
    pub fn add_declared(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        if !self.model.is_neutral {
            self.model.declared.extend(versions.into().to_vec());
        }

        self
    }

    /// Adds supported API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - the [API versions](ApiVersion) to add
    pub fn add_supported(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        if !self.model.is_neutral {
            self.model.supported.extend(versions.into().to_vec());
        }

        self
    }

    /// Adds deprecated API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - the [API versions](ApiVersion) to add
    ///
    /// # Remarks
    ///
    /// A deprecated API version does not mean it is not supported. A deprecated API version is typically advertised six
    /// months or more before it becomes unsupported.
    pub fn add_deprecated(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        if !self.model.is_neutral {
            self.model.deprecated.extend(versions.into().to_vec());
        }

        self
    }

    /// Adds advertised API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - the [API versions](ApiVersion) to add
    pub fn add_advertised(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        if !self.model.is_neutral {
            self.advertised.extend(versions.into().to_vec());
        }

        self
    }

    /// Adds deprecated, advertised API versions.
    ///
    /// # Arguments
    ///
    /// * `versions` - the [API versions](ApiVersion) to add
    ///
    /// # Remarks
    ///
    /// A deprecated API version does not mean it is not supported. A deprecated API version is typically advertised six
    /// months or more before it becomes unsupported.
    pub fn add_deprecated_advertised(mut self, versions: impl Into<Variadic<ApiVersion<'static>>>) -> Self {
        if !self.model.is_neutral {
            self.deprecated_advertised.extend(versions.into().to_vec());
        }

        self
    }

    /// Builds and returns the corresponding API version [model](Model).
    pub fn build(self) -> Model {
        if self.model.is_neutral {
            self.model
        } else {
            // advertised versions are only reported; they are never declared or implemented
            let implemented = union([self.model.supported.clone(), self.model.deprecated.clone()].concat());
            let declared = union(self.model.declared);
            let supported = union([self.model.supported, self.advertised].concat());
            let deprecated = union([self.model.deprecated, self.deprecated_advertised].concat());

            Model {
                is_neutral: false,
                declared,
                supported,
                deprecated,
                implemented,
            }
        }
    }
}

/// Represents the version information for an API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    is_neutral: bool,
    declared: Vec<ApiVersion<'static>>,
    implemented: Vec<ApiVersion<'static>>,
    supported: Vec<ApiVersion<'static>>,
    deprecated: Vec<ApiVersion<'static>>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            is_neutral: false,
            declared: vec![ApiVersion::DEFAULT],
            implemented: vec![ApiVersion::DEFAULT],
            supported: vec![ApiVersion::DEFAULT],
            deprecated: Vec::new(),
        }
    }
}

impl Model {
    /// Creates and returns a new, empty [model](Model).
    pub fn empty() -> Self {
        Self {
            is_neutral: false,
            declared: Vec::new(),
            implemented: Vec::new(),
            supported: Vec::new(),
            deprecated: Vec::new(),
        }
    }

    /// Creates and returns a new, version-neutral [model](Model).
    pub fn neutral() -> Self {
        Self {
            is_neutral: true,
            declared: Vec::new(),
            implemented: Vec::new(),
            supported: Vec::new(),
            deprecated: Vec::new(),
        }
    }

    /// Creates and returns a new [model builder](Builder).
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Gets a value indicating whether the API is version-neutral.
    #[inline]
    pub fn is_neutral(&self) -> bool {
        self.is_neutral
    }

    /// Gets the API versions declared by an implementation.
    ///
    /// # Remarks
    ///
    /// The declared API versions are constrained to the versions declared explicitly by an implementation.
    #[inline]
    pub fn declared(&self) -> &[ApiVersion<'_>] {
        &self.declared
    }

    /// Gets the versions implemented by the API.
    ///
    /// # Remarks
    ///
    /// The implemented API versions include the supported and deprecated API versions.
    #[inline]
    pub fn implemented(&self) -> &[ApiVersion<'_>] {
        &self.implemented
    }

    /// Gets the versions supported by the API.
    #[inline]
    pub fn supported(&self) -> &[ApiVersion<'_>] {
        &self.supported
    }

    /// Gets the versions deprecated by the API.
    ///
    /// # Remarks
    ///
    /// A deprecated API version does not mean it is not supported. A deprecated API version is typically advertised six
    /// months or more before it becomes unsupported; in which case, the API would no longer indicate that it is an
    /// [implemented version](Self::implemented).
    #[inline]
    pub fn deprecated(&self) -> &[ApiVersion<'_>] {
        &self.deprecated
    }

    /// Merges the specified model into the current model.
    ///
    /// # Arguments
    ///
    /// * `other` - the other [model](Model) to merge
    pub fn merge(&mut self, other: Self) {
        if self.implemented.is_empty() || other.implemented.is_empty() {
            return;
        }

        self.implemented = union([mem::take(&mut self.implemented), other.implemented].concat());
        self.supported = union([mem::take(&mut self.supported), other.supported].concat());
        self.deprecated = union([mem::take(&mut self.deprecated), other.deprecated].concat());

        for i in (0..self.deprecated.len()).rev() {
            if self.supported.contains(&self.deprecated[i]) {
                self.deprecated.remove(i);
            }
        }
    }

    /// Merges the specified models into the current model.
    ///
    /// # Arguments
    ///
    /// * `others` - the other [models](Model) to merge
    pub fn merge_all(&mut self, others: impl IntoIterator<Item = Self>) {
        if self.implemented.is_empty() {
            return;
        }

        for other in others {
            self.implemented.extend(other.implemented);
            self.supported.extend(other.supported);
            self.deprecated.extend(other.deprecated);
        }

        self.implemented = union(mem::take(&mut self.implemented));
        self.supported = union(mem::take(&mut self.supported));
        self.deprecated = union(mem::take(&mut self.deprecated));

        for i in (0..self.deprecated.len()).rev() {
            if self.supported.contains(&self.deprecated[i]) {
                self.deprecated.remove(i);
            }
        }
    }

    /// Combines the current model and specified model into a new model.
    ///
    /// # Arguments
    ///
    /// * `other` - the other [model](Model) to combine
    pub fn combine(self, other: Self) -> Self {
        let mut combined = self.clone();
        combined.merge(other);
        combined
    }

    /// Combines the current model and specified models into a new model.
    ///
    /// # Arguments
    ///
    /// * `others` - the other [models](Model) to combine
    pub fn combine_all(self, others: impl IntoIterator<Item = Self>) -> Self {
        let mut combined = self.clone();
        combined.merge_all(others);
        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version;

    #[test]
    fn combine_should_merge_api_version_models() {
        // arrange
        let model1 = Model::builder()
            .add_supported(api_version!(1.0))
            .add_deprecated(api_version!(0.9))
            .build();
        let model2 = Model::builder().add_supported(api_version!(2.0)).build();
        let expected = Model::builder()
            .add_supported(api_version!(1.0))
            .add_supported(api_version!(2.0))
            .add_deprecated(api_version!(0.9))
            .build();

        // act
        let actual = model1.combine(model2);

        // assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn combine_all_should_merge_api_version_models() {
        // arrange
        let model = Model::builder()
            .add_supported(api_version!(1.0))
            .add_deprecated(api_version!(0.9))
            .build();
        let others = [
            Model::builder().add_supported(api_version!(2.0)).build(),
            Model::builder()
                .add_supported(api_version!(3.0))
                .add_deprecated(api_version!(3.0 - alpha))
                .build(),
        ];
        let expected = Model::builder()
            .add_supported([api_version!(1.0), api_version!(2.0), api_version!(3.0)])
            .add_deprecated([api_version!(0.9), api_version!(3.0 - alpha)])
            .build();

        // act
        let actual = model.combine_all(others);

        // assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn combine_should_not_merge_deprecated_api_version_when_also_supported() {
        // arrange
        let model1 = Model::builder().add_supported([api_version!(1.0)]).build();
        let model2 = Model::builder()
            .add_supported(api_version!(2.0))
            .add_deprecated(api_version!(1.0))
            .build();
        let expected = Model::builder()
            .add_supported([api_version!(1.0), api_version!(2.0)])
            .build();

        // act
        let actual = model1.combine(model2);

        // assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn default_model_should_declare_and_support_default_version() {
        // arrange

        // act
        let model = Model::default();

        // assert
        assert!(!model.is_neutral());
        assert_eq!(model.declared(), [ApiVersion::DEFAULT]);
        assert_eq!(model.supported(), [ApiVersion::DEFAULT]);
        assert_eq!(model.implemented(), [ApiVersion::DEFAULT]);
        assert!(model.deprecated().is_empty());
    }

    #[test]
    fn empty_model_should_not_have_versions() {
        // arrange

        // act
        let model = Model::empty();

        // assert
        assert!(!model.is_neutral());
        assert!(model.declared().is_empty());
        assert!(model.supported().is_empty());
        assert!(model.implemented().is_empty());
        assert!(model.deprecated().is_empty());
        assert_eq!(Builder::default().build(), model);
    }

    #[test]
    fn neutral_builder_should_ignore_added_versions() {
        // arrange
        let builder = Model::builder().neutral();

        // act
        let model = builder
            .add_declared(api_version!(1.0))
            .add_supported(api_version!(1.0))
            .add_deprecated(api_version!(0.9))
            .add_advertised(api_version!(2.0))
            .add_deprecated_advertised(api_version!(0.8))
            .build();

        // assert
        assert_eq!(model, Model::neutral());
    }

    #[test]
    fn neutral_builder_should_clear_previously_added_versions() {
        // arrange
        let builder = Model::builder().add_supported(api_version!(1.0));

        // act
        let model = builder.neutral().build();

        // assert
        assert_eq!(model, Model::neutral());
    }

    #[test]
    fn build_should_sort_and_remove_duplicate_versions() {
        // arrange
        let builder = Model::builder()
            .add_declared([api_version!(2.0), api_version!(1.0), api_version!(2.0)])
            .add_supported([api_version!(3.0), api_version!(1.0), api_version!(3.0)])
            .add_deprecated([api_version!(0.9), api_version!(0.8), api_version!(0.9)]);

        // act
        let model = builder.build();

        // assert
        assert_eq!(model.declared(), [api_version!(1.0), api_version!(2.0)]);
        assert_eq!(model.supported(), [api_version!(1.0), api_version!(3.0)]);
        assert_eq!(model.deprecated(), [api_version!(0.8), api_version!(0.9)]);
    }

    #[test]
    fn build_should_implement_supported_and_deprecated_versions() {
        // arrange
        let builder = Model::builder()
            .add_declared(api_version!(4.0))
            .add_supported([api_version!(2.0), api_version!(1.0)])
            .add_deprecated([api_version!(1.0), api_version!(0.9)]);

        // act
        let model = builder.build();

        // assert
        assert_eq!(
            model.implemented(),
            [api_version!(0.9), api_version!(1.0), api_version!(2.0)]
        );
    }

    #[test]
    fn add_advertised_should_add_supported_versions() {
        // arrange
        let builder = Model::builder();

        // act
        let model = builder.add_advertised(api_version!(1.0)).build();

        // assert
        assert_eq!(model.supported(), [api_version!(1.0)]);
        assert!(model.implemented().is_empty());
        assert!(model.deprecated().is_empty());
    }

    #[test]
    fn add_deprecated_advertised_should_add_deprecated_versions() {
        // arrange
        let builder = Model::builder();

        // act
        let model = builder.add_deprecated_advertised(api_version!(1.0)).build();

        // assert
        assert_eq!(model.deprecated(), [api_version!(1.0)]);
        assert!(model.implemented().is_empty());
        assert!(model.supported().is_empty());
    }

    #[test]
    fn build_should_not_implement_advertised_versions() {
        // arrange
        let builder = Model::builder()
            .add_declared([api_version!(0.9), api_version!(1.0)])
            .add_supported(api_version!(1.0))
            .add_deprecated(api_version!(0.9))
            .add_advertised(api_version!(2.0))
            .add_deprecated_advertised(api_version!(0.8));

        // act
        let model = builder.build();

        // assert
        assert_eq!(model.declared(), [api_version!(0.9), api_version!(1.0)]);
        assert_eq!(model.implemented(), [api_version!(0.9), api_version!(1.0)]);
        assert_eq!(model.supported(), [api_version!(1.0), api_version!(2.0)]);
        assert_eq!(model.deprecated(), [api_version!(0.8), api_version!(0.9)]);
    }

    #[test]
    fn merge_should_not_change_model_when_other_is_empty() {
        // arrange
        let mut model = Model::builder().add_supported(api_version!(1.0)).build();
        let expected = model.clone();

        // act
        model.merge(Model::empty());

        // assert
        assert_eq!(model, expected);
    }

    #[test]
    fn merge_should_not_change_model_when_empty() {
        // arrange
        let mut model = Model::empty();
        let other = Model::builder().add_supported(api_version!(1.0)).build();

        // act
        model.merge(other);

        // assert
        assert_eq!(model, Model::empty());
    }

    #[test]
    fn merge_all_should_not_change_model_when_empty() {
        // arrange
        let mut model = Model::empty();
        let others = [Model::builder().add_supported(api_version!(1.0)).build()];

        // act
        model.merge_all(others);

        // assert
        assert_eq!(model, Model::empty());
    }

    #[test]
    fn merge_all_should_not_merge_deprecated_api_version_when_also_supported() {
        // arrange
        let mut model = Model::builder().add_supported(api_version!(1.0)).build();
        let others = [
            Model::builder().add_deprecated(api_version!(1.0)).build(),
            Model::builder().add_deprecated(api_version!(0.9)).build(),
        ];
        let expected = Model::builder()
            .add_supported(api_version!(1.0))
            .add_deprecated(api_version!(0.9))
            .build();

        // act
        model.merge_all(others);

        // assert
        assert_eq!(model, expected);
    }
}
