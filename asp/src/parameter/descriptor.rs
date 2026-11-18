use super::Context;

/// Defines the behavior of a parameter descriptor.
pub trait Descriptor {
    /// Describes the parameter using the provided context.
    ///
    /// # Arguments
    ///
    /// * `context` - the current [context](Context) the parameter is being described in
    fn describe<'a>(&'a self, context: &mut dyn Context<'a>);
}
