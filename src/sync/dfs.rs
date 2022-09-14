use super::*;
use std::collections::VecDeque;
use std::iter::Iterator;

#[derive(Clone)]
pub struct DfsQueue<I, E> {
    inner: VecDeque<(usize, Result<I, E>)>,
}

impl<I, E> Queue for DfsQueue<I, E> {
    // fn len(&self) -> usize {
    //     self.inner.len()
    // }

    fn split_off(&mut self, at: usize) -> Self {
        let split = self.inner.split_off(at);
        Self { inner: split }
    }
}

impl<I, E> std::ops::Deref for DfsQueue<I, E> {
    type Target = VecDeque<(usize, Result<I, E>)>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I, E> std::ops::DerefMut for DfsQueue<I, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I, E> DfsQueue<I, E> {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }
}

impl<I, E> ExtendQueue<I, E> for DfsQueue<I, E> {
    fn add(&mut self, depth: usize, item: Result<I, E>) {
        self.inner.push_back((depth, item));
    }

    fn add_all<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>,
    {
        self.inner.extend(iter.into_iter().map(|i| (depth, i)));
    }
}

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Dfs<N>
where
    N: Node,
{
    queue: DfsQueue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> Dfs<N>
where
    N: Node,
{
    #[inline]
    pub fn new<R, D>(root: R, max_depth: D) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = DfsQueue::new();
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
            Some((depth, Err(err))) => Some(Err(err)),
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
                        Some(Ok(node))
                    }
                    Err(err) => Some(Err(err)),
                }
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
    queue: DfsQueue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> FastDfs<N>
where
    N: FastNode,
{
    #[inline]
    pub fn new<R, D>(root: R, max_depth: D) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = DfsQueue::new();
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
    use crate::utils::test::sync::*;
    use crate::utils::test::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    macro_rules! test_depths {
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

                #[cfg(feature = "rayon")]
                #[test]
                fn [< test_ $name _ parallel >] () -> Result<()> {
                    // use crate::sync::par::IntoParallelIterator;
                    use rayon::iter::IntoParallelIterator;
                    use rayon::iter::ParallelIterator;

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

    test_depths!(
        dfs:
        (
            Dfs::<TestNode>::new(0, 3),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        )
    );

    test_depths!(
        fast_dfs:
        (
            FastDfs::<TestNode>::new(0, 3),
            [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3]
        )
    );
}
