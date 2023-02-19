mod entry;
mod error;
mod generic;
mod index;

pub use self::{
    entry::Entry, error::SparseSetError, generic::SparseSetGeneric, index::SparseSetIndex,
};

pub type SparseSet<V> = SparseSetGeneric<usize, V>;
