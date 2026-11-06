use crate::core::builder::NodeBuilder;

/// Handle to a node allocated in a builder. Copyable and comparable by
/// handle identity; resolve through a builder to reach the node itself.
pub struct NodeRef<B: NodeBuilder> {
    pub(crate) handle: B::Handle,
}

// Manual impls: the derives would incorrectly require `B: Copy`/`B: PartialEq`
// instead of relying on the `Handle: Copy + Eq` bound from `NodeBuilder`.
impl<B: NodeBuilder> Clone for NodeRef<B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B: NodeBuilder> Copy for NodeRef<B> {}

impl<B: NodeBuilder> PartialEq for NodeRef<B> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<B: NodeBuilder> Eq for NodeRef<B> {}

impl<B: NodeBuilder> std::fmt::Debug for NodeRef<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRef").finish_non_exhaustive()
    }
}

impl<B: NodeBuilder> NodeRef<B> {
    pub fn new(handle: B::Handle) -> Self {
        Self { handle }
    }

    #[inline(always)]
    pub fn resolve(self, builder: &B) -> &Node<B> {
        builder.resolve(self)
    }

    #[inline(always)]
    pub(crate) fn resolve_mut(self, builder: &mut B) -> &mut Node<B> {
        builder.resolve_mut(self)
    }
}

/// A node allocated in a builder: the user's value plus the storage for all
/// container memberships the node participates in.
#[derive(Debug)]
pub struct Node<B: NodeBuilder> {
    value: B::Value,
    storage: B::Storage,
}

impl<B: NodeBuilder> Node<B> {
    pub fn build(value: B::Value, builder: &B) -> NodeRef<B> {
        builder.build(Node {
            value,
            storage: Default::default(),
        })
    }

    #[inline(always)]
    pub fn value(&self) -> &B::Value {
        &self.value
    }

    #[inline(always)]
    pub fn value_mut(&mut self) -> &mut B::Value {
        &mut self.value
    }

    #[inline(always)]
    pub(crate) fn storage(&self) -> &B::Storage {
        &self.storage
    }

    #[inline(always)]
    pub(crate) fn storage_mut(&mut self) -> &mut B::Storage {
        &mut self.storage
    }
}
