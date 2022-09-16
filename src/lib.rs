//! Parallel, serial, and async DFS and BFS traversal iterators.

// #[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
#[cfg(feature = "sync")]
pub mod sync;

// #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub mod r#async;

mod utils;
