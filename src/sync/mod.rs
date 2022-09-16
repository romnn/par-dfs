pub mod bfs;
pub mod dfs;
#[cfg(feature = "rayon")]
#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
pub mod par;
pub mod queue;

pub use bfs::{Bfs, FastBfs};
pub use dfs::{Dfs, FastDfs};

use std::hash::Hash;
use std::iter::{IntoIterator, Iterator};

/// Extend a queue with the contents of an [`Iterator`].
///
/// Queues to be used by [`FastNode`] must implement this trait.
///
/// [`FastNode`]: trait@crate::sync::FastNode
/// [`Iterator`]: trait@std::iter::Iterator
pub trait ExtendQueue<I, E> {
    /// Add single item with given depth to the queue.
    fn add(&mut self, depth: usize, item: Result<I, E>);

    /// Extend the queue with the contents of an [`Iterator`].
    ///
    /// [`Iterator`]: trait@std::iter::Iterator
    fn add_all<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>;
}

/// A Queue that can be split and allows removing elements from the front or back.
pub(crate) trait Queue<I, E> {
    /// Returns the number of items in the queue
    fn len(&self) -> usize;

    /// Returns `true` if the queue is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pops the last item from the queue and returns it, or [`None`] if it is empty.
    /// [`None`]: enum@std::option::Option::None
    fn pop_back(&mut self) -> Option<(usize, Result<I, E>)>;

    /// Pops the first item from the queue and returns it, or [`None`] if it is empty.
    ///
    /// [`None`]: enum@std::option::Option::None
    fn pop_front(&mut self) -> Option<(usize, Result<I, E>)>;

    #[must_use]
    /// Splits the queue into two at the given index.
    /// Returns a newly allocated queue containing the elements in the range `[at, len)`.
    /// After the call, the original vector will be left containing the elements `[0, at)` with its previous capacity unchanged.
    ///
    /// # Panics
    ///   
    /// Panics if `at > self.len()`
    fn split_off(&mut self, at: usize) -> Self;
}

/// A boxed [`Iterator`] of [`Node`]s.
///
/// [`Iterator`]: trait@std::iter::Iterator
/// [`Node`]: trait@crate::sync::Node
pub type NodeIter<I, E> = Result<Box<dyn Iterator<Item = Result<I, E>>>, E>;

/// A node with produces an [`Iterator`] of children [`Node`]s for a given depth.
///
/// [`Iterator`]: trait@std::iter::Iterator
/// [`Node`]: trait@crate::sync::Node
pub trait Node
where
    Self: Hash + Eq + Clone + std::fmt::Debug,
{
    /// The type of the error when producing children fails.
    type Error: std::fmt::Debug;

    /// Returns an [`Iterator`] over its children [`Node`]s.
    ///
    /// # Errors
    ///
    /// Should return [`Self::Error`] if the iterator cannot be crated.
    ///
    /// [`Iterator`]: trait@std::iter::Iterator
    /// [`Node`]: trait@crate::sync::Node
    /// [`Self::Error`]: type@crate::async::Node::Error
    fn children(&self, depth: usize) -> NodeIter<Self, Self::Error>;
}

/// A node which adds children [`Node`]s to a queue in place.
pub trait FastNode
where
    Self: Hash + Eq + Clone + std::fmt::Debug,
{
    /// The type of the error when adding children fails.
    type Error: std::fmt::Debug;

    /// Callback for adding children [`Node`]s to a queue implementing [`ExtendQueue`].
    ///
    /// # Errors
    ///
    /// Should return `Self::Error` if the children could not be added.
    ///
    /// [`ExtendQueue`]: trait@crate::sync::ExtendQueue
    /// [`Node`]: trait@crate::sync::Node
    /// [`Self::Error`]: type@crate::async::Node::Error
    fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
    where
        E: ExtendQueue<Self, Self::Error>;
}
