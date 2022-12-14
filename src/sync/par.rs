//! Support for parallel iteration using [`rayon`].
//!
//! To efficiently parallelize dynamically growing iterators,
//! whose size is not known upfront, the [`ParallelSplittableIterator`] bridge
//! can be used for any iterator that implements [`SplittableIterator`].
//!
//! [`ParallelSplittableIterator`]
//! implements [`rayon::iter::ParallelIterator`].
//!
//! ### Acknowledgements
//!
//! This approach is taken from the amazing [blog post by tavianator](https://tavianator.com/2022/parallel_graph_search.html).
//!
//! [`rayon`]: mod@rayon
//! [`ParallelSplittableIterator`]: struct@self::ParallelSplittableIterator
//! [`SplittableIterator`]: trait@self::SplittableIterator
//! [`rayon::iter::ParallelIterator`]: trait@rayon::iter::ParallelIterator

use rayon::iter::plumbing::{Folder, Reducer, UnindexedConsumer};
use rayon::iter::ParallelIterator;
use rayon::{current_num_threads, join_context};
use std::iter::Iterator;

/// An iterator that can be split.
pub trait SplittableIterator: Iterator + Sized {
    /// Split this iterator in two, if possible.
    ///
    /// Returns a newly allocated [`SplittableIterator`] of the second half,
    /// or [`None`], if the iterator is too small to split.
    ///
    /// After the call, [`self`]
    /// will be left containing the first half.
    ///
    /// [`None`]: type@std::option::Option::None
    /// [`self`]: trait@self::SplittableIterator
    fn split(&mut self) -> Option<Self>;
}

/// Converts a [`SplittableIterator`] into a [`rayon::iter::ParallelIterator`].
pub trait IntoParallelIterator: Sized {
    /// Parallelizes this iterator.
    ///
    /// Returns a [`ParallelSplittableIterator`] bridge that implements
    /// [`rayon::iter::ParallelIterator`].
    fn into_par_iter(self) -> ParallelSplittableIterator<Self>;
}

impl<T> IntoParallelIterator for T
where
    T: SplittableIterator + Send,
    T::Item: Send,
{
    fn into_par_iter(self) -> ParallelSplittableIterator<Self> {
        ParallelSplittableIterator::new(self)
    }
}

/// A bridge from a [`SplittableIterator`] to a [`rayon::iter::ParallelIterator`].
pub struct ParallelSplittableIterator<Iter> {
    iter: Iter,
    splits: usize,
}

impl<Iter> ParallelSplittableIterator<Iter>
where
    Iter: SplittableIterator,
{
    /// Creates a new [`ParallelSplittableIterator`] bridge from a [`SplittableIterator`].
    pub fn new(iter: Iter) -> Self {
        Self {
            iter,
            splits: current_num_threads(),
        }
    }

    /// Split the underlying iterator in half.
    fn split(&mut self) -> Option<Self> {
        if self.splits == 0 {
            return None;
        }

        if let Some(split) = self.iter.split() {
            self.splits /= 2;

            Some(Self {
                iter: split,
                splits: self.splits,
            })
        } else {
            None
        }
    }

    /// Bridge to an [`UnindexedConsumer`].
    ///
    /// [`UnindexedConsumer`]: struct@rayon::iter::plumbing::UnindexedConsumer
    fn bridge<C>(&mut self, stolen: bool, consumer: C) -> C::Result
    where
        Iter: Send,
        C: UnindexedConsumer<Iter::Item>,
    {
        // Thief-splitting: start with enough splits to fill the thread pool,
        // and reset every time a job is stolen by another thread.
        if stolen {
            self.splits = current_num_threads();
        }

        let mut folder = consumer.split_off_left().into_folder();

        if self.splits == 0 {
            return folder.consume_iter(&mut self.iter).complete();
        }

        while !folder.full() {
            // Try to split
            if let Some(mut split) = self.split() {
                let (r1, r2) = (consumer.to_reducer(), consumer.to_reducer());
                let left_consumer = consumer.split_off_left();

                let (left, right) = join_context(
                    |ctx| self.bridge(ctx.migrated(), left_consumer),
                    |ctx| split.bridge(ctx.migrated(), consumer),
                );
                return r1.reduce(folder.complete(), r2.reduce(left, right));
            }

            // Otherwise, consume an item and try again
            if let Some(next) = self.iter.next() {
                folder = folder.consume(next);
            } else {
                break;
            }
        }

        folder.complete()
    }
}

impl<Iter> ParallelIterator for ParallelSplittableIterator<Iter>
where
    Iter: SplittableIterator + Send,
    Iter::Item: Send,
{
    type Item = Iter::Item;

    fn drive_unindexed<C>(mut self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.bridge(false, consumer)
    }
}

macro_rules! parallel_iterator {
    ($iter:ident<$node:ident>) => {
        impl<N> $crate::sync::par::SplittableIterator for $iter<N>
        where
            N: $node,
        {
            fn split(&mut self) -> Option<Self> {
                use $crate::sync::Queue;
                let len = self.queue.len();
                if len >= 2 {
                    let split = self.queue.split_off(len / 2);
                    Some(Self {
                        queue: split,
                        // visited: self.visited.clone(),
                        max_depth: self.max_depth,
                        // allow_circles: self.allow_circles,
                    })
                } else {
                    None
                }
            }
        }

        impl<N> rayon::iter::IntoParallelIterator for $iter<N>
        where
            N: $node + Sync + Send,
            N::Error: Send,
        {
            type Iter = $crate::sync::par::ParallelSplittableIterator<Self>;
            type Item = <Self as Iterator>::Item;

            fn into_par_iter(self) -> Self::Iter {
                $crate::sync::par::ParallelSplittableIterator::new(self)
            }
        }
    };
}
pub(crate) use parallel_iterator;
