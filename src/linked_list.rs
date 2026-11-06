// Concrete doubly-linked storage (Zero Cell, plain Option)
use crate::{NodeBuilder, NodeRef, Storage};

#[derive(PartialEq, Eq)]
pub struct LinkedListStorage<B: NodeBuilder> {
    pub next: Option<NodeRef<B>>,
    pub prev: Option<NodeRef<B>>,
}

impl<B: NodeBuilder> Default for LinkedListStorage<B> {
    fn default() -> Self {
        Self {
            next: None,
            prev: None,
        }
    }
}

impl<B: NodeBuilder> Storage for LinkedListStorage<B> {
    fn is_linked(&self) -> bool {
        self.next.is_some() || self.prev.is_some()
    }
}

pub struct LinkedList<B: NodeBuilder, S = LinkedListStorage<B>> {
    head: Option<NodeRef<B>>,
    tail: Option<NodeRef<B>>,
    _marker: std::marker::PhantomData<S>,
}

impl<B: NodeBuilder, S> LinkedList<B, S> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn push_back(&mut self, node: NodeRef<B>, builder: &mut B) {
        unimplemented!()
    }
}
