use super::*;
use std::collections::VecDeque;
use std::iter::Iterator;

#[derive(Clone)]
pub struct DfsQueue<I, E> {
    inner: VecDeque<(usize, Result<I, E>)>,
}

impl<I, E> Queue for DfsQueue<I, E> {
    fn len(&self) -> usize {
        self.inner.len()
    }

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

// impl<I, E> DfsQueue<I, E> {
//     pub fn split_off(&mut self, at: usize) -> Self {
//         Self {
//             inner: self.inner.split_off(at),
//         }
//     }
// }

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
// pub struct Dfs<Q, N>
where
    N: Node,
    // Q: Queue // <N, N::Error>,
{
    // queue: Q,
    queue: DfsQueue<N, N::Error>,
    max_depth: Option<usize>,
}

// impl<Q, N> Dfs<Q, N>
impl<N> Dfs<N>
where
    N: Node,
    // Q: Queue<N, N::Error>,
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

// impl<Q, I, E, N> GraphIterator<Q, I, E> for Dfs<Q, N>
// impl<Q, N> GraphIterator<Q, N, N::Error> for Dfs<Q, N>
// impl<Q, N> GraphIterator<Q> for Dfs<Q, N>
//<Queue = DfsQueue<N, N::Error>>
impl<N> Iterator for Dfs<N>
where
    N: Node,
    // Q: Queue, // <N, N::Error>,
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
    // Q: Queue<N, N::Error>,
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

// impl<N> GraphIterator for FastDfs<N>
// where
//     N: FastNode,
// {
//     fn queue(&self) -> Vec<usize> {
//         vec![]
//     }
// }

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

    impl<N> HasQueue for Dfs<N>
    where
        N: Node,
    {
        type Queue = DfsQueue<N, N::Error>;
        fn queue_mut(&mut self) -> &mut Self::Queue {
            &mut self.queue
        }
        fn queue(&self) -> &Self::Queue {
            &self.queue
        }
    }

    impl<N> GraphIterator<DfsQueue<N, N::Error>> for Dfs<N>
    where
        N: Node,
    {
        fn from_split(&self, queue: DfsQueue<N, N::Error>) -> Self {
            Self {
                queue,
                max_depth: self.max_depth,
            }
        }
    }

    impl<N> HasQueue for FastDfs<N>
    where
        N: FastNode,
    {
        type Queue = DfsQueue<N, N::Error>;
        fn queue_mut(&mut self) -> &mut Self::Queue {
            &mut self.queue
        }
        fn queue(&self) -> &Self::Queue {
            &self.queue
        }
    }

    impl<N> GraphIterator<DfsQueue<N, N::Error>> for FastDfs<N>
    where
        N: FastNode,
    {
        fn from_split(&self, queue: DfsQueue<N, N::Error>) -> Self {
            Self {
                queue,
                max_depth: self.max_depth,
            }
        }
    }

    //     impl<N> SplittableIterator for super::Dfs<N>
    //     where
    //         N: Node,
    //     {
    //         #[inline(always)]
    //         fn split(&mut self) -> Option<Self> {
    //             let len = self.queue.len();
    //             if len >= 2 {
    //                 let split = self.queue.split_off(len / 2);
    //                 Some(Self {
    //                     queue: split,
    //                     max_depth: self.max_depth,
    //                 })
    //             } else {
    //                 None
    //             }
    //         }
    //     }

    //     impl<N> SplittableIterator for super::FastDfs<N>
    //     where
    //         N: FastNode,
    //     {
    //         #[inline(always)]
    //         fn split(&mut self) -> Option<Self> {
    //             let len = self.queue.len();
    //             if len >= 2 {
    //                 let split = self.queue.split_off(len / 2);
    //                 Some(Self {
    //                     queue: split,
    //                     max_depth: self.max_depth,
    //                 })
    //             } else {
    //                 None
    //             }
    //         }
    //     }

    impl<N> rayon::iter::IntoParallelIterator for FastDfs<N>
    where
        N: FastNode + Send,
        N::Error: Send,
    {
        type Iter = ParallelSplittableIterator<Self>;
        type Item = <Self as Iterator>::Item;

        fn into_par_iter(self) -> Self::Iter {
            ParallelSplittableIterator::new(self)
        }
    }

    impl<N> rayon::iter::IntoParallelIterator for Dfs<N>
    where
        N: Node + Send,
        N::Error: Send,
    {
        type Iter = ParallelSplittableIterator<Self>;
        type Item = <Self as Iterator>::Item;

        fn into_par_iter(self) -> Self::Iter {
            ParallelSplittableIterator::new(self)
        }
    }
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

    #[test]
    fn test_dfs() -> Result<()> {
        let dfs: Dfs<TestNode> = Dfs::new(0, 3);
        let expected_depths = [1, 2, 3, 3, 2, 3, 3, 1, 2, 3, 3, 2, 3, 3];

        let output = dfs.clone().collect::<Result<Vec<_>, _>>()?;
        println!("dfs output: {:?}", &output);
        let depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
        assert_eq!(depths, expected_depths);

        #[cfg(feature = "rayon")]
        {
            // use crate::sync::par::IntoParallelIterator;
            use rayon::iter::IntoParallelIterator;
            use rayon::iter::ParallelIterator;

            let output = dfs.clone().into_par_iter().collect::<Result<Vec<_>, _>>()?;
            println!("dfs parallel output: {:?}", &output);
            let depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
            assert_eq_vec!(depths, expected_depths);
        }

        let dfs: FastDfs<TestNode> = FastDfs::new(0, 3);
        let output = dfs.clone().collect::<Result<Vec<_>, _>>()?;
        println!("fast dfs output: {:?}", &output);
        let depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
        assert_eq!(depths, expected_depths);

        #[cfg(feature = "rayon")]
        {
            // use crate::sync::par::IntoParallelIterator;
            use rayon::iter::IntoParallelIterator;
            use rayon::iter::ParallelIterator;

            let output = dfs.clone().into_par_iter().collect::<Result<Vec<_>, _>>()?;
            println!("fast dfs parallel output: {:?}", &output);
            let mut depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
            assert_eq_vec!(depths, expected_depths);
        }

        // assert_eq!(0, 2);
        Ok(())
    }
}
