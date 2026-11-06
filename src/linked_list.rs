// Concrete doubly-linked storage (Zero Cell, plain Option)
use crate::{HasStorage, NodeBuilder, NodeRef, Storage, StorageAccess};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    AlreadyLinked,
    NotLinked,
}

/// Marker naming doubly-linked-list membership. `Tag` disambiguates when the
/// same node participates in multiple lists (e.g. an LRU list and a free
/// list): different tags give different markers and different `Links<B, Tag>`
/// state inside the node.
///
/// This is only used to parameterize builders (possibly in a tuple of
/// markers, see `MultiStorage`); containers take the `Tag` directly.
pub struct LinkedListStorage<Tag = ()>(std::marker::PhantomData<Tag>);

/// Per-node state contributed by `LinkedListStorage<Tag>`.
///
/// While linked, the list is circular: `tail.next == head` and
/// `head.prev == tail`, so every linked node has both links set.
pub struct Links<B: NodeBuilder, Tag = ()> {
    pub next: Option<NodeRef<B>>,
    pub prev: Option<NodeRef<B>>,
    _marker: std::marker::PhantomData<Tag>,
}

impl<B: NodeBuilder, Tag> PartialEq for Links<B, Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.prev == other.prev
    }
}

impl<B: NodeBuilder, Tag> Eq for Links<B, Tag> {}

impl<B: NodeBuilder, Tag> Default for Links<B, Tag> {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<B: NodeBuilder, Tag> Storage for Links<B, Tag> {
    fn is_linked(&self) -> bool {
        self.next.is_some() || self.prev.is_some()
    }
}

impl<Tag> StorageAccess for LinkedListStorage<Tag> {
    type State<B: NodeBuilder> = Links<B, Tag>;
}

pub struct LinkedList<B: NodeBuilder, Tag = ()> {
    head: Option<NodeRef<B>>,
    tail: Option<NodeRef<B>>,
    len: usize,
    _marker: std::marker::PhantomData<Tag>,
}

impl<B: NodeBuilder, Tag> Default for LinkedList<B, Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: NodeBuilder, Tag> LinkedList<B, Tag> {
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

    pub fn push_back<const I: usize>(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
    ) -> Result<(), Error>
    where
        B::Storage: HasStorage<Links<B, Tag>, I>,
    {
        if builder.resolve(node).storage().storage().is_linked() {
            return Err(Error::AlreadyLinked);
        }

        match self.tail {
            None => {
                let links = node.resolve_mut(builder).storage_mut().storage_mut();
                links.next = Some(node);
                links.prev = Some(node);
            }
            Some(tail) => {
                let head = self.head.unwrap();
                if head != tail {
                    // len >= 2: head, tail and the unlinked node are distinct.
                    let [tail_node, head_node, new_node] =
                        builder.resolve_many_mut([tail, head, node]);
                    tail_node.storage_mut().storage_mut().next = Some(node);
                    head_node.storage_mut().storage_mut().prev = Some(node);
                    let new_links = new_node.storage_mut().storage_mut();
                    new_links.next = Some(head);
                    new_links.prev = Some(tail);
                } else {
                    // len == 1: the circle grows from the single node.
                    let [old_node, new_node] = builder.resolve_many_mut([tail, node]);
                    let old_links = old_node.storage_mut().storage_mut();
                    old_links.next = Some(node);
                    old_links.prev = Some(node);
                    let new_links = new_node.storage_mut().storage_mut();
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

    pub fn pop_front<const I: usize>(&mut self, builder: &mut B) -> Option<NodeRef<B>>
    where
        B::Storage: HasStorage<Links<B, Tag>, I>,
    {
        let head = self.head?;

        if self.len == 1 {
            let links = builder.resolve_mut(head).storage_mut().storage_mut();
            links.next = None;
            links.prev = None;
            self.head = None;
            self.tail = None;
            self.len = 0;
            return Some(head);
        }

        let next = builder.resolve(head).storage().storage().next.unwrap();

        if self.len == 2 {
            let [head_node, next_node] = builder.resolve_many_mut([head, next]);
            let head_links = head_node.storage_mut().storage_mut();
            head_links.next = None;
            head_links.prev = None;
            let next_links = next_node.storage_mut().storage_mut();
            next_links.next = Some(next);
            next_links.prev = Some(next);
        } else {
            let tail = self.tail.unwrap();
            let [head_node, next_node, tail_node] = builder.resolve_many_mut([head, next, tail]);
            let head_links = head_node.storage_mut().storage_mut();
            head_links.next = None;
            head_links.prev = None;
            next_node.storage_mut().storage_mut().prev = Some(tail);
            tail_node.storage_mut().storage_mut().next = Some(next);
        }

        self.head = Some(next);
        self.len -= 1;
        Some(head)
    }
}

#[cfg(all(test, feature = "bumpalo"))]
mod tests {
    use bumpalo::Bump;

    use super::*;
    use crate::Node;
    use crate::arena_node_builder::ArenaNodeBuilder;

    type Builder<'a, M> = ArenaNodeBuilder<'a, u32, M>;

    #[test]
    fn fifo_order() {
        let arena = Bump::new();
        let mut builder = Builder::<LinkedListStorage>::new(&arena);
        let mut list = LinkedList::new();

        let a = Node::build(1, &builder);
        let b = Node::build(2, &builder);
        let c = Node::build(3, &builder);
        list.push_back(&mut builder, a).unwrap();
        list.push_back(&mut builder, b).unwrap();
        list.push_back(&mut builder, c).unwrap();
        assert_eq!(list.len(), 3);

        assert_eq!(
            list.pop_front(&mut builder)
                .unwrap()
                .resolve(&builder)
                .value(),
            &1
        );
        assert_eq!(
            list.pop_front(&mut builder)
                .unwrap()
                .resolve(&builder)
                .value(),
            &2
        );
        assert_eq!(
            list.pop_front(&mut builder)
                .unwrap()
                .resolve(&builder)
                .value(),
            &3
        );
        assert!(list.pop_front(&mut builder).is_none());
        assert!(list.is_empty());
    }

    #[test]
    fn double_push_is_already_linked() {
        let arena = Bump::new();
        let mut builder = Builder::<LinkedListStorage>::new(&arena);
        let mut list = LinkedList::new();

        let a = Node::build(1, &builder);
        list.push_back(&mut builder, a).unwrap();
        assert_eq!(list.push_back(&mut builder, a), Err(Error::AlreadyLinked));
        assert_eq!(list.len(), 1);

        // The failed push did not corrupt the list.
        assert_eq!(
            list.pop_front(&mut builder)
                .unwrap()
                .resolve(&builder)
                .value(),
            &1
        );
    }

    #[test]
    fn popped_node_can_be_pushed_again() {
        let arena = Bump::new();
        let mut builder = Builder::<LinkedListStorage>::new(&arena);
        let mut list = LinkedList::new();

        let a = Node::build(1, &builder);
        list.push_back(&mut builder, a).unwrap();
        let a = list.pop_front(&mut builder).unwrap();
        list.push_back(&mut builder, a).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list.front(), Some(a));
    }

    #[test]
    fn circular_invariants() {
        let arena = Bump::new();
        let mut builder = Builder::<LinkedListStorage>::new(&arena);
        let mut list = LinkedList::new();

        let a = Node::build(1, &builder);
        let b = Node::build(2, &builder);
        let c = Node::build(3, &builder);
        list.push_back(&mut builder, a).unwrap();
        list.push_back(&mut builder, b).unwrap();
        list.push_back(&mut builder, c).unwrap();

        // tail.next == head and head.prev == tail.
        assert_eq!(a_links::<_, (), _>(&builder, a).prev, Some(c));
        assert_eq!(a_links::<_, (), _>(&builder, c).next, Some(a));

        // After popping the head, the new head's prev is still the tail.
        list.pop_front(&mut builder);
        assert_eq!(a_links::<_, (), _>(&builder, b).prev, Some(c));
        assert_eq!(a_links::<_, (), _>(&builder, c).next, Some(b));
    }

    fn a_links<B, Tag, const I: usize>(builder: &B, r: NodeRef<B>) -> &Links<B, Tag>
    where
        B: NodeBuilder,
        B::Storage: HasStorage<Links<B, Tag>, I>,
    {
        r.resolve(builder).storage().storage()
    }

    #[test]
    fn two_lists_via_multi_storage() {
        #[derive(Default)]
        struct Lru;

        type Membership0 = LinkedListStorage;
        type Membership1 = LinkedListStorage<Lru>;
        type Builder2<'a> = Builder<'a, (Membership0, Membership1)>;

        let arena = Bump::new();
        let mut builder = Builder2::new(&arena);

        let mut list: LinkedList<Builder2> = LinkedList::new();
        let mut lru: LinkedList<Builder2, Lru> = LinkedList::new();

        let a = Node::build(1, &builder);
        let b = Node::build(2, &builder);
        list.push_back(&mut builder, a).unwrap();
        list.push_back(&mut builder, b).unwrap();
        lru.push_back(&mut builder, a).unwrap();

        assert_eq!(list.len(), 2);
        assert_eq!(lru.len(), 1);

        // Same node, independent link state per tag.
        let links_a = HasStorage::<Links<_, ()>, _>::storage(a.resolve(&builder).storage());
        let links_a_lru = HasStorage::<Links<_, Lru>, _>::storage(a.resolve(&builder).storage());
        assert_eq!(links_a.next, Some(b));
        assert_eq!(links_a_lru.next, Some(a)); // singleton: self-linked

        // Removing from one list leaves the other intact.
        let popped = list.pop_front(&mut builder).unwrap();
        assert_eq!(popped.resolve(&builder).value(), &1);
        assert_eq!(lru.len(), 1);
        assert_eq!(
            lru.pop_front(&mut builder)
                .unwrap()
                .resolve(&builder)
                .value(),
            &1
        );

        // The composite storage's is_linked reflects both memberships.
        let c = Node::build(3, &builder);
        assert!(!builder.resolve(c).storage().is_linked());
        list.push_back(&mut builder, c).unwrap();
        assert!(builder.resolve(c).storage().is_linked());
    }
}
