pub mod bfs;
pub mod dfs;

pub use bfs::*;
pub use dfs::*;

use async_trait::async_trait;
use futures::stream::{FuturesOrdered, Stream};
use futures::Future;
use std::hash::Hash;
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
    Self: Sized + Hash + Eq + std::fmt::Debug,
{
    type Error: Hash + Eq + std::fmt::Debug;

    async fn children(
        self: Arc<Self>,
        depth: usize,
    ) -> Result<NodeStream<Self, Self::Error>, Self::Error>;
}
