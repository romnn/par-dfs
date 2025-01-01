#[cfg(any(feature = "async", feature = "sync"))]
use criterion::{black_box, criterion_group};
// use std::iter::Iterator;

use collatz_dfs::CollatzNode;

// #[cfg(feature = "sync")]
// mod sync_collatz {
//     use super::CollatzNode;
//     use par_dfs::sync::{ExtendQueue, FastNode, Node, NodeIter};
//
//     impl FastNode for CollatzNode {
//         type Error = std::convert::Infallible;
//
//         #[inline]
//         fn add_children<E>(&self, _depth: usize, queue: &mut E) -> Result<(), Self::Error>
//         where
//             E: ExtendQueue<Self, Self::Error>,
//         {
//             let n = self.0;
//
//             // n can be reached by dividing by two
//             // as long as it doesn't overflow
//             if let Some(even) = n.checked_mul(2) {
//                 queue.add(Ok(Self(even)));
//             }
//
//             // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
//             if n > 4 && n % 6 == 4 {
//                 queue.add(Ok(Self((n - 1) / 3)));
//             }
//             Ok(())
//         }
//     }
//
//     impl Node for CollatzNode {
//         type Error = std::convert::Infallible;
//
//         #[inline]
//         fn children(&self, _depth: usize) -> NodeIter<Self, Self::Error> {
//             Ok(Box::new(self.collatz_children()))
//         }
//     }
// }

#[cfg(feature = "sync")]
const SYNC_LIMIT: Option<usize> = Some(100);
#[cfg(feature = "async")]
const ASYNC_LIMIT: Option<usize> = Some(50);

#[cfg(any(feature = "async", feature = "sync"))]
const ALLOW_CIRCLES: bool = true;

#[cfg(any(feature = "async", feature = "sync"))]
const START: u32 = 1;

#[cfg(any(feature = "async", feature = "sync"))]
fn configure_group<M>(group: &mut criterion::BenchmarkGroup<M>)
where
    M: criterion::measurement::Measurement,
{
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.sampling_mode(criterion::SamplingMode::Flat);
}

#[cfg(feature = "async")]
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

#[cfg(feature = "sync")]
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

            // parallel bridge is just so painfully slow we will not even use it
            // #[cfg(feature = "rayon")]
            // group.bench_function(
            //     format!("parallel-bridge ({} threads)", rayon::current_num_threads()),
            //     |b| {
            //         b.iter(|| {
            //             use rayon::iter::{ParallelBridge, ParallelIterator};
            //             iter.clone().par_bridge().count()
            //         })
            //     },
            // );

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
    par_dfs::r#async::Dfs::<CollatzNode>::new(black_box(START), ASYNC_LIMIT, ALLOW_CIRCLES)
);

#[cfg(feature = "async")]
bench_collatz_async!(
    bench_collatz_async_bfs:
    "collatz/async/bfs",
    par_dfs::r#async::Bfs::<CollatzNode>::new(black_box(START), ASYNC_LIMIT, ALLOW_CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_fast_bfs:
    "collatz/sync/fastbfs",
    par_dfs::sync::FastBfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, ALLOW_CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_bfs:
    "collatz/sync/bfs",
    par_dfs::sync::Bfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, ALLOW_CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_fast_dfs:
    "collatz/sync/fastdfs",
    par_dfs::sync::FastDfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, ALLOW_CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_dfs:
    "collatz/sync/dfs",
    par_dfs::sync::Dfs::<CollatzNode>::new(black_box(START), SYNC_LIMIT, ALLOW_CIRCLES)
);

#[cfg(feature = "sync")]
bench_collatz_sync!(
    bench_collatz_sync_custom_dfs:
    "collatz/sync/customdfs",
    collatz_dfs::CollatzDfs::new(black_box(START), SYNC_LIMIT, ALLOW_CIRCLES)
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
