pub mod bfs;
pub mod dfs;
#[cfg(feature = "rayon")]
pub mod par;

pub use bfs::*;
pub use dfs::*;

use std::iter::{IntoIterator, Iterator};

pub trait ExtendQueue<I, E> {
    fn add(&mut self, depth: usize, item: Result<I, E>);

    fn add_all<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>;
}

pub trait Queue {
    fn split_off(&mut self, at: usize) -> Self;
}

pub type NodeIter<I, E> = Result<Box<dyn Iterator<Item = Result<I, E>>>, E>;

pub trait Node: std::fmt::Debug
where
    Self: Sized,
{
    type Error: std::fmt::Debug;

    #[inline]
    fn children(&self, depth: usize) -> NodeIter<Self, Self::Error>;
}

pub trait FastNode: std::fmt::Debug
where
    Self: Sized,
{
    type Error: std::fmt::Debug;

    #[inline]
    fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
    where
        E: ExtendQueue<Self, Self::Error>;
}
