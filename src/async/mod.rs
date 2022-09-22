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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use futures::StreamExt;
    use pretty_assertions::assert_eq;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_streams_iter_is_clonable() -> Result<()> {
        let stream = futures::stream::iter([1, 2, 3]);
        let s1: Vec<_> = stream.clone().collect().await;
        let s2: Vec<_> = stream.clone().collect().await;
        assert_eq!(s1.as_slice(), [1, 2, 3]);
        assert_eq!(s2.as_slice(), [1, 2, 3]);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_streams_map_is_not_clonable() -> Result<()> {
        let stream = futures::stream::iter([1, 2, 3]);
        let _stream = stream.map(|i| async move { i * 2 });
        // this does not work!
        // cannot clone any complex streams!
        // let s1: Vec<_> = stream.clone().collect().await;
        // let s2: Vec<_> = stream.clone().collect().await;
        // assert_eq!(s1.as_slice(), [1, 4, 6]);
        // assert_eq!(s2.as_slice(), [1, 4, 6]);
        Ok(())
    }
}
