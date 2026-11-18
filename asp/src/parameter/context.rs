use super::Location;

/// Defines the behavior of a context for API version parameter description.
pub trait Context<'a> {
    /// Registers a parameter with the given name and location.
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the parameter
    /// * `location` - the [location](Location) where the parameter is expected
    fn register(&mut self, name: &'a str, location: Location);
}
