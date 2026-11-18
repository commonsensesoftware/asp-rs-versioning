use bitflags::bitflags;

use crate::ApiVersion;

bitflags! {
    /// Represents the possible API version provider options.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct ProviderOptions : u32 {
        /// Indicates no options.
        const NONE = 0;

        /// Indicates the provided API versions are deprecated.
        const DEPRECATED = 1;

        /// Indicates the provided API versions are only advertised.
        ///
        /// # Remarks
        ///
        /// Advertised service API versions indicate the existence of other versioned services, but the implementation of
        /// those services are implemented elsewhere.
        const ADVERTISED = 2;

        /// Indicates the provided API versions are only mapped.
        ///
        /// # Remarks
        ///
        /// Mapped API versions indicate that the defined API versions are used for only meant to be used for mapping
        /// purposes. This option should not typically be combined with other options.
        const MAPPED = 4;
    }
}

/// Defines the behavior of an [API version](ApiVersion) provider.
pub trait Provider {
    /// Gets the options associated with the provided API versions.
    fn options(&self) -> ProviderOptions;

    /// Gets the defined API versions defined.
    fn versions(&self) -> Vec<ApiVersion<'static>>;
}
