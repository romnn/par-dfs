pub mod bfs;
pub mod dfs;
#[cfg(feature = "rayon")]
pub mod par;
pub mod queue;

pub use bfs::*;
pub use dfs::*;

use std::hash::Hash;
use std::iter::{IntoIterator, Iterator};

pub trait ExtendQueue<I, E> {
    fn add(&mut self, depth: usize, item: Result<I, E>);

    fn add_all<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>;
}

pub trait Queue<I, E> {
    fn len(&self) -> usize;
    fn pop_back(&mut self) -> Option<(usize, Result<I, E>)>;
    fn pop_front(&mut self) -> Option<(usize, Result<I, E>)>;
    fn split_off(&mut self, at: usize) -> Self;
}

pub type NodeIter<I, E> = Result<Box<dyn Iterator<Item = Result<I, E>>>, E>;

pub trait Node
where
    Self: Hash + Eq + Clone + std::fmt::Debug,
{
    type Error: Hash + Eq + Clone + std::fmt::Debug;

    fn children(&self, depth: usize) -> NodeIter<Self, Self::Error>;
}

pub trait FastNode
where
    Self: Hash + Eq + Clone + std::fmt::Debug,
{
    type Error: Hash + Eq + Clone + std::fmt::Debug;

    fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
    where
        E: ExtendQueue<Self, Self::Error>;
}
