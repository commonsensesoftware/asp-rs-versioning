/// Represents the possible types of API version mappings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mapping {
    /// Indicates no mapping.
    None,

    /// Indicates an explicit mapping.
    Explicit,

    /// Indicates an implicit mapping.
    Implicit,
}
