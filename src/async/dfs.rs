use super::{NewNodesFut, Node, Stack};

use futures::Stream;
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

#[derive(Default)]
#[pin_project]
// pub struct Dfs<N, E>
pub struct Dfs<N>
where
    N: Node, // <Error = E>,
{
    #[pin]
    root: N,
    #[pin]
    stack: Stack<N, N::Error>,
    #[pin]
    new_nodes_stream_fut: Option<(usize, NewNodesFut<N, N::Error>)>,
    max_depth: Option<usize>,
}

// impl<N, E> Dfs<N, E>
impl<N> Dfs<N>
where
    N: Node + Send + Unpin + Clone + 'static,
    // N: Node<Error = E> + Send + Unpin + Clone + 'static,
    N::Error: Send + 'static,
{
    #[inline]
    pub async fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let root = root.into();
        let max_depth = max_depth.into();
        let new_nodes_stream_fut = Box::pin(Arc::new(root.clone()).children(1));
        Self {
            root,
            stack: vec![],
            new_nodes_stream_fut: Some((0, new_nodes_stream_fut)),
            max_depth,
        }
    }
}

// impl<N, E> Stream for Dfs<N, E>
impl<N> Stream for Dfs<N>
where
    // N: Node<Error = E> + Send + Clone + Unpin + 'static,
    N: Node + Send + Clone + Unpin + 'static,
    N::Error: Send + 'static,
{
    type Item = Result<N, N::Error>;
    // type Item = Result<(usize, Result<N, N::Error>), N::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;
    use anyhow::Result;
    use futures::{Stream, StreamExt};
    use pretty_assertions::assert_eq;
    use tokio::time::{sleep, Duration};

    // #[futures_test::test]
    // async fn test_dfs() -> Result<()> {
    //     let dfs = Dfs::<test::Node>::new(1, 3).await;
    //     Ok(())
    // }

    macro_rules! test_depths_ordered {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[tokio::test(flavor = "multi_thread")]
                async fn [< test_ $name _ ordered >] () -> Result<()> {
                    let (iter_fut, expected_depths) = $values;
                    let iter = iter_fut.await;
                    let output = iter
                        .map(|node| async move {
                            sleep(Duration::from_millis(100)).await;
                            node
                        })
                        .buffered(8)
                        .collect::<Vec<_>>()
                        .await
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()?;
                    let depths: Vec<_> = output.into_iter()
                        .map(|item| item.0).collect();
                    assert_eq!(depths, expected_depths);
                    Ok(())
                }
            }
        };
    }

    macro_rules! test_depths {
        ($name:ident: $values:expr, $($macro:ident,)*) => {
            $(
                $macro!($name: $values);
            )*
        }
    }

    test_depths!(
        dfs:
        (
            Dfs::<test::Node>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_ordered,
        // test_depths_unordered,
    );
}
