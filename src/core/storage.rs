use crate::core::builder::NodeBuilder;

/// Per-node state for container memberships, with a linked/unlinked notion.
pub trait Storage: Default {
    fn is_linked(&self) -> bool;
}

/// Access to the membership state `S` inside a node's storage.
///
/// `I` is the position of `S`: 0 when the storage is `S` itself, the tuple
/// index inside a composite storage (see `multi_storage`). It is always
/// inferred from `S`, so users never name it; it only exists so that the
/// per-position impls are disjoint for coherence.
pub trait HasStorage<S, const I: usize> {
    fn storage(&self) -> &S;
    fn storage_mut(&mut self) -> &mut S;
}

// Disjoint from the tuple impls: overlap would need `S0 == (S0, S1)`.
impl<S> HasStorage<S, 0> for S
where
    S: Storage,
{
    #[inline(always)]
    fn storage(&self) -> &S {
        self
    }

    #[inline(always)]
    fn storage_mut(&mut self) -> &mut S {
        self
    }
}

/// Names a kind of intrusive membership (a ZST marker, e.g.
/// `LinkedListStorage<Tag>`) and maps it to the concrete per-node state that
/// membership contributes inside `Node::storage`.
///
/// The marker never mentions the builder, so generic builders can be
/// parameterized by it (the `B`-dependent state is only reachable through
/// this projection, never written out by the user). Extracting a marker's
/// state back out of a (possibly composite) node storage is `HasStorage`'s
/// job.
pub trait StorageAccess {
    type State<B: NodeBuilder>: Storage;
}
