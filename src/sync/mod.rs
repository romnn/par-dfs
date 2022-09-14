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
    // fn len(&self) -> usize;
    fn split_off(&mut self, at: usize) -> Self;
}

// #[cfg(feature = "rayon")]
// mod par_iter {
//     use crate::sync::par::*;
//     use crate::sync::*;

//     pub trait HasQueue {
//         type Queue: Queue;

//         fn queue_mut(&mut self) -> &mut Self::Queue;
//         fn queue(&self) -> &Self::Queue;
//     }

//     pub trait GraphIterator<Q>
//     where
//         Q: Queue,
//     {
//         fn from_split(&self, queue: Q) -> Self;
//     }

//     impl<Q, T> SplittableIterator for T
//     where
//         Q: Queue,
//         T: Iterator + HasQueue<Queue = Q> + GraphIterator<Q> + Sized,
//     {
//         fn split(&mut self) -> Option<Self> {
//             let len = self.queue().len();
//             if len >= 2 {
//                 let split = self.queue_mut().split_off(len / 2);
//                 Some(self.from_split(split))
//             } else {
//                 None
//             }
//         }
//     }
// }

// #[cfg(feature = "rayon")]
// pub use par_iter::*;

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
