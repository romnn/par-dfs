use super::queue;
// use super::{ExtendQueue, FastNode, Node, Queue as QueueTrait};
use super::*;
use std::iter::Iterator;

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Dfs<N>
where
    N: Node,
{
    queue: queue::Queue<N, N::Error>,
    max_depth: Option<usize>,
    // allow_circles: bool,
}

impl<N> Dfs<N>
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
        // let visited = HashSet::new();
        let root = root.into();
        let max_depth = max_depth.into();
        let depth = 1;
        match root.children(depth) {
            Ok(children) => queue.add_all(depth, children),
            Err(err) => queue.add(depth, Err(err)),
        }
        Self {
            queue,
            // visited,
            max_depth,
            // allow_circles,
        }
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
            Some((depth, Err(err))) => Some(Err(err)),
            // next node succeeded
            Some((depth, Ok(node))) => {
                let add_children = self
                    .max_depth
                    .map(|max_depth| depth < max_depth)
                    .unwrap_or(true);

                if add_children {
                    match node.children(depth + 1) {
                        Ok(children) => {
                            self.queue.add_all(depth + 1, children);
                        }
                        Err(err) => self.queue.add(depth + 1, Err(err)),
                    }
                }
                Some(Ok(node))
            }
            // no next node
            None => None,
        }
    }
}

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct FastDfs<N>
where
    N: FastNode,
{
    queue: queue::Queue<N, N::Error>,
    // visited: HashSet<N>,
    max_depth: Option<usize>,
    // allow_circles: bool,
}

impl<N> FastDfs<N>
where
    N: FastNode,
{
    #[inline]
    pub fn new<R, D>(root: R, max_depth: D, allow_circles: bool) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = queue::Queue::new(allow_circles);
        // let visited = HashSet::new();
        let root: N = root.into();
        let max_depth = max_depth.into();
        if let Err(err) = root.add_children(1, &mut queue) {
            queue.add(0, Err(err));
        }
        Self {
            queue,
            // visited,
            max_depth,
            // allow_circles,
        }
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
            Some((depth, Err(err))) => Some(Err(err)),
            // next node succeeded
            Some((depth, Ok(node))) => {
                if let Some(max_depth) = self.max_depth {
                    if depth >= max_depth {
                        return Some(Ok(node));
                    }
                }
                match node.add_children(depth + 1, &mut self.queue) {
                    Ok(_) => Some(Ok(node)),
                    Err(err) => Some(Err(err)),
                }
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

    parallel_iterator!(Dfs<Node>);
    parallel_iterator!(FastDfs<FastNode>);
}

#[cfg(feature = "rayon")]
pub use par::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

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
                    assert_eq_vec!(depths, expected_depths);
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

    // macro_rules! test_depths {
    //     ($name:ident: $values:expr) => {
    //         paste::item! {
    //             #[test]
    //             fn [< test_ $name _ serial >] () -> Result<()> {
    //                 let (iter, expected_depths) = $values;
    //                 let output = iter.collect::<Result<Vec<_>, _>>()?;
    //                 let depths: Vec<_> = output.into_iter()
    //                     .map(|item| item.0).collect();
    //                 assert_eq!(depths, expected_depths);
    //                 Ok(())
    //             }

    //             #[cfg(feature = "rayon")]
    //             #[test]
    //             fn [< test_ $name _ parallel >] () -> Result<()> {

    //                 let (iter, expected_depths) = $values;
    //                 let output = iter.into_par_iter()
    //                     .collect::<Result<Vec<_>, _>>()?;
    //                 let depths: Vec<_> = output.into_iter()
    //                     .map(|item| item.0).collect();
    //                 assert_eq_vec!(depths, expected_depths);
    //                 Ok(())
    //             }

    //         }
    //     };
    // }

    test_depths!(
        dfs:
        (
            Dfs::<TestNode>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_dfs:
        (
            FastDfs::<TestNode>::new(0, 3, true),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        ),
        test_depths_serial,
        test_depths_parallel,
    );

    test_depths!(
        fast_dfs_no_circles:
        (
            FastDfs::<TestNode>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
    );

    test_depths!(
        dfs_no_circles:
        (
            Dfs::<TestNode>::new(0, 3, false),
            [1, 2, 3]
        ),
        test_depths_serial,
    );
}
