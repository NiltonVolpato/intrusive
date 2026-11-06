use core::fmt::Debug;

use crate::core::node::{Node, NodeRef};

/// Allocates nodes and resolves handles back to them.
///
/// Builders own the nodes; containers only hold [`NodeRef`] handles and take
/// `&mut B` (or `&B`) on every operation to resolve them.
pub trait NodeBuilder: Sized + Clone + Debug {
    /// The user-facing payload of each node.
    type Value;
    /// Per-node state for all container memberships (see [`StorageAccess`](crate::core::StorageAccess)).
    type Storage: crate::core::Storage;
    /// Identity of a node inside this builder.
    type Handle: Copy + Eq;

    fn build(&self, node: Node<Self>) -> NodeRef<Self>;
    fn resolve(&self, node_ref: NodeRef<Self>) -> &Node<Self>;
    fn resolve_mut(&mut self, node_ref: NodeRef<Self>) -> &mut Node<Self>;
    fn resolve_many_mut<const N: usize>(
        &mut self,
        node_refs: [NodeRef<Self>; N],
    ) -> [&mut Node<Self>; N];
}
