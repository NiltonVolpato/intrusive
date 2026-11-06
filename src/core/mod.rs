//! The node layer: nodes, node handles, the builder trait, and the storage
//! traits that map membership markers to per-node state.

mod builder;
mod multi_storage;
mod node;
mod storage;

pub use builder::NodeBuilder;
pub use node::{Node, NodeRef};
pub use storage::{HasStorage, Storage, StorageAccess};
