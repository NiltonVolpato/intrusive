// Concrete doubly-linked storage (Zero Cell, plain Option)
use crate::{NodeBuilder, NodeRef, Storage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    AlreadyLinked,
    NotLinked,
}

/// Doubly-linked-list storage. `Tag` disambiguates when the same node type
/// participates in multiple lists (e.g. an LRU list and a free list).
///
/// While linked, the list is circular: `tail.next == head` and
/// `head.prev == tail`, so every linked node has both links set.
pub struct LinkedListStorage<B: NodeBuilder, Tag = ()> {
    pub next: Option<NodeRef<B>>,
    pub prev: Option<NodeRef<B>>,
    _marker: std::marker::PhantomData<Tag>,
}

impl<B: NodeBuilder, Tag> PartialEq for LinkedListStorage<B, Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.prev == other.prev
    }
}

impl<B: NodeBuilder, Tag> Eq for LinkedListStorage<B, Tag> {}

impl<B: NodeBuilder, Tag> Default for LinkedListStorage<B, Tag> {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B: NodeBuilder, Tag> Storage for LinkedListStorage<B, Tag> {
    fn is_linked(&self) -> bool {
        self.next.is_some() || self.prev.is_some()
    }
}

/// Link access for doubly-linked-list membership, implemented by the node's
/// storage (bare `LinkedListStorage` or a composite storage holding one per
/// `Tag`).
pub trait LinkedListLinks<B: NodeBuilder, Tag = ()>: Storage {
    fn links(&self) -> &LinkedListStorage<B, Tag>;
    fn links_mut(&mut self) -> &mut LinkedListStorage<B, Tag>;
}

impl<B: NodeBuilder, Tag> LinkedListLinks<B, Tag> for LinkedListStorage<B, Tag> {
    fn links(&self) -> &Self {
        self
    }

    fn links_mut(&mut self) -> &mut Self {
        self
    }
}

pub struct LinkedList<B: NodeBuilder, Tag = ()>
where
    B::Storage: LinkedListLinks<B, Tag>,
{
    head: Option<NodeRef<B>>,
    tail: Option<NodeRef<B>>,
    len: usize,
    _marker: std::marker::PhantomData<Tag>,
}

impl<B, Tag> Default for LinkedList<B, Tag>
where
    B: NodeBuilder,
    B::Storage: LinkedListLinks<B, Tag>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B, Tag> LinkedList<B, Tag>
where
    B: NodeBuilder,
    B::Storage: LinkedListLinks<B, Tag>,
{
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn front(&self) -> Option<NodeRef<B>> {
        self.head
    }

    pub fn push_back(&mut self, builder: &mut B, node: NodeRef<B>) -> Result<(), Error> {
        if builder.resolve(node).storage().links().is_linked() {
            return Err(Error::AlreadyLinked);
        }

        match self.tail {
            None => {
                let links = builder.resolve_mut(node).storage_mut().links_mut();
                links.next = Some(node);
                links.prev = Some(node);
            }
            Some(tail) => {
                let head = self.head.unwrap();
                if head != tail {
                    // len >= 2: head, tail and the unlinked node are distinct.
                    let [tail_node, head_node, new_node] =
                        builder.resolve_many_mut([tail, head, node]);
                    tail_node.storage_mut().links_mut().next = Some(node);
                    head_node.storage_mut().links_mut().prev = Some(node);
                    let new_links = new_node.storage_mut().links_mut();
                    new_links.next = Some(head);
                    new_links.prev = Some(tail);
                } else {
                    // len == 1: the circle grows from the single node.
                    let [old_node, new_node] = builder.resolve_many_mut([tail, node]);
                    let old_links = old_node.storage_mut().links_mut();
                    old_links.next = Some(node);
                    old_links.prev = Some(node);
                    let new_links = new_node.storage_mut().links_mut();
                    new_links.next = Some(tail);
                    new_links.prev = Some(tail);
                }
            }
        }

        self.head.get_or_insert(node);
        self.tail = Some(node);
        self.len += 1;
        Ok(())
    }

    pub fn pop_front(&mut self, builder: &mut B) -> Option<NodeRef<B>> {
        let head = self.head?;

        if self.len == 1 {
            let links = builder.resolve_mut(head).storage_mut().links_mut();
            links.next = None;
            links.prev = None;
            self.head = None;
            self.tail = None;
            self.len = 0;
            return Some(head);
        }

        let next = builder.resolve(head).storage().links().next.unwrap();

        if self.len == 2 {
            let [head_node, next_node] = builder.resolve_many_mut([head, next]);
            let head_links = head_node.storage_mut().links_mut();
            head_links.next = None;
            head_links.prev = None;
            let next_links = next_node.storage_mut().links_mut();
            next_links.next = Some(next);
            next_links.prev = Some(next);
        } else {
            let tail = self.tail.unwrap();
            let [head_node, next_node, tail_node] =
                builder.resolve_many_mut([head, next, tail]);
            let head_links = head_node.storage_mut().links_mut();
            head_links.next = None;
            head_links.prev = None;
            next_node.storage_mut().links_mut().prev = Some(tail);
            tail_node.storage_mut().links_mut().next = Some(next);
        }

        self.head = Some(next);
        self.len -= 1;
        Some(head)
    }
}

#[cfg(all(test, feature = "bumpalo"))]
mod tests {
    use core::ptr::NonNull;

    use bumpalo::Bump;

    use super::*;
    use crate::Node;

    /// Concrete arena builder with the storage fixed to `LinkedListStorage`,
    /// so its type can be named (a generic `ArenaNodeBuilder<V, S>` cannot be
    /// instantiated with a storage that mentions the builder itself).
        struct TestBuilder<'a, Tag = ()> {
        arena: &'a Bump,
        _marker: std::marker::PhantomData<Tag>,
    }

    impl<'a, Tag> Clone for TestBuilder<'a, Tag> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<'a, Tag> Copy for TestBuilder<'a, Tag> {}

    impl<'a, Tag> PartialEq for TestBuilder<'a, Tag> {
        fn eq(&self, other: &Self) -> bool {
            core::ptr::eq(self.arena, other.arena)
        }
    }

    impl<'a, Tag> Eq for TestBuilder<'a, Tag> {}

    impl<'a, Tag> core::fmt::Debug for TestBuilder<'a, Tag> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("TestBuilder").finish_non_exhaustive()
        }
    }

    impl<'a, Tag> NodeBuilder for TestBuilder<'a, Tag> {
        type Value = u32;
        type Storage = LinkedListStorage<TestBuilder<'a, Tag>, Tag>;
        type Handle = NonNull<Node<Self>>;

        fn build(&self, node: Node<Self>) -> NodeRef<Self> {
            NodeRef::new(self.arena.alloc(node).into())
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
            for i in 0..N {
                for j in (i + 1)..N {
                    assert_ne!(node_refs[i], node_refs[j]);
                }
            }
            node_refs.map(|node_ref| unsafe { &mut *node_ref.handle.as_ptr() })
        }
    }

    fn build_node<'a, Tag>(builder: &TestBuilder<'a, Tag>, value: u32) -> NodeRef<TestBuilder<'a, Tag>> {
        Node::build(value, builder)
    }

    #[test]
    fn fifo_order() {
        let arena = Bump::new();
        let mut builder: TestBuilder = TestBuilder { arena: &arena, _marker: std::marker::PhantomData };
        let mut list = LinkedList::new();

        let a = build_node(&builder, 1);
        let b = build_node(&builder, 2);
        let c = build_node(&builder, 3);
        list.push_back(&mut builder, a).unwrap();
        list.push_back(&mut builder, b).unwrap();
        list.push_back(&mut builder, c).unwrap();
        assert_eq!(list.len(), 3);

        assert_eq!(list.pop_front(&mut builder).unwrap().resolve(&builder).value(), &1);
        assert_eq!(list.pop_front(&mut builder).unwrap().resolve(&builder).value(), &2);
        assert_eq!(list.pop_front(&mut builder).unwrap().resolve(&builder).value(), &3);
        assert!(list.pop_front(&mut builder).is_none());
        assert!(list.is_empty());
    }

    #[test]
    fn double_push_is_already_linked() {
        let arena = Bump::new();
        let mut builder: TestBuilder = TestBuilder { arena: &arena, _marker: std::marker::PhantomData };
        let mut list = LinkedList::new();

        let a = build_node(&builder, 1);
        list.push_back(&mut builder, a).unwrap();
        assert_eq!(list.push_back(&mut builder, a), Err(Error::AlreadyLinked));
        assert_eq!(list.len(), 1);

        // The failed push did not corrupt the list.
        assert_eq!(list.pop_front(&mut builder).unwrap().resolve(&builder).value(), &1);
    }

    #[test]
    fn popped_node_can_be_pushed_again() {
        let arena = Bump::new();
        let mut builder: TestBuilder = TestBuilder { arena: &arena, _marker: std::marker::PhantomData };
        let mut list = LinkedList::new();

        let a = build_node(&builder, 1);
        list.push_back(&mut builder, a).unwrap();
        let a = list.pop_front(&mut builder).unwrap();
        list.push_back(&mut builder, a).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list.front(), Some(a));
    }

    #[test]
    fn circular_invariants() {
        let arena = Bump::new();
        let mut builder: TestBuilder = TestBuilder { arena: &arena, _marker: std::marker::PhantomData };
        let mut list = LinkedList::new();

        let a = build_node(&builder, 1);
        let b = build_node(&builder, 2);
        let c = build_node(&builder, 3);
        list.push_back(&mut builder, a).unwrap();
        list.push_back(&mut builder, b).unwrap();
        list.push_back(&mut builder, c).unwrap();

        // tail.next == head and head.prev == tail.
        let la_prev = a.resolve(&builder).storage().links().prev;
        let lc_next = c.resolve(&builder).storage().links().next;
        assert_eq!(lc_next, Some(a));
        assert_eq!(la_prev, Some(c));

        // After popping the head, the new head's prev is still the tail.
        list.pop_front(&mut builder);
        let lb_prev = b.resolve(&builder).storage().links().prev;
        let lc_next = c.resolve(&builder).storage().links().next;
        assert_eq!(lb_prev, Some(c));
        assert_eq!(lc_next, Some(b));
    }

    #[test]
    fn tagged_storage() {
        #[derive(Default)]
        struct Lru;

        let arena = Bump::new();
        let mut builder: TestBuilder<Lru> = TestBuilder { arena: &arena, _marker: std::marker::PhantomData };
        let mut list: LinkedList<TestBuilder<Lru>, Lru> = LinkedList::new();

        let a = build_node(&builder, 1);
        list.push_back(&mut builder, a).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(&mut builder).unwrap().resolve(&builder).value(), &1);
    }
}
