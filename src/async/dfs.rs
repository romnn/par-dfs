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
// pub struct Dfs<N, Fut>
pub struct Dfs<N>
where
    N: Node,
    // Fut: NodeFut<N, N::Error>,
{
    // #[pin]
    // root: N,
    // #[pin]
    // stack: Stack<N, N::Error>,
    // #[pin]
    // pending_child_streams: FuturesOrdered<Fut>,
    // pending_child_streams: FuturesOrdered<NewNodesFut<N, N::Error>>,
    #[pin]
    stack: Stack<N, N::Error>,
    #[pin]
    child_streams_futs: StreamQueue<N, N::Error>,
    // queue: FuturesOrdered<(usize, N)>,
    // new_nodes_stream_fut: Option<(usize, NewNodesFut<N, N::Error>)>,
    max_depth: Option<usize>,
}

// impl<N, Fut> Dfs<N, Fut>
impl<N> Dfs<N>
where
    N: Node + Send + Unpin + Clone + 'static,
    N::Error: Send + 'static,
    // Fut: NodeFut<N, N::Error>,
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
        child_streams_futs.push(Box::pin(child_stream_fut));
        Self {
            stack: vec![],
            child_streams_futs,
            // queue: FuturesOrdered::new(),
            max_depth,
        }
    }
}

// impl<N, Fut> Stream for Dfs<N, Fut>
impl<N> Stream for Dfs<N>
where
    N: Node + Send + Clone + Unpin + 'static,
    N::Error: Send + 'static,
    // Fut: NodeFut<N, N::Error>,
{
    type Item = Result<N, N::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        println!("------- poll");
        println!("stack size: {:?}", this.stack.len());

        // we first poll for the newest child stream in dfs
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
                this.stack.push((depth, Box::pin(stream)));
                println!("stack size: {}", this.stack.len());
            }
            // when there are no more child streams left,
            // continue to consume the stream
            Poll::Ready(None) => {
                println!("no more child streams to wait for");
            }
            // still waiting for the new child stream
            Poll::Pending => {
                println!("child stream is still pending");
                return Poll::Pending;
            }
        }

        // if this.stack.is_empty() {
        //     match this.child_streams_futs.poll_next_unpin(cx) {
        //         Poll::Ready(Some((depth, stream))) => {
        //             // println!(
        //             //     "first child stream fut depth {} completed: {:?}",
        //             //     depth,
        //             //     stream.is_ok()
        //             // );
        //             let stream = match stream {
        //                 Ok(stream) => stream.boxed(),
        //                 Err(err) => futures::stream::iter([Err(err)]).boxed(),
        //             };
        //             this.stack.push((depth, Box::pin(stream)));
        //             // println!("stack size: {}", this.stack.len());
        //         }
        //         _ => {}
        //     }
        // }

        loop {
            let next_item = match this.stack.last_mut() {
                Some((depth, current_stream)) => {
                    // let depth = depth.clone();
                    // let mut current: &mut Pin<Box<_>> = current;
                    // let mut current: Pin<&mut _> = current.as_mut();
                    // futures::ready!(stream.poll_next(cx)).map(|node| (depth, node))
                    let next_item = current_stream.as_mut().poll_next(cx);
                    Some(next_item.map(|node| (depth, node)))

                    // match current_stream.as_mut().poll_next(cx) {
                    //     Poll::Ready(node) => Some(Poll::Ready(node.map(|node| (depth, node)))),
                    //     Poll::Pending => Some(Poll::Pending),
                    //     // Poll::Ready(Some(Err(err))) => {
                    //     //     return Poll::Ready(Some(Err(err)));
                    //     // }
                    //     // Poll::Ready(Some(Ok(node))) => {
                    //     //     if let Some(max_depth) = this.max_depth {
                    //     //         if depth >= *max_depth {
                    //     //             return Poll::Ready(Some(Ok((depth, Ok(node)))));
                    //     //         }
                    //     //     }
                    //     //     println!("node: {:?}", node);

                    //     //     // add child stream future to be polled
                    //     //     let child_stream_fut = Arc::new(node.clone()).children(depth + 1);
                    //     //     this.child_streams_futs.push(Box::pin(child_stream_fut));

                    //     //     return Poll::Ready(Some(Ok((depth, Ok(node)))));
                    //     // }
                    //     // // Poll::Ready(Some(fut)) => this.in_progress_queue.push_back(fut),
                    //     // Poll::Ready(None) | Poll::Pending => break,
                    // }
                }
                None => None,
            };

            // dbg!(&this.stack.len());
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
                            // return Poll::Ready(Some(Ok((depth, Ok(node)))));
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
                // stream completed for this level completed
                Some(Poll::Ready((depth, None))) => {
                    let _ = this.stack.pop();
                    println!("pop stack to size: {}", this.stack.len());
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

        // // at this point, we can poll the new child futures
        // println!("child stream futs: {:?}", this.child_streams_futs.len());
        // match this.child_streams_futs.poll_next_unpin(cx) {
        //     // match Pin::new(&mut this.child_streams_futs).poll_next(cx) {
        //     Poll::Ready(Some((depth, stream))) => {
        //         println!(
        //             "child stream fut depth {} completed: {:?}",
        //             depth,
        //             stream.is_ok()
        //         );
        //         let stream = match stream {
        //             Ok(stream) => stream.boxed(),
        //             Err(err) => futures::stream::iter([Err(err)]).boxed(),
        //         };
        //         // this.stack.push((*depth + 1, Box::pin(new_nodes_stream)));
        //         this.stack.push((depth, Box::pin(stream)));
        //         println!("stack size: {}", this.stack.len());
        //         Poll::Pending
        //     }
        //     // Poll::Ready(Some(Err(err))) => {}
        //     // Poll::Ready(Some(Ok(stream))) => {
        //     // }
        //     // when there is no child streams left, we are done
        //     Poll::Ready(None) => {
        //         println!("no more child streams to wait for");
        //         Poll::Ready(None)
        //     },
        //     // still waiting for the new child streams
        //     Poll::Pending => {
        //         println!("child streams is still pending");
        //         Poll::Pending
        //     },
        // }

        // take from queue as long as possible
        // match this.queue.poll_next_unpin(cx) {
        //     Poll::Ready(Some((depth, Err(err)))) => {
        //         return Poll::Ready(Some(Err(err)));
        //     }
        //     Poll::Ready(Some((depth, Ok(node)))) => {
        //         if let Some(max_depth) = this.max_depth {
        //             if depth >= *max_depth {
        //                 return Poll::Ready(Some(Ok((depth, Ok(node)))));
        //             }
        //         }
        //         println!("node: {:?}", node);

        //         // add child stream future to be polled
        //         let child_stream_fut = Arc::new(node.clone()).children(depth + 1);
        //         this.child_streams_futs.push(Box::pin(child_stream_fut));

        //         return Poll::Ready(Some(Ok((depth, Ok(node)))));
        //     }
        //     // Poll::Ready(Some(fut)) => this.in_progress_queue.push_back(fut),
        //     Poll::Ready(None) | Poll::Pending => break,
        // }

        // queue is empty: fill with a few children the current child stream

        // Poll::Ready(None)
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

    macro_rules! test_depths_ordered {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[tokio::test(flavor = "multi_thread")]
                async fn [< test_ $name _ ordered >] () -> Result<()> {
                    let (iter, expected_depths) = $values;
                    // let iter = iter.await;
                    let output = iter
                        // .map(|node| async move {
                        //     sleep(Duration::from_millis(100)).await;
                        //     node
                        // })
                        // .buffered(8)
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
            // Dfs::<test::Node, _>::new(0, 3, true),
            Dfs::<test::Node>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_ordered,
        // test_depths_unordered,
    );
}
