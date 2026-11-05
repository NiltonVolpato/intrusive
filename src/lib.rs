pub mod binary_heap;

/// Trait implemented by user domain structs to participate in intrusive priority queues.
pub trait IntrusiveNode {
    type Priority: Ord + Copy;

    /// Embedded metadata container required by the heap implementation.
    type Hook: Default;

    /// Reads the current priority key.
    fn priority(&self) -> Self::Priority;

    /// Sets a new priority key (typically backed by a `Cell`).
    fn set_priority(&self, priority: Self::Priority);

    /// Exposes a reference to the embedded heap hook.
    fn hook(&self) -> &Self::Hook;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecreaseKeyError<P> {
    NotPresent,
    /// For min-heaps, new priority must be strictly less than current priority.
    NotDecreased {
        current: P,
        attempted: P,
    },
}

/// Abstract intrusive priority queue interface.
pub trait IntrusivePriorityQueue<'a> {
    type Node: IntrusiveNode;

    fn push(&mut self, node: &'a Self::Node);
    fn peek(&self) -> Option<&'a Self::Node>;
    fn pop(&mut self) -> Option<&'a Self::Node>;

    fn decrease_key(
        &mut self,
        node: &'a Self::Node,
        new_priority: <Self::Node as IntrusiveNode>::Priority,
    ) -> Result<(), DecreaseKeyError<<Self::Node as IntrusiveNode>::Priority>>;

    fn remove(
        &mut self,
        node: &'a Self::Node,
    ) -> Result<(), DecreaseKeyError<<Self::Node as IntrusiveNode>::Priority>>;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self);
}
