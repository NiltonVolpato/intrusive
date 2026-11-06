use core::fmt::Debug;

// pub mod binary_heap;
#[cfg(feature = "bumpalo")]
pub mod arena_node_builder;
pub mod linked_list;
// pub mod priority_queue;

pub trait Storage: Default {
    fn is_linked(&self) -> bool;
}

// Disambiguation wrapper for identical storage types
#[derive(Default)]
pub struct Tagged<S, Tag> {
    pub(crate) storage: S,
    _marker: std::marker::PhantomData<Tag>,
}

impl<S: Storage, Tag: Default> Storage for Tagged<S, Tag> {
    fn is_linked(&self) -> bool {
        self.storage.is_linked()
    }
}

pub trait HasStorage<S> {
    fn storage(&self) -> &S;
    fn storage_mut(&mut self) -> &mut S;
}

impl<S> HasStorage<S> for S
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

pub trait NodeBuilder: Sized + Clone + Debug {
    type Value;
    type Storage: Storage;
    type Handle: Copy + Eq;

    fn build(&self, node: Node<Self>) -> NodeRef<Self>;
    fn resolve(&self, node_ref: NodeRef<Self>) -> &Node<Self>;
    fn resolve_mut(&mut self, node_ref: NodeRef<Self>) -> &mut Node<Self>;
    fn resolve_many_mut<const N: usize>(
        &mut self,
        node_refs: [NodeRef<Self>; N],
    ) -> [&mut Node<Self>; N];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRef<B: NodeBuilder> {
    handle: B::Handle,
}

impl<B: NodeBuilder> NodeRef<B> {
    pub fn new(handle: B::Handle) -> Self {
        Self { handle }
    }

    #[inline(always)]
    pub fn resolve(self, builder: &B) -> &Node<B> {
        builder.resolve(self)
    }

    #[inline(always)]
    fn resolve_mut(self, builder: &mut B) -> &mut Node<B> {
        builder.resolve_mut(self)
    }
}

#[derive(Debug)]
pub struct Node<B: NodeBuilder> {
    value: B::Value,
    storage: B::Storage,
}

impl<B: NodeBuilder> Node<B> {
    pub fn build(value: B::Value, builder: &B) -> NodeRef<B> {
        builder.build(Node {
            value,
            storage: Default::default(),
        })
    }

    #[inline(always)]
    pub fn value(&self) -> &B::Value {
        &self.value
    }

    #[inline(always)]
    pub fn value_mut(&mut self) -> &mut B::Value {
        &mut self.value
    }
}
