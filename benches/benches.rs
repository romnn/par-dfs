#[cfg(feature = "async")]
use par_dfs::r#async;
#[cfg(feature = "sync")]
use par_dfs::sync;

use criterion::{black_box, criterion_group};
use std::convert::Infallible;
use std::iter::Iterator;

/// Enumerates the numbers that reach the given starting point when iterating
/// the [Collatz] map, by depth-first search over the [graph] of their orbits.
///
/// [Collatz]: https://en.wikipedia.org/wiki/Collatz_conjecture
/// [graph]: https://en.wikipedia.org/wiki/File:Collatz_orbits_of_the_all_integers_up_to_1000.svg

pub mod custom_dfs {
    use std::collections::{HashSet, VecDeque};
    use std::convert::Infallible;
    use std::iter::Iterator;

    type Queue = VecDeque<(usize, Result<u32, Infallible>)>;

    #[derive(Clone, Debug)]
    pub struct CollatzDfs {
        max_depth: Option<usize>,
        queue: Queue,
        visited: HashSet<u32>,
        allow_circles: bool,
    }

    impl CollatzDfs {
        pub fn new<D: Into<Option<usize>>>(start: u32, max_depth: D, allow_circles: bool) -> Self {
            Self {
                max_depth: max_depth.into(),
                queue: VecDeque::from_iter([(0, Ok(start))]),
                visited: HashSet::from_iter([start]),
                allow_circles,
            }
        }
    }

    impl Iterator for CollatzDfs {
        type Item = Result<u32, Infallible>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            match self.queue.pop_back() {
                Some((depth, Ok(n))) => {
                    if let Some(max_depth) = self.max_depth {
                        if max_depth >= depth {
                            return Some(Ok(n));
                        }
                    }
                    // n can be reached by dividing by two
                    // as long as it doesn't overflow
                    if let Some(even) = n.checked_mul(2) {
                        if self.allow_circles || !self.visited.contains(&even) {
                            self.queue.push_back((depth, Ok(even)));
                        }
                    }

                    // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
                    if n > 4 && n % 6 == 4 {
                        let odd = (n - 1) / 3;
                        if self.allow_circles || !self.visited.contains(&odd) {
                            self.queue.push_back((depth, Ok(odd)));
                        }
                    }
                    Some(Ok(n))
                }
                Some((_, n)) => Some(n),
                None => None,
            }
        }
    }

    #[cfg(all(feature = "sync", feature = "rayon"))]
    impl par_dfs::sync::par::SplittableIterator for CollatzDfs {
        fn split(&mut self) -> Option<Self> {
            let len = self.queue.len();
            if len >= 2 {
                let split = self.queue.split_off(len / 2);
                // cannot avoid circles when running in parallel
                self.visited.clear();
                self.allow_circles = true;
                Some(Self {
                    queue: split,
                    visited: HashSet::new(),
                    allow_circles: true,
                    max_depth: self.max_depth,
                })
            } else {
                None
            }
        }
    }
}

pub use custom_dfs::*;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct CollatzNode(pub u32);

impl From<u32> for CollatzNode {
    #[inline]
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl CollatzNode {
    #[inline]
    pub fn collatz_children(&self) -> impl Iterator<Item = Result<CollatzNode, Infallible>> {
        let n = self.0;
        let mut children = vec![];

        // n can be reached by dividing by two
        // as long as it doesn't overflow
        if let Some(even) = n.checked_mul(2) {
            children.push(even);
        }

        // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
        if n > 4 && n % 6 == 4 {
            children.push((n - 1) / 3);
        }
        children.into_iter().map(Self).map(Result::Ok)
    }
}

#[cfg(feature = "async")]
mod async_collatz {
    use super::CollatzNode;
    use async_trait::async_trait;
    use futures::StreamExt;
    use par_dfs::r#async::{Node, NodeStream};
    use std::convert::Infallible;
    use std::sync::Arc;

    #[async_trait]
    impl Node for CollatzNode {
        type Error = Infallible;

        #[inline]
        async fn children(
            self: Arc<Self>,
            _depth: usize,
        ) -> Result<NodeStream<Self, Self::Error>, Self::Error> {
            // let stream = tokio::task::spawn_blocking(move || {
            //     futures::stream::iter(self.collatz_children()).boxed()
            // })
            // .await
            // .unwrap();
            let stream = futures::stream::iter(self.collatz_children()).boxed();
            Ok(Box::pin(stream))
        }
    }
}

#[cfg(feature = "async")]
pub use async_collatz::*;

#[cfg(feature = "sync")]
mod sync_collatz {
    use super::CollatzNode;
    use par_dfs::sync::{ExtendQueue, FastNode, Node, NodeIter};
    use std::convert::Infallible;

    impl FastNode for CollatzNode {
        type Error = Infallible;

        #[inline]
        fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
        where
            E: ExtendQueue<Self, Self::Error>,
        {
            let n = self.0;

            // n can be reached by dividing by two
            // as long as it doesn't overflow
            if let Some(even) = n.checked_mul(2) {
                queue.add(depth, Ok(Self(even)));
            }

            // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
            if n > 4 && n % 6 == 4 {
                queue.add(depth, Ok(Self((n - 1) / 3)));
            }
            Ok(())
        }
    }

    impl Node for CollatzNode {
        type Error = Infallible;

        #[inline]
        fn children(&self, _depth: usize) -> NodeIter<Self, Self::Error> {
            Ok(Box::new(self.collatz_children()))
        }
    }
}

#[cfg(feature = "sync")]
pub use sync_collatz::*;

const START: u32 = 1;
const SYNC_LIMIT: Option<usize> = Some(1_00);
const ASYNC_LIMIT: Option<usize> = Some(60);
const CIRCLES: bool = true;

fn configure_group<M>(group: &mut criterion::BenchmarkGroup<M>)
where
    M: criterion::measurement::Measurement,
{
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
}

macro_rules! bench_collatz_async {
    ($name:ident: $group:literal, $iter:expr) => {
        /// Benchmarks for [Collatz] $name.
        fn $name(c: &mut criterion::Criterion) {
            let mut group = c.benchmark_group($group);
            configure_group(&mut group);

            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("build tokio runtime");

            group.bench_function("ordered", |b| {
                b.to_async(&runtime).iter(|| async {
                    use futures::StreamExt;
                    $iter
                        .map(|node| async move { node })
                        .buffered(8)
                        .count()
                        .await;
                })
            });

            group.bench_function("unordered", |b| {
                b.to_async(&runtime).iter(|| async {
                    use futures::StreamExt;
                    $iter
                        .map(|node| async move { node })
                        .buffer_unordered(8)
                        .count()
                        .await;
                })
            });
        }
    };
}

macro_rules! bench_collatz_sync {
    ($name:ident: $group:literal, $iter:expr) => {
        /// Benchmarks for [Collatz] $name.
        fn $name(c: &mut criterion::Criterion) {
            let mut group = c.benchmark_group($group);
            configure_group(&mut group);
            let iter = $iter;

            group.bench_function("sequential", |b| {
                b.iter(|| {
                    iter.clone().count();
                })
            });

            #[cfg(feature = "rayon")]
            group.bench_function(
                format!("parallel bridge ({} threads)", rayon::current_num_threads()),
                |b| {
                    b.iter(|| {
                        use rayon::iter::{ParallelBridge, ParallelIterator};
                        iter.clone().par_bridge().count()
                    })
                },
            );

            #[cfg(feature = "rayon")]
            group.bench_function(
                format!("parallel ({} threads)", rayon::current_num_threads()),
                |b| {
                    b.iter(|| {
                        use par_dfs::sync::par::IntoParallelIterator;
                        use rayon::iter::ParallelIterator;
                        iter.clone().into_par_iter().count()
                    })
                },
            );
        }
    };
}

#[cfg(feature = "async")]
bench_collatz_async!(
    bench_collatz_async_dfs:
    "collatz/async/dfs",
    r#async::Dfs::<CollatzNode>::new(black_box(START), ASYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "async")]
bench_collatz_async!(
    bench_collatz_async_bfs:
    "collatz/async/bfs",
    r#async::Bfs::<CollatzNode>::new(black_box(START), ASYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_fast_bfs:
    "collatz/sync/fastbfs",
    sync::FastBfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_bfs:
    "collatz/sync/bfs",
    sync::Bfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_fast_dfs:
    "collatz/sync/fastdfs",
    sync::FastDfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_dfs:
    "collatz/sync/dfs",
    sync::Dfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_custom_dfs:
    "collatz/sync/customdfs",
    CollatzDfs::new(black_box(START), SYNC_LIMIT, CIRCLES)
);

#[cfg(feature = "async")]
criterion_group!(
    collatz_async,
    bench_collatz_async_bfs,
    bench_collatz_async_dfs,
);

#[cfg(feature = "sync")]
criterion_group!(
    collatz_sync,
    bench_collatz_sync_bfs,
    bench_collatz_sync_fast_bfs,
    bench_collatz_sync_dfs,
    bench_collatz_sync_fast_dfs,
    bench_collatz_sync_custom_dfs
);

fn main() {
    #[cfg(feature = "sync")]
    collatz_sync();
    #[cfg(feature = "async")]
    collatz_async();

    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
