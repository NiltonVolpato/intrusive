//! Intrusive data structures: nodes live in a builder-owned arena and
//! containers hold only [`NodeRef`] handles.
//!
//! The crate is organized by responsibility:
//!
//! - [`core`]: the node layer — [`Node`], [`NodeRef`], the [`NodeBuilder`]
//!   trait, and the storage traits ([`Storage`], [`HasStorage`],
//!   [`StorageAccess`]) that map membership markers to per-node state.
//! - [`builders`]: concrete [`NodeBuilder`] implementations.
//! - [`containers`]: the data structures themselves.
//!
//! The public API is re-exported flat at the crate root, so
//! `intrusive::LinkedList` works without naming submodules.

mod builders;
mod containers;
mod core;

mod errors;

#[cfg(feature = "bumpalo")]
pub use builders::arena::ArenaNodeBuilder;
pub use containers::linked_list::{LinkedList, LinkedListStorage, Links};
pub use core::{HasStorage, Node, NodeBuilder, NodeRef, Storage, StorageAccess};
pub use errors::Error;
