#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    AlreadyLinked,
    NotLinked,
    NotDecreased,
}

pub trait IntrusivePriorityQueue<B: NodeBuilder> {
    type Priority: Ord + Copy + Debug;

    fn push(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
        priority: Self::Priority,
    ) -> Result<(), Error>;

    fn peek(&self, builder: &B) -> Option<Node<B>>;

    fn pop(&mut self, builder: &mut B) -> Option<Node<B>>;

    fn decrease_key(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
        priority: Self::Priority,
    ) -> Result<(), Error>;

    fn push_or_decrease_key(
        &mut self,
        builder: &mut B,
        node: NodeRef<B>,
        priority: Self::Priority,
    ) -> Result<bool, Error>;

    fn remove(&mut self, builder: &mut B, node: NodeRef<B>) -> Result<NodeRef<B>, Error>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self, builder: &mut B);
}
