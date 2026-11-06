//! Crate-level error type for container misuse.

/// Errors returned when a container operation is applied to a node in the
/// wrong link state.
///
/// Containers return this directly; a container with more specific failure
/// modes can define its own error type and convert from this one via
/// `From`, so `?` keeps working.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The node is already linked (in this container's membership).
    AlreadyLinked,
    /// The node is not linked (in this container's membership).
    NotLinked,
    /// A priority update did not decrease the key.
    NotDecreased,
}
