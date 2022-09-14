// pub mod bfs;
pub mod dfs;
// pub mod queue;

// pub use bfs::*;
pub use dfs::*;

use async_trait::async_trait;
use futures::{Future, Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;

pub type NodeStream<N, E> = Pin<Box<dyn Stream<Item = Result<N, E>> + Unpin + Send + 'static>>;

// type StaticNodeStream<E> = Pin<Box<dyn Stream<Item = Result<N, E>> + Unpin + Send + 'static>>;

#[async_trait]
pub trait Node
where
    Self: Sized + std::fmt::Debug,
{
    type Error: std::fmt::Debug;

    async fn children(
        self: Arc<Self>,
        depth: usize,
    ) -> Result<NodeStream<Self, Self::Error>, Self::Error>;
}

type Stack<N, E> = Vec<(usize, NodeStream<N, E>)>;

type NewNodesFut<N, E> =
    Pin<Box<dyn Future<Output = Result<NodeStream<N, E>, E>> + Unpin + Send + 'static>>;
