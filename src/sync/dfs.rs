use super::queue;
use super::{ExtendQueue, FastNode, Node, Queue};
use std::iter::Iterator;

/// Synchronous depth-first iterator for types implementing the [`Node`] trait.
///
/// ### Example
/// ```
/// use par_dfs::sync::{Node, Dfs, NodeIter};
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// struct WordNode(String);
///
/// impl Node for WordNode {
///     type Error = std::convert::Infallible;
///
///     fn children(&self, _depth: usize) -> NodeIter<Self, Self::Error> {
///         let len = self.0.len();
///         let nodes: Vec<String> = if len < 2 {
///             vec![]
///         } else {
///             let mid = len/2;
///             vec![self.0[mid..].into(), self.0[..mid].into()]
///         };
///         let nodes = nodes.into_iter()
///             .map(Self)
///             .map(Result::Ok);
///         Ok(Box::new(nodes))
///     }
/// }
///
/// let root = WordNode("Hello World".into());
/// let dfs = Dfs::<WordNode>::new(root, None, true);
/// let output = dfs.collect::<Result<Vec<_>, _>>().unwrap();
/// let result = output.into_iter()
///     .filter_map(|s| if s.0.len() == 1 { Some(s.0) } else { None })
///     .collect::<String>();
/// assert_eq!(result, "Hello World");
/// ```
///
/// [`Node`]: trait@crate::sync::Node
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct Dfs<N>
where
    N: Node,
{
    queue: queue::Queue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> Dfs<N>
where
    N: Node,
{
    #[inline]
    /// Creates a new [`Dfs`] iterator.
    ///
    /// The DFS will be performed from the `root` node up to depth `max_depth`.
    ///
    /// When `allow_circles`, visited nodes will not be tracked, which can lead to cycles.
    ///
    /// [`Dfs`]: struct@crate::sync::Dfs
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
            Err(err) => queue.add(depth, Err(err)),
        }
        Self { queue, max_depth }
    }
}

impl<N> Iterator for Dfs<N>
where
    N: Node,
{
    type Item = Result<N, N::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_back() {
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
/// Synchronous, fast depth-first iterator for types implementing the [`FastNode`] trait.
///
/// ### Example
/// ```
/// use par_dfs::sync::{FastNode, FastDfs, ExtendQueue, NodeIter};
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
///             queue.add(Ok(Self(self.0[mid..].into())));
///             queue.add(Ok(Self(self.0[..mid].into())));
///         }
///         Ok(())
///     }
/// }
///
/// let root = WordNode("Hello World".into());
/// let dfs = FastDfs::<WordNode>::new(root, None, true);
/// let output = dfs.collect::<Result<Vec<_>, _>>().unwrap();
/// let result = output.into_iter()
///     .filter_map(|s| if s.0.len() == 1 { Some(s.0) } else { None })
///     .collect::<String>();
/// assert_eq!(result, "Hello World");
/// ```
///
/// [`FastNode`]: trait@crate::sync::FastNode
pub struct FastDfs<N>
where
    N: FastNode,
{
    queue: queue::Queue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> FastDfs<N>
where
    N: FastNode,
{
    #[inline]
    /// Creates a new [`FastDfs`] iterator.
    ///
    /// The DFS will be performed from the `root` node up to depth `max_depth`.
    ///
    /// When `allow_circles`, visited nodes will not be tracked, which can lead to cycles.
    ///
    /// [`FastDfs`]: struct@crate::sync::FastDfs
    pub fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = queue::Queue::new(allow_circles);
        let root: N = root.into();
        let max_depth = max_depth.into();
        let mut depth_queue = queue::QueueWrapper::new(0, &mut queue);
        depth_queue.add(Ok(root));
        Self { queue, max_depth }
    }
}

impl<N> Iterator for FastDfs<N>
where
    N: FastNode,
{
    type Item = Result<N, N::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_back() {
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
    use crate::sync::{Dfs, FastDfs, FastNode, Node};

    parallel_iterator!(Dfs<Node>);
    parallel_iterator!(FastDfs<FastNode>);
}

#[cfg(test)]
mod tests {
    use super::{Dfs, FastDfs};
    use anyhow::Result;

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
                    similar_asserts::assert_eq!(depths, expected_depths);
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
                    crate::utils::test::assert_eq_sorted!(depths, expected_depths);
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
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_dfs:
        (
            FastDfs::<crate::utils::test::Node>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_dfs_no_circles:
        (
            FastDfs::<crate::utils::test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
    );

    test_depths!(
        dfs_no_circles:
        (
            Dfs::<crate::utils::test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );
}
