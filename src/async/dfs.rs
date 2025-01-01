use super::{Node, Stack, StreamQueue};

use futures::stream::{FuturesOrdered, Stream, StreamExt};
use futures::FutureExt;
use pin_project::pin_project;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// Asynchronous depth-first stream for types implementing the [`Node`] trait.
///
/// ### Example
/// ```
/// use futures::StreamExt;
/// use par_dfs::r#async::{Node, Dfs, NodeStream};
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// struct WordNode(String);
///
/// #[async_trait::async_trait]
/// impl Node for WordNode {
///     type Error = std::convert::Infallible;
///
///     async fn children(
///         self: std::sync::Arc<Self>,
///         _depth: usize
///     ) -> Result<NodeStream<Self, Self::Error>, Self::Error> {
///         let len = self.0.len();
///         let nodes: Vec<String> = if len < 2 {
///             vec![]
///         } else {
///             let mid = len/2;
///             vec![self.0[..mid].into(), self.0[mid..].into()]
///         };
///         let nodes = nodes.into_iter()
///             .map(Self)
///             .map(Result::Ok);
///         let stream = futures::stream::iter(nodes);
///         Ok(Box::pin(stream.boxed()))
///     }
/// }
///
/// let result = tokio_test::block_on(async {
///     let root = WordNode("Hello World".into());
///     let dfs = Dfs::<WordNode>::new(root, None, true);
///     let output = dfs
///         .collect::<Vec<_>>()
///         .await
///         .into_iter()
///         .collect::<Result<Vec<_>, _>>()
///         .unwrap();
///     output.into_iter()
///         .filter_map(|s| if s.0.len() == 1 { Some(s.0) } else { None })
///         .collect::<String>()
/// });
/// assert_eq!(result, "Hello World");
/// ```
///
/// [`Node`]: trait@crate::async::Node
#[derive(Default)]
#[pin_project]
pub struct Dfs<N>
where
    N: Node,
{
    stack: Stack<N, N::Error>,
    child_streams_futs: StreamQueue<N, N::Error>,
    max_depth: Option<usize>,
    allow_circles: bool,
    visited: HashSet<N>,
}

impl<N> Dfs<N>
where
    N: Node + Send + Unpin + Clone + 'static,
    N::Error: Send + 'static,
{
    #[inline]
    /// Creates a new [`Dfs`] stream.
    ///
    /// The DFS will be performed from the `root` node up to depth `max_depth`.
    ///
    /// When `allow_circles`, visited nodes will not be tracked, which can lead to cycles.
    ///
    /// [`Dfs`]: struct@crate::async::Dfs
    pub fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let root = root.into();
        let max_depth = max_depth.into();
        let mut child_streams_futs: StreamQueue<N, N::Error> = FuturesOrdered::new();
        let depth = 1;
        let child_stream_fut = Arc::new(root.clone())
            .children(depth)
            .map(move |stream| (depth, stream));
        child_streams_futs.push_front(Box::pin(child_stream_fut));

        Self {
            stack: vec![],
            child_streams_futs,
            max_depth,
            visited: HashSet::from_iter([root]),
            allow_circles,
        }
    }
}

impl<N> Stream for Dfs<N>
where
    N: Node + Send + Clone + Unpin + 'static,
    N::Error: Send + 'static,
{
    type Item = Result<N, N::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // println!("------- poll");
        // println!("stack size: {:?}", this.stack.len());

        // we first poll for the newest child stream in dfs
        // println!("child stream futs: {:?}", this.child_streams_futs.len());
        match this.child_streams_futs.poll_next_unpin(cx) {
            Poll::Ready(Some((depth, stream))) => {
                // println!(
                //     "child stream fut depth {} completed: {:?}",
                //     depth,
                //     stream.is_ok()
                // );
                let stream = match stream {
                    Ok(stream) => stream.boxed(),
                    Err(err) => futures::stream::iter([Err(err)]).boxed(),
                };
                this.stack.push((depth, Box::pin(stream)));
                // println!("stack size: {}", this.stack.len());
            }
            // when there is no child stream future,
            // continue to poll the current stream
            Poll::Ready(None) => {
                // println!("no child stream to wait for");
            }
            // still waiting for the new child stream
            Poll::Pending => {
                // println!("child stream is still pending");
                return Poll::Pending;
            }
        }

        // at this point, the last element in the stack is the current level
        loop {
            let next_item = match this.stack.last_mut() {
                Some((depth, current_stream)) => {
                    let next_item = current_stream.as_mut().poll_next(cx);
                    Some(next_item.map(|node| (depth, node)))
                }
                None => None,
            };

            // println!("next item: {:?}", next_item);
            match next_item {
                // stream item is ready but failure success
                Some(Poll::Ready((_, Some(Err(err))))) => {
                    return Poll::Ready(Some(Err(err)));
                }
                // stream item is ready and success
                Some(Poll::Ready((depth, Some(Ok(node))))) => {
                    if *this.allow_circles || !this.visited.contains(&node) {
                        if !*this.allow_circles {
                            this.visited.insert(node.clone());
                        }

                        if let Some(max_depth) = this.max_depth {
                            if depth >= max_depth {
                                return Poll::Ready(Some(Ok(node)));
                            }
                        }

                        // add child stream future to be polled
                        let arc_node = Arc::new(node.clone());
                        let next_depth = *depth + 1;
                        let child_stream_fut = arc_node
                            .children(next_depth)
                            .map(move |stream| (next_depth, stream));
                        this.child_streams_futs
                            .push_front(Box::pin(child_stream_fut));

                        return Poll::Ready(Some(Ok(node)));
                    }
                }
                // stream completed for this level completed
                Some(Poll::Ready((_, None))) => {
                    this.stack.pop();
                    // println!("pop stack to size: {}", this.stack.len());
                    // try again in the next round
                    // returning Poll::Pending here is bad because the runtime can not know when to poll
                    // us again to make progress since we never passed the cx to poll of the next
                    // level stream
                }
                // stream item is pending
                Some(Poll::Pending) => {
                    return Poll::Pending;
                }
                // stack is empty and we are done
                None => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dfs;
    use anyhow::Result;

    macro_rules! depths {
        ($stream:ident) => {{
            $stream
                // collect the entire stream
                .collect::<Vec<_>>()
                .await
                .into_iter()
                // fail on first error
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                // get depth
                .map(|item| item.0)
                .collect::<Vec<_>>()
        }};
    }

    macro_rules! test_depths_unordered {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[tokio::test(flavor = "multi_thread")]
                async fn [< test_ $name _ unordered >] () -> Result<()> {
                    use tokio::time::{sleep, Duration};
                    use std::cmp::Ordering;
                    use futures::StreamExt;
                    let (iter, expected_depths) = $values;
                    let iter = iter
                        .map(|node| async move {
                            sleep(Duration::from_millis(100)).await;
                            node
                        })
                        .buffer_unordered(8);
                    let depths = depths!(iter);
                    dbg!(&depths);
                    dbg!(&expected_depths);
                    assert!(crate::utils::test::is_monotonic(&depths, Ordering::Greater));
                    crate::utils::test::assert_eq_sorted!(depths, expected_depths);
                    Ok(())
                }
            }
        };
    }

    macro_rules! test_depths_ordered {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[tokio::test(flavor = "multi_thread")]
                async fn [< test_ $name _ ordered >] () -> Result<()> {
                    use tokio::time::{sleep, Duration};
                    use std::cmp::Ordering;
                    use futures::StreamExt;
                    let (iter, expected_depths) = $values;
                    let iter = iter
                        .map(|node| async move {
                            sleep(Duration::from_millis(100)).await;
                            node
                        })
                        .buffered(8);
                    let depths = depths!(iter);
                    assert!(crate::utils::test::is_monotonic(&depths, Ordering::Greater));
                    similar_asserts::assert_eq!(depths, expected_depths);
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
            Dfs::<crate::utils::test::Node>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_ordered,
        test_depths_unordered,
    );

    test_depths!(
        dfs_no_circles:
        (
            Dfs::<crate::utils::test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_ordered,
        test_depths_unordered,
    );
}
