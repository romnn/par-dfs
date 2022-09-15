use super::{NewNodesFut, Node, NodeStream, Stack, StreamQueue};

use futures::stream::{FuturesOrdered, Stream, StreamExt};
use futures::{Future, FutureExt};
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub trait NodeFut<N, E>:
    Future<Output = Result<NodeStream<N, E>, E>> + Unpin + Send + 'static
{
}

struct IsUnpin<T>(T)
where
    T: Unpin;

#[derive(Default)]
#[pin_project]
pub struct Bfs<N>
where
    N: Node,
{
    // stack: Stack<N, N::Error>,
    #[pin]
    current_stream: Option<(usize, NodeStream<N, N::Error>)>,
    child_streams_futs: StreamQueue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> Bfs<N>
where
    N: Node + Send + Unpin + Clone + 'static,
    N::Error: Send + 'static,
{
    #[inline]
    pub fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let root = root.into();
        let max_depth = max_depth.into();
        let mut child_streams_futs: StreamQueue<N, N::Error> = FuturesOrdered::new();
        let depth = 1;
        let child_stream_fut = Arc::new(root)
            .children(depth)
            .map(move |stream| (depth, stream));
        child_streams_futs.push_back(Box::pin(child_stream_fut));

        Self {
            // stack: vec![],
            current_stream: None,
            child_streams_futs,
            max_depth,
        }
    }
}

impl<N> Stream for Bfs<N>
where
    N: Node + Send + Clone + Unpin + 'static,
    N::Error: Send + 'static,
{
    type Item = Result<N, N::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        println!("------- poll");
        println!("has current stream: {:?}", this.current_stream.is_some());

        loop {
            let mut current_stream = this.current_stream.as_mut().as_pin_mut();
            let next_item = match current_stream.as_deref_mut() {
                Some((depth, stream)) => {
                    let next_item = stream.as_mut().poll_next(cx);
                    Some(next_item.map(|node| (depth, node)))
                }
                None => None,
            };

            println!("next item: {:?}", next_item);
            match next_item {
                // stream item is ready but failure success
                Some(Poll::Ready((depth, Some(Err(err))))) => {
                    return Poll::Ready(Some(Err(err)));
                }
                // stream item is ready and success
                Some(Poll::Ready((depth, Some(Ok(node))))) => {
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
                        .push_back(Box::pin(child_stream_fut));

                    return Poll::Ready(Some(Ok(node)));
                }
                // stream item is pending
                Some(Poll::Pending) => {
                    return Poll::Pending;
                }
                // no current stream or completed
                Some(Poll::Ready((_, None))) | None => {
                    // proceed to poll the next stream
                }
            }

            // poll the next stream
            println!("child stream futs: {:?}", this.child_streams_futs.len());
            match this.child_streams_futs.poll_next_unpin(cx) {
                Poll::Ready(Some((depth, stream))) => {
                    println!(
                        "child stream fut depth {} completed: {:?}",
                        depth,
                        stream.is_ok()
                    );
                    let stream = match stream {
                        Ok(stream) => stream.boxed(),
                        Err(err) => futures::stream::iter([Err(err)]).boxed(),
                    };
                    this.current_stream.set(Some((depth, Box::pin(stream))));
                }
                // when there are no more child stream futures,
                // we are done
                Poll::Ready(None) => {
                    println!("no more child streams");
                    return Poll::Ready(None);
                }
                // still waiting for the next stream
                Poll::Pending => {
                    println!("child stream is still pending");
                    return Poll::Pending;
                }
            }
        }

        // we first poll for the newest child stream in Bfs

        // at this point, the last element in the stack is the current level
        // loop {
        //     let next_item = match this.stack.last_mut() {
        //         Some((depth, current_stream)) => {
        //             let next_item = current_stream.as_mut().poll_next(cx);
        //             Some(next_item.map(|node| (depth, node)))
        //         }
        //         None => None,
        //     };

        //     println!("next item: {:?}", next_item);
        //     match next_item {
        //         // stream item is ready but failure success
        //         Some(Poll::Ready((depth, Some(Err(err))))) => {
        //             return Poll::Ready(Some(Err(err)));
        //         }
        //         // stream item is ready and success
        //         Some(Poll::Ready((depth, Some(Ok(node))))) => {
        //             if let Some(max_depth) = this.max_depth {
        //                 if depth >= max_depth {
        //                     return Poll::Ready(Some(Ok(node)));
        //                 }
        //             }

        //             // add child stream future to be polled
        //             let arc_node = Arc::new(node.clone());
        //             let next_depth = *depth + 1;
        //             let child_stream_fut = arc_node
        //                 .children(next_depth)
        //                 .map(move |stream| (next_depth, stream));
        //             this.child_streams_futs
        //                 .push_back(Box::pin(child_stream_fut));

        //             return Poll::Ready(Some(Ok(node)));
        //         }
        //         // stream completed for this level completed
        //         Some(Poll::Ready((depth, None))) => {
        //             let _ = this.stack.pop();
        //             println!("pop stack to size: {}", this.stack.len());
        //             // try again in the next round
        //             // returning Poll::Pending here is bad because the runtime can not know when to poll
        //             // us again to make progress since we never passed the cx to poll of the next
        //             // level stream
        //         }
        //         // stream item is pending
        //         Some(Poll::Pending) => {
        //             return Poll::Pending;
        //         }
        //         // stack is empty and we are done
        //         None => {
        //             return Poll::Ready(None);
        //         }
        //     }
        // }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;
    use anyhow::Result;
    use futures::{Stream, StreamExt};
    use pretty_assertions::assert_eq;
    use std::cmp::Ordering;
    use tokio::time::{sleep, Duration};

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
                    let (iter, expected_depths) = $values;
                    let iter = iter
                        .map(|node| async move {
                            sleep(Duration::from_millis(100)).await;
                            node
                        })
                        .buffer_unordered(8);
                    let depths = depths!(iter);
                    assert!(test::is_monotonic(&depths, Ordering::Greater));
                    test::assert_eq_vec!(depths, expected_depths);
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
                    let (iter, expected_depths) = $values;
                    let iter = iter
                        .map(|node| async move {
                            sleep(Duration::from_millis(100)).await;
                            node
                        })
                        .buffered(8);
                    let depths = depths!(iter);
                    assert!(test::is_monotonic(&depths, Ordering::Greater));
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
        bfs:
        (
            Bfs::<test::Node>::new(0, 3, true),
            // [1, 1, 2, 2, 2, 2]
            [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3]
        ),
        test_depths_ordered,
        test_depths_unordered,
    );
}
