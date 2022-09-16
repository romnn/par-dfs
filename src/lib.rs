//! Parallel, serial, and async DFS and BFS traversal iterators.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "sync")]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
pub mod sync;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod r#async;

mod utils;
