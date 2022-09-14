use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkGroup, Criterion, SamplingMode,
};
use par_dfs::sync::*;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::iter::Iterator;
use std::time::Duration;

/// Enumerates the numbers that reach the given starting point when iterating
/// the [Collatz] map, by depth-first search over the [graph] of their orbits.
///
/// [Collatz]: https://en.wikipedia.org/wiki/Collatz_conjecture
/// [graph]: https://en.wikipedia.org/wiki/File:Collatz_orbits_of_the_all_integers_up_to_1000.svg

type Queue = VecDeque<(usize, Result<u32, Infallible>)>;

pub mod custom_dfs {
    use super::*;
    use std::collections::VecDeque;
    use std::convert::Infallible;

    #[derive(Clone, Debug)]
    pub struct CollatzDfs {
        max_depth: Option<usize>,
        queue: Queue,
    }

    impl CollatzDfs {
        pub fn new<D: Into<Option<usize>>>(start: u32, max_depth: D) -> Self {
            Self {
                max_depth: max_depth.into(),
                queue: VecDeque::from_iter([(0, Ok(start))]),
            }
        }
    }

    impl Iterator for CollatzDfs {
        type Item = Result<u32, Infallible>;

        #[inline(always)]
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
                        self.queue.push_back((depth, Ok(even)));
                    }

                    // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
                    if n > 4 && n % 6 == 4 {
                        self.queue.push_back((depth, Ok((n - 1) / 3)));
                    }
                    Some(Ok(n))
                }
                Some((depth, n)) => Some(n),
                None => None,
            }
        }
    }

    #[cfg(feature = "rayon")]
    impl par_dfs::sync::par::SplittableIterator for CollatzDfs {
        fn split(&mut self) -> Option<Self> {
            let len = self.queue.len();
            if len >= 2 {
                let split = self.queue.split_off(len / 2);
                Some(Self {
                    queue: split,
                    max_depth: self.max_depth,
                })
            } else {
                None
            }
        }
    }
}

pub use custom_dfs::*;

#[derive(Clone, Debug)]
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
        children.into_iter().map(|d| Self(d)).map(Result::Ok)
    }
}

pub mod sync {
    use super::*;
    use par_dfs::sync::*;
    use std::convert::Infallible;

    impl FastNode for super::CollatzNode {
        type Error = Infallible;

        #[inline(always)]
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

    impl Node for super::CollatzNode {
        type Error = Infallible;

        #[inline(always)]
        fn children(&self, depth: usize) -> NodeIter<Self, Self::Error> {
            Ok(Box::new(self.collatz_children()))
        }
    }
}

const START: u32 = 1;
const LIMIT: Option<usize> = Some(1_00);

fn configure_group<M>(group: &mut BenchmarkGroup<M>)
where
    M: criterion::measurement::Measurement,
{
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
}

macro_rules! bench_collatz_sync {
    ($name:ident: $group:literal, $iter:expr) => {
        /// Benchmarks for [Collatz] $name.
        fn $name(c: &mut Criterion) {
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

bench_collatz_sync!(
    bench_collatz_sync_fast_bfs:
    "collatz/sync/fastbfs", FastBfs::<CollatzNode>::new(black_box(START), LIMIT)
);

bench_collatz_sync!(
    bench_collatz_sync_bfs:
    "collatz/sync/bfs", Bfs::<CollatzNode>::new(black_box(START), LIMIT)
);

bench_collatz_sync!(
    bench_collatz_sync_fast_dfs:
    "collatz/sync/fastdfs", FastDfs::<CollatzNode>::new(black_box(START), LIMIT)
);

bench_collatz_sync!(
    bench_collatz_sync_dfs:
    "collatz/sync/dfs", Dfs::<CollatzNode>::new(black_box(START), LIMIT)
);

bench_collatz_sync!(
    bench_collatz_sync_custom_dfs:
    "collatz/sync/customdfs", CollatzDfs::new(black_box(START), LIMIT)
);

criterion_group!(
    collatz,
    bench_collatz_sync_bfs,
    bench_collatz_sync_fast_bfs,
    bench_collatz_sync_dfs,
    bench_collatz_sync_fast_dfs,
    bench_collatz_sync_custom_dfs
);
criterion_main!(collatz);
