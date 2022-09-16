use super::queue;
use super::{ExtendQueue, FastNode, Node, Queue};
use std::iter::Iterator;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// Synchronous breadth-first iterator for types implementing the [`Node`] trait.
///
/// ### Example
/// ```
/// use par_dfs::sync::{Node, Bfs, NodeIter};
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// struct WordNode(String);
///
/// impl Node for WordNode {
///     type Error = std::convert::Infallible;
///
///     fn children(&self, _depth: usize) -> NodeIter<Self, Self::Error> {
///         let len = self.0.len();
///         let nodes: Vec<String> = if len > 1 {
///             let mid = len/2;
///             vec![self.0[..mid].into(), self.0[mid..].into()]
///         } else {
///             assert!(len == 1);
///             vec![self.0.clone()]
///         };
///         let nodes = nodes.into_iter()
///             .map(Self)
///             .map(Result::Ok);
///         Ok(Box::new(nodes))
///     }
/// }
///
/// let word = "Hello World";
/// let root = WordNode(word.into());
/// let limit = (word.len() as f32).log2().ceil() as usize;
///
/// let bfs = Bfs::<WordNode>::new(root, limit, true);
/// let output = bfs.collect::<Result<Vec<_>, _>>().unwrap();
/// let result = output[output.len()-word.len()..]
///     .into_iter().map(|s| s.0.as_str()).collect::<String>();
/// assert_eq!(result, "Hello World");
/// ```
///
/// [`Node`]: trait@crate::sync::Node
pub struct Bfs<N>
where
    N: Node,
{
    queue: queue::Queue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> Bfs<N>
where
    N: Node,
{
    #[inline]
    /// Creates a new [`Bfs`] iterator.
    ///
    /// The BFS will be performed from the `root` node up to depth `max_depth`.
    ///
    /// When `allow_circles`, visited nodes will not be tracked, which can lead to cycles.
    ///
    /// [`Bfs`]: struct@crate::sync::Bfs
    pub fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = queue::Queue::new(allow_circles);
        let root = root.into();
        let max_depth = max_depth.into();

        let depth = 1;
        match root.children(depth) {
            Ok(children) => queue.add_all(depth, children),
            Err(err) => queue.add(0, Err(err)),
        }

        Self { queue, max_depth }
    }
}

impl<N> Iterator for Bfs<N>
where
    N: Node,
{
    type Item = Result<N, N::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_front() {
            // next node failed
            Some((_, Err(err))) => Some(Err(err)),
            // next node succeeded
            Some((depth, Ok(node))) => {
                if let Some(max_depth) = self.max_depth {
                    if depth >= max_depth {
                        return Some(Ok(node));
                    }
                }
                match node.children(depth + 1) {
                    Ok(children) => {
                        self.queue.add_all(depth + 1, children);
                    }
                    Err(err) => self.queue.add(depth + 1, Err(err)),
                };
                Some(Ok(node))
            }
            // no next node
            None => None,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// Synchronous, fast breadth-first iterator for types implementing the [`FastNode`] trait.
///
/// ### Example
/// ```
/// use par_dfs::sync::{FastNode, FastBfs, ExtendQueue, NodeIter};
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// struct WordNode(String);
///
/// impl FastNode for WordNode {
///     type Error = std::convert::Infallible;
///
///     fn add_children<E>(
///         &self, _depth: usize, queue: &mut E
///     ) -> Result<(), Self::Error>
///     where
///         E: ExtendQueue<Self, Self::Error>,
///     {
///         let len = self.0.len();
///         if len > 1 {
///             let mid = len/2;
///             queue.add(Ok(Self(self.0[..mid].into())));
///             queue.add(Ok(Self(self.0[mid..].into())));
///         } else {
///             assert!(len == 1);
///             queue.add(Ok(Self(self.0.clone())));
///         }
///         Ok(())
///     }
/// }
///
/// let word = "Hello World";
/// let root = WordNode(word.into());
/// let limit = (word.len() as f32).log2().ceil() as usize;
///
/// let bfs = FastBfs::<WordNode>::new(root, limit, true);
/// let output = bfs.collect::<Result<Vec<_>, _>>().unwrap();
/// let result = output[output.len()-word.len()..]
///     .into_iter().map(|s| s.0.as_str()).collect::<String>();
/// assert_eq!(result, "Hello World");
/// ```
///
/// [`FastNode`]: trait@crate::sync::FastNode
pub struct FastBfs<N>
where
    N: FastNode,
{
    queue: queue::Queue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> FastBfs<N>
where
    N: FastNode,
{
    #[inline]
    /// Creates a new [`FastBfs`] iterator.
    ///
    /// The BFS will be performed from the `root` node up to depth `max_depth`.
    ///
    /// When `allow_circles`, visited nodes will not be tracked, which can lead to cycles.
    ///
    /// [`FastBfs`]: struct@crate::sync::FastBfs
    pub fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = queue::Queue::new(allow_circles);
        let root: N = root.into();
        let max_depth = max_depth.into();
        let depth = 1;
        let mut depth_queue = queue::QueueWrapper::new(depth, &mut queue);
        if let Err(err) = root.add_children(depth, &mut depth_queue) {
            depth_queue.add(Err(err));
        }
        Self { queue, max_depth }
    }
}

impl<N> Iterator for FastBfs<N>
where
    N: FastNode,
{
    type Item = Result<N, N::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_front() {
            // next node failed
            Some((_, Err(err))) => Some(Err(err)),
            // next node succeeded
            Some((depth, Ok(node))) => {
                if let Some(max_depth) = self.max_depth {
                    if depth >= max_depth {
                        return Some(Ok(node));
                    }
                }
                let next_depth = depth + 1;
                let mut depth_queue = queue::QueueWrapper::new(next_depth, &mut self.queue);
                if let Err(err) = node.add_children(next_depth, &mut depth_queue) {
                    depth_queue.add(Err(err));
                }
                Some(Ok(node))
            }
            // no next node
            None => None,
        }
    }
}

#[cfg(feature = "rayon")]
#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
mod par {
    use crate::sync::par::parallel_iterator;
    use crate::sync::{Bfs, FastBfs, FastNode, Node};

    parallel_iterator!(Bfs<Node>);
    parallel_iterator!(FastBfs<FastNode>);
}

#[cfg(feature = "rayon")]
pub use par::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use std::cmp::Ordering;

    #[cfg(feature = "rayon")]
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    macro_rules! depths {
        ($iter:ident) => {{
            $iter
                // fail on first error
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                // get depth
                .map(|item| item.0)
                .collect::<Vec<_>>()
        }};
    }

    macro_rules! test_depths_serial {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[test]
                fn [< test_ $name _ serial >] () -> Result<()> {
                    let (iter, expected_depths) = $values;
                    let depths = depths!(iter);
                    assert!(test::is_monotonic(&depths, Ordering::Greater));
                    assert_eq!(depths, expected_depths);
                    Ok(())
                }
            }
        };
    }

    macro_rules! test_depths_parallel {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[cfg(feature = "rayon")]
                #[test]
                fn [< test_ $name _ parallel >] () -> Result<()> {
                    let (iter, expected_depths) = $values;
                    let iter = iter.into_par_iter();
                    let depths = depths!(iter);
                    test::assert_eq_vec!(depths, expected_depths);
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
            [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_bfs:
        (
            FastBfs::<test::Node>::new(0, 3, true),
            [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_bfs_no_circles:
        (
            FastBfs::<test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
    );

    test_depths!(
        bfs_no_circles:
        (
            Bfs::<test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );
}
