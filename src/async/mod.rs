pub mod bfs;
pub mod dfs;

pub use bfs::Bfs;
pub use dfs::Dfs;

use async_trait::async_trait;
use futures::stream::{FuturesOrdered, Stream};
use futures::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

type Stack<N, E> = Vec<(usize, NodeStream<N, E>)>;

type NewNodesFut<N, E> =
    Pin<Box<dyn Future<Output = (usize, Result<NodeStream<N, E>, E>)> + Unpin + Send + 'static>>;

type StreamQueue<N, E> = FuturesOrdered<NewNodesFut<N, E>>;

/// A pinned [`Stream`] of [`Node`]s
///
/// [`Stream`]: trait@futures::stream::Stream
/// [`Node`]: trait@crate::async::Node
pub type NodeStream<N, E> = Pin<Box<dyn Stream<Item = Result<N, E>> + Unpin + Send>>;

#[async_trait]
/// A node which produces a [`Stream`] of children [`Node`]s for a given depth.
///
/// [`Stream`]: trait@futures::stream::Stream
/// [`Node`]: trait@crate::async::Node
pub trait Node
where
    Self: Sized + Hash + Eq + std::fmt::Debug,
{
    /// The type of the error when creating the stream fails.
    type Error: std::fmt::Debug;

    /// Returns a [`NodeStream`] of its children.
    ///
    /// # Errors
    ///
    /// Should return [`Self::Error`] if the stream can not be created.
    ///
    /// [`NodeStream`]: type@crate::async::NodeStream
    /// [`Self::Error`]: type@crate::async::Node::Error
    async fn children(
        self: Arc<Self>,
        depth: usize,
    ) -> Result<NodeStream<Self, Self::Error>, Self::Error>;
}
