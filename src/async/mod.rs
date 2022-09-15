// pub mod bfs;
pub mod dfs;
// pub mod queue;

// pub use bfs::*;
pub use dfs::*;

use async_trait::async_trait;
use futures::stream::{FuturesOrdered, Stream, StreamExt};
use futures::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type NodeStream<N, E> = Pin<Box<dyn Stream<Item = Result<N, E>> + Unpin + Send + 'static>>;

type Stack<N, E> = Vec<(usize, NodeStream<N, E>)>;

type NewNodesFut<N, E> =
    Pin<Box<dyn Future<Output = (usize, Result<NodeStream<N, E>, E>)> + Unpin + Send + 'static>>;

type StreamQueue<N, E> = FuturesOrdered<NewNodesFut<N, E>>;

#[async_trait]
pub trait Node
where
    Self: Sized + std::fmt::Debug,
{
    type Error: std::fmt::Debug;

    async fn children(
        self: Arc<Self>,
        // &self,
        depth: usize,
    ) -> Result<NodeStream<Self, Self::Error>, Self::Error>;
}
