#![allow(unused_variables)]

use crate::{Error, IntrusivePriorityQueue, NodeBuilder, NodeRef};
use core::{cell::Cell, ptr};

#[derive(Debug, Default)]
pub struct BinaryHeapHook {
    index: Cell<Option<usize>>,
}

impl BinaryHeapHook {
    pub const fn new() -> Self {
        Self {
            index: Cell::new(None),
        }
    }

    #[inline]
    pub fn is_queued(&self) -> bool {
        self.index.get().is_some()
    }
}

pub struct IntrusiveBinaryHeap<B: NodeBuilder> {
    nodes: Vec<NodeRef<B>>,
}

impl<B: NodeBuilder> IntrusiveBinaryHeap<B> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<B: NodeBuilder> Default for IntrusiveBinaryHeap<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: NodeBuilder> IntrusiveBinaryHeap<B> {
    #[inline]
    fn swap_nodes(&mut self, i: usize, j: usize) {
        self.nodes.swap(i, j);
        self.nodes[i].hook().index.set(Some(i));
        self.nodes[j].hook().index.set(Some(j));
    }

    fn sift_up(&mut self, mut pos: usize) {
        while pos > 0 {
            let parent = (pos - 1) / 2;
            if self.nodes[pos].priority() < self.nodes[parent].priority() {
                self.swap_nodes(pos, parent);
                pos = parent;
            } else {
                break;
            }
        }
    }

    fn sift_down(&mut self, mut pos: usize) {
        let len = self.nodes.len();
        loop {
            let left = 2 * pos + 1;
            let right = 2 * pos + 2;
            let mut smallest = pos;

            if left < len && self.nodes[left].priority() < self.nodes[smallest].priority() {
                smallest = left;
            }
            if right < len && self.nodes[right].priority() < self.nodes[smallest].priority() {
                smallest = right;
            }

            if smallest != pos {
                self.swap_nodes(pos, smallest);
                pos = smallest;
            } else {
                break;
            }
        }
    }

    #[inline]
    fn validate_membership(&self, node: NodeRef<B>) -> Result<usize, Error> {
        let idx = node.hook().index.get().ok_or(Error::NotLinked)?;
        if idx >= self.nodes.len() || !ptr::eq(self.nodes[idx], node) {
            return Err(Error::NotLinked);
        }
        Ok(idx)
    }
}

impl<B: NodeBuilder> IntrusivePriorityQueue<B> for IntrusiveBinaryHeap<B> {
    type Priority = u32;

    fn push(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
        priority: Self::Priority,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn peek(&self, builder: &B) -> Option<Node<B>> {
        unimplemented!()
    }

    fn pop(&mut self, builder: &mut B) -> Option<Node<B>> {
        unimplemented!()
    }

    fn decrease_key(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
        priority: Self::Priority,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn push_or_decrease_key(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
        priority: Self::Priority,
    ) -> Result<bool, Error> {
        unimplemented!()
    }

    fn remove(&mut self, builder: &mut B, node: NodeRef<B>) -> Result<NodeRef<B>, Error> {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self, builder: &mut B) {
        unimplemented!()
    }
}
