//! Composite node storage, for a node in several containers at once.
//!
//! A tuple of markers is itself a marker, e.g.
//! `(LinkedListStorage, LinkedListStorage<Lru>)`, and its state is the tuple
//! of the member states. `HasStorage` finds a member by its tuple index,
//! which is inferred from the requested state type; this is unambiguous as
//! long as the member states are distinct types (e.g. different tags).
use crate::{HasStorage, NodeBuilder, Storage, StorageAccess};

macro_rules! tuple_storage {
    ($($M:ident $S:ident $i:tt),+) => {
        impl<$($S: Storage),+> Storage for ($($S,)+) {
            fn is_linked(&self) -> bool {
                false $(|| self.$i.is_linked())+
            }
        }

        impl<$($M: StorageAccess),+> StorageAccess for ($($M,)+) {
            type State<B: NodeBuilder> = ($($M::State<B>,)+);
        }

        tuple_storage!(@each [$($S),+] $($S $i),+);
    };
    // Re-bind the full list as one tt so it can be repeated per element.
    (@each $all:tt $($S:ident $i:tt),+) => {
        $(tuple_storage!(@has $all $S $i);)+
    };
    (@has [$($T:ident),+] $S:ident $i:tt) => {
        impl<$($T: Storage),+> HasStorage<$S, $i> for ($($T,)+) {
            #[inline(always)]
            fn storage(&self) -> &$S {
                &self.$i
            }

            #[inline(always)]
            fn storage_mut(&mut self) -> &mut $S {
                &mut self.$i
            }
        }
    };
}

tuple_storage!(M0 S0 0, M1 S1 1);
tuple_storage!(M0 S0 0, M1 S1 1, M2 S2 2);
tuple_storage!(M0 S0 0, M1 S1 1, M2 S2 2, M3 S3 3);
