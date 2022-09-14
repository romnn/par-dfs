use super::queue;
use super::{ExtendQueue, FastNode, Node, Queue};
use std::iter::Iterator;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
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

    #[inline(always)]
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
    #[inline(always)]
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

impl<N> Iterator for FastBfs<N>
where
    N: FastNode,
{
    type Item = Result<N, N::Error>;

    #[inline(always)]
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
pub mod par {
    use crate::sync::par::*;
    use crate::sync::*;

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

    macro_rules! test_depths_serial {
        ($name:ident: $values:expr) => {
            paste::item! {
                #[test]
                fn [< test_ $name _ serial >] () -> Result<()> {
                    let (iter, expected_depths) = $values;
                    let output = iter.collect::<Result<Vec<_>, _>>()?;
                    let depths: Vec<_> = output.into_iter()
                        .map(|item| item.0).collect();
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
                    let output = iter.into_par_iter()
                        .collect::<Result<Vec<_>, _>>()?;
                    let depths: Vec<_> = output.into_iter()
                        .map(|item| item.0).collect();
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
    );
}
