use core::ptr::NonNull;

use bumpalo::Bump;

use crate::{Node, NodeBuilder, NodeRef, StorageAccess};

/// Arena-backed node builder, generic over `L`: the marker (or tuple of
/// markers, see `MultiStorage`) naming the memberships the nodes carry.
pub struct ArenaNodeBuilder<'a, V, L> {
    arena: &'a Bump,
    _marker: std::marker::PhantomData<(V, L)>,
}

impl<V, L> Clone for ArenaNodeBuilder<'_, V, L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, L> Copy for ArenaNodeBuilder<'_, V, L> {}

impl<V, L> PartialEq for ArenaNodeBuilder<'_, V, L> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.arena, other.arena)
    }
}

impl<V, L> Eq for ArenaNodeBuilder<'_, V, L> {}

impl<V, L> core::fmt::Debug for ArenaNodeBuilder<'_, V, L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArenaNodeBuilder")
            .field("arena", &self.arena)
            .finish()
    }
}

impl<'a, V, L> ArenaNodeBuilder<'a, V, L> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, V, L> NodeBuilder for ArenaNodeBuilder<'a, V, L>
where
    L: StorageAccess,
{
    type Value = V;
    type Storage = L::State<Self>;
    type Handle = NonNull<Node<Self>>;

    fn build(&self, node: Node<Self>) -> NodeRef<Self> {
        let node_ref = self.arena.alloc(node);
        NodeRef::new(node_ref.into())
    }

    fn resolve(&self, node_ref: NodeRef<Self>) -> &Node<Self> {
        unsafe { node_ref.handle.as_ref() }
    }

    fn resolve_mut(&mut self, node_ref: NodeRef<Self>) -> &mut Node<Self> {
        unsafe { &mut *node_ref.handle.as_ptr() }
    }

    fn resolve_many_mut<const N: usize>(
        &mut self,
        node_refs: [NodeRef<Self>; N],
    ) -> [&mut Node<Self>; N] {
        // TODO: Maybe return an error instead?
        for i in 0..N {
            for j in (i + 1)..N {
                assert_ne!(
                    node_refs[i], node_refs[j],
                    "Cannot mutably resolve the same node handle more than once"
                )
            }
        }

        node_refs.map(|node_ref| unsafe { &mut *node_ref.handle.as_ptr() })
    }
}
