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

    fn next_nodes(&mut self) {}
}

// pub trait Queue<I, E> {
pub trait Queue {
    fn len(&self) -> usize;
    fn split_off(&mut self, at: usize) -> Self;
    // fn next(&mut self) -> Option<(usize, Result<I, E>)>;
}

#[cfg(feature = "rayon")]
mod par_iter {
    use crate::sync::par::*;
    use crate::sync::*;

    // pub trait GraphIterator<Q, I, E>: SplittableIterator
    pub trait HasQueue {
        type Queue: Queue;

        fn queue_mut(&mut self) -> &mut Self::Queue;
        fn queue(&self) -> &Self::Queue;
    }

    // pub trait GraphIterator<Q>
    pub trait GraphIterator<Q>
    where
        Q: Queue,
    {
        // fn max_depth(&self) -> Option<usize>;

        // fn from_split(queue: Q, max_depth: Option<usize>) -> Self;
        fn from_split(&self, queue: Q) -> Self;
    }
    // }

    // impl<Q, T> SplittableIterator for T
    // where
    //     // Q: Queue,
    //     T: GraphIterator // <Q>,
    // {

    impl<Q, T> SplittableIterator for T
    where
        Q: Queue,
        T: Iterator + HasQueue<Queue = Q> + GraphIterator<Q> + Sized,
    {
        fn split(&mut self) -> Option<Self> {
            let len = self.queue().len();
            // let len = self.queue.len();
            if len >= 2 {
                let split = self.queue_mut().split_off(len / 2);
                Some(self.from_split(split))
                // Some(Self::from_split(split))
                // None
                // Some(Self {
                //     queue: split,
                //     max_depth: self.max_depth,
                // })
            } else {
                None
            }
        }
        // fn split(&mut self, at: usize) -> Self {
        //     Self::from_split()
        // }
    }
}

#[cfg(feature = "rayon")]
pub use par_iter::*;

// pub trait CloneableIterator: Iterator + Clone + Sized {}

pub type NodeIter<I, E> = Result<Box<dyn Iterator<Item = Result<I, E>>>, E>;
// pub type NodeIter<I, E> = Result<Box<dyn CloneableIterator<Item = Result<I, E>>>, E>;

// pub struct NodeIter<I, E> {
//     inner: Box<dyn Iterator<Item = Result<I, E>>>,
// }

// impl<I, E> From<E> for NodeIter<I, E> {
//     fn from(err: E) -> Self {
//         let inner = [Err(err)].into_iter();
//         NodeIter { inner }
//         // Ok(iter)
//     }
// }

// impl<Iter, I, E> From<Iter> for NodeIter<I, E>
// where
//     Iter: IntoIterator<Item = Result<I, E>>,
// {
//     fn from(inner: Iter) -> Self {
//         // let inner = [Err(err)].into_iter();
//         let inner = inner.into_iter();
//         NodeIter { inner }
//         // Ok(iter)
//     }
// }

// pub type BoxedNode<E> = Box<dyn Node<Error = E, Item = Self>>;
// pub type BoxedNode<E> = Box<dyn Node<Error = E>>;

// pub struct NodeIter<N, I>
// where
//     N: Node,
//     I: Iterator<Item = Result<N, N::Error>>,
// {
//     iter: I,
// }

pub trait Node: std::fmt::Debug
where
    Self: Sized,
{
    type Error: std::fmt::Debug;
    // type Iter: impl Iterator<Item = Result<Self, Self::Error>;
    // type Item;

    // fn children(&self) -> Result<NodeIter<Self, Self::Error>, Self::Error>
    // fn children(&self) -> Result<NodeIter<Self::Item, Self::Error>, Self::Error>
    // fn children(&self, depth: usize) -> NodeIter<Self::Item, Self::Error>;
    // fn children<Iter>(&self, depth: usize) -> Result<NodeIter<_, Self>, Self::Error>;
    // fn children<Iter>(&self, depth: usize) -> Result<Iter, Self::Error>
    // where
    //     Iter: IntoIterator<Item = Result<Self, Self::Error>>;
    // fn children(&self, depth: usize) -> Result<Self::Iter, Self::Error>;
    #[inline]
    fn children(&self, depth: usize) -> NodeIter<Self, Self::Error>;
    // where
    //     Self: Sized;
}

pub trait FastNode: std::fmt::Debug
where
    Self: Sized,
{
    type Error;

    #[inline]
    fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
    where
        E: ExtendQueue<Self, Self::Error>;
}

// use rayon::iter::ParallelBridge;

// impl ParallelIterator for TaskQueue {
//     type Item = QueueItem;

//     fn drive_unindexed<C>(self, consumer: C) -> C::Result
//     where
//         C: UnindexedConsumer<QueueItem>,
//     {
//         self.next().unwrap()
//     }
// }

// pub struct ParDfs<N>
// where
//     N: Node, // <Error = E, Item = N> + Send,
// {
//     inner: Dfs<N>,
// }

// impl<N, E> rayon::iter::IntoParallelIterator for Dfs<N>
// where
//     N: Node<Error = E, Item = N> + Send,
//     E: Send,
// {
//     type Iter = ParDfs<N>;
//     type Item = <ParDfs<N> as rayon::iter::ParallelIterator>::Item;

//     fn into_par_iter(self) -> Self::Iter {
//         ParDfs { inner: self }
//     }
// }

// impl<N, E> rayon::iter::ParallelIterator for ParDfs<N>
// where
//     N: Node<Error = E, Item = N> + Send,
//     E: Send,
// {
//     type Item = <Dfs<N> as Iterator>::Item;

//     fn drive_unindexed<C>(self, consumer: C) -> C::Result
//     where
//         C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
//     {
//         self.inner.next()
//         // todo!()
//         // Err()
//         // self.next().unwrap()
//     }
// }
