use super::queue;
use super::{ExtendQueue, FastNode, Node, Queue};
use std::iter::Iterator;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// Synchronous depth-first iterator for types implementing the [`Node`] trait.
///
/// [`Node`]: trait@crate::sync::Node
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
        if let Err(err) = root.add_children(1, &mut queue) {
            queue.add(0, Err(err));
        }
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
                if let Err(err) = node.add_children(depth + 1, &mut self.queue) {
                    self.queue.add(depth + 1, Err(err));
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

#[cfg(feature = "rayon")]
pub use par::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

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
        dfs:
        (
            Dfs::<test::Node>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_dfs:
        (
            FastDfs::<test::Node>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_dfs_no_circles:
        (
            FastDfs::<test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
    );

    test_depths!(
        dfs_no_circles:
        (
            Dfs::<test::Node>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
    );
}
