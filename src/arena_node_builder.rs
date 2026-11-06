use core::ptr::NonNull;

use bumpalo::Bump;

use crate::{Node, NodeBuilder, NodeRef, Storage};

struct ArenaNodeBuilder<'a, V, S> {
    arena: &'a Bump,
    _marker: std::marker::PhantomData<(V, S)>,
}

impl<V, S> Clone for ArenaNodeBuilder<'_, V, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, S> Copy for ArenaNodeBuilder<'_, V, S> {}

impl<'a, V, S> PartialEq for ArenaNodeBuilder<'a, V, S> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.arena, other.arena)
    }
}

impl<'a, V, S> Eq for ArenaNodeBuilder<'a, V, S> {}

impl<'a, V, S> core::fmt::Debug for ArenaNodeBuilder<'a, V, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArenaNodeBuilder")
            .field("arena", &self.arena)
            .finish()
    }
}

impl<'a, V, S> ArenaNodeBuilder<'a, V, S> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, S> NodeBuilder for ArenaNodeBuilder<'_, V, S>
where
    S: Storage,
{
    type Value = V;
    type Storage = S;
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
