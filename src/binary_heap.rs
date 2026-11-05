use crate::{DecreaseKeyError, IntrusiveNode, IntrusivePriorityQueue};
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

pub struct IntrusiveBinaryHeap<'a, N> {
    nodes: Vec<&'a N>,
}

impl<'a, N> IntrusiveBinaryHeap<'a, N> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<'a, N> Default for IntrusiveBinaryHeap<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, N> IntrusiveBinaryHeap<'a, N>
where
    N: IntrusiveNode<Hook = BinaryHeapHook>,
{
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
    fn validate_membership(&self, node: &'a N) -> Result<usize, DecreaseKeyError<N::Priority>> {
        let idx = node
            .hook()
            .index
            .get()
            .ok_or(DecreaseKeyError::NotPresent)?;
        if idx >= self.nodes.len() || !ptr::eq(self.nodes[idx], node) {
            return Err(DecreaseKeyError::NotPresent);
        }
        Ok(idx)
    }
}

impl<'a, N> IntrusivePriorityQueue<'a> for IntrusiveBinaryHeap<'a, N>
where
    N: IntrusiveNode<Hook = BinaryHeapHook>,
{
    type Node = N;

    fn push(&mut self, node: &'a N) {
        assert!(
            node.hook().index.get().is_none(),
            "Cannot push an item that is already enqueued"
        );

        let idx = self.nodes.len();
        node.hook().index.set(Some(idx));
        self.nodes.push(node);
        self.sift_up(idx);
    }

    fn peek(&self) -> Option<&'a N> {
        self.nodes.first().copied()
    }

    fn pop(&mut self) -> Option<&'a N> {
        if self.nodes.is_empty() {
            return None;
        }

        let last_idx = self.nodes.len() - 1;
        self.swap_nodes(0, last_idx);

        let node = self.nodes.pop().unwrap();
        node.hook().index.set(None);

        if !self.nodes.is_empty() {
            self.sift_down(0);
        }

        Some(node)
    }

    fn decrease_key(
        &mut self,
        node: &'a N,
        new_priority: N::Priority,
    ) -> Result<(), DecreaseKeyError<N::Priority>> {
        let idx = self.validate_membership(node)?;
        let current = node.priority();

        if new_priority >= current {
            return Err(DecreaseKeyError::NotDecreased {
                current,
                attempted: new_priority,
            });
        }

        node.set_priority(new_priority);
        self.sift_up(idx);
        Ok(())
    }

    fn remove(&mut self, node: &'a N) -> Result<(), DecreaseKeyError<N::Priority>> {
        let idx = self.validate_membership(node)?;
        let last_idx = self.nodes.len() - 1;

        if idx != last_idx {
            self.swap_nodes(idx, last_idx);
        }

        let removed = self.nodes.pop().unwrap();
        removed.hook().index.set(None);

        if idx < self.nodes.len() {
            let parent = (idx.saturating_sub(1)) / 2;
            if idx > 0 && self.nodes[idx].priority() < self.nodes[parent].priority() {
                self.sift_up(idx);
            } else {
                self.sift_down(idx);
            }
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn clear(&mut self) {
        for node in &self.nodes {
            node.hook().index.set(None);
        }
        self.nodes.clear();
    }
}
