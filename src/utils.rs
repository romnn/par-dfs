#[cfg(test)]
pub mod test {

    #[allow(unused_macros)]
    macro_rules! assert_eq_sorted {
        ($left:expr, $right:expr $(,)?) => {{
            let mut left = $left.clone();
            let mut right = $right.clone();
            left.sort_unstable();
            right.sort_unstable();
            similar_asserts::assert_eq!(left, right);
        }};
        ($left:expr, $right:expr, $($arg:tt)+) => {{
            let mut left = $left.clone();
            let mut right = $right.clone();
            left.sort_unstable();
            right.sort_unstable();
            similar_asserts::assert_eq!(left, right, $($arg)+);
        }};
    }

    #[allow(unused_imports)]
    pub(crate) use assert_eq_sorted;

    #[derive(thiserror::Error, Hash, PartialEq, Eq, Clone, Debug)]
    #[error("error")]
    pub struct Error;

    #[derive(PartialEq, Eq, Hash, Clone, Debug)]
    pub struct Node(pub usize);

    impl From<usize> for Node {
        fn from(depth: usize) -> Self {
            Self(depth)
        }
    }

    #[cfg(feature = "async")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub mod r#async {
        use crate::r#async::{Node, NodeStream};
        use async_trait::async_trait;
        use futures::{stream, StreamExt};
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        #[async_trait]
        impl Node for super::Node {
            type Error = super::Error;

            async fn children(
                self: Arc<Self>,
                depth: usize,
            ) -> Result<NodeStream<Self, Self::Error>, Self::Error> {
                // we want to test with multiple await points

                sleep(Duration::from_millis(50)).await;
                let nodes = [depth, depth];

                sleep(Duration::from_millis(50)).await;
                let nodes = nodes.into_iter().map(Self).map(Result::Ok);

                sleep(Duration::from_millis(50)).await;
                let stream = stream::iter(nodes);
                Ok(Box::pin(stream.boxed()))
            }
        }
    }

    #[cfg(feature = "sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
    pub mod sync {
        use crate::sync::{ExtendQueue, FastNode, Node, NodeIter};

        impl Node for super::Node {
            type Error = super::Error;

            fn children(&self, depth: usize) -> NodeIter<Self, Self::Error> {
                let nodes = [depth, depth];
                let nodes = nodes.into_iter().map(Self).map(Result::Ok);
                Ok(Box::new(nodes))
            }
        }

        impl FastNode for super::Node {
            type Error = super::Error;

            fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
            where
                E: ExtendQueue<Self, Self::Error>,
            {
                queue.add(Ok(Self(depth)));
                queue.add_all([Ok(Self(depth))]);
                Ok(())
            }
        }
    }

    #[cfg(any(feature = "async", feature = "sync"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "async", feature = "sync"))))]
    pub(crate) fn is_monotonic<I, T>(iter: I, order: std::cmp::Ordering) -> bool
    where
        I: std::iter::IntoIterator<Item = T>,
        <I as std::iter::IntoIterator>::IntoIter: Clone,
        T: std::cmp::Ord,
    {
        let prev = iter.into_iter();
        let next = prev.clone().next();
        prev.zip(next).all(|(prev, next)| {
            let found = next.cmp(&prev);
            found == std::cmp::Ordering::Equal || found == order
        })
    }
}
