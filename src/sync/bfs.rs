use super::*;
use std::collections::VecDeque;
use std::iter::Iterator;

#[derive(Clone)]
pub struct BfsQueue<I, E> {
    current: VecDeque<(usize, Result<I, E>)>,
    next: VecDeque<(usize, Result<I, E>)>,
}

impl<I, E> Queue for BfsQueue<I, E> {
    fn len(&self) -> usize {
        self.current.len()
    }

    fn split_off(&mut self, at: usize) -> Self {
        let split = self.current.split_off(at);
        Self {
            current: split,
            next: VecDeque::new(),
        }
    }
}

impl<I, E> std::ops::Deref for BfsQueue<I, E> {
    type Target = VecDeque<(usize, Result<I, E>)>;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl<I, E> std::ops::DerefMut for BfsQueue<I, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current
    }
}

impl<I, E> BfsQueue<I, E> {
    pub fn new() -> Self {
        Self {
            current: VecDeque::new(),
            next: VecDeque::new(),
        }
    }
}

impl<I, E> BfsQueue<I, E>
where
    I: std::fmt::Debug,
    E: std::fmt::Debug,
{
    pub fn inspect(&self) {
        println!("current = {:?}", self.current);
        println!("next = {:?}", self.next);
    }
}

impl<I, E> ExtendQueue<I, E> for BfsQueue<I, E> {
    fn add(&mut self, depth: usize, item: Result<I, E>) {
        self.current.push_back((depth, item));
        // self.next.push_back((depth, item));
    }

    fn add_all<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>,
    {
        self.current.extend(iter.into_iter().map(|i| (depth, i)));
        // self.next.extend(iter.into_iter().map(|i| (depth, i)));
    }

    fn next_nodes(&mut self) {
        // println!("next nodes");
        // std::mem::swap(&mut self.current, &mut self.next);
        // while let Some(next) = self.next.pop_front() {
        //     self.current.push_back(next);
        // }
    }
}

// impl<I, E> BfsQueue<I, E> {
//     pub fn split_off(&mut self, at: usize) -> Self {
//         Self {
//             inner: self.inner.split_off(at),
//         }
//     }
// }

// impl<I, E> Queue<I, E> for BfsQueue<I, E> {
//     fn next(&mut self) -> Option<(usize, Result<I, E>)> {
//         self.inner.pop_front()
//     }
// }

#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Bfs<N>
where
    N: Node,
{
    queue: BfsQueue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N, E> Bfs<N>
where
    N: Node<Error = E>,
{
    #[inline]
    pub fn new<R, D>(root: R, max_depth: D) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = BfsQueue::new();
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

impl<N, E> Iterator for Bfs<N>
where
    N: Node<Error = E>,
    E: std::fmt::Debug,
{
    type Item = Result<N, N::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // self.queue.inspect();
        if self.queue.is_empty() {
            self.queue.next_nodes();
        }
        // self.queue.inspect();
        match self.queue.pop_front() {
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
pub struct FastBfs<N>
where
    N: FastNode,
    // Q: Queue<N, N::Error>,
{
    queue: BfsQueue<N, N::Error>,
    max_depth: Option<usize>,
}

impl<N> FastBfs<N>
where
    N: FastNode,
{
    #[inline]
    pub fn new<R, D>(root: R, max_depth: D) -> Self
    where
        R: Into<N>,
        D: Into<Option<usize>>,
    {
        let mut queue = BfsQueue::new();
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_front() {
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

    impl<N> HasQueue for Bfs<N>
    where
        N: Node,
    {
        type Queue = BfsQueue<N, N::Error>;
        fn queue_mut(&mut self) -> &mut Self::Queue {
            &mut self.queue
        }
        fn queue(&self) -> &Self::Queue {
            &self.queue
        }
    }

    impl<N> GraphIterator<BfsQueue<N, N::Error>> for Bfs<N>
    where
        N: Node,
    {
        fn from_split(&self, queue: BfsQueue<N, N::Error>) -> Self {
            Self {
                queue,
                max_depth: self.max_depth,
            }
        }
    }

    impl<N> rayon::iter::IntoParallelIterator for Bfs<N>
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

    impl<N> HasQueue for FastBfs<N>
    where
        N: FastNode,
    {
        type Queue = BfsQueue<N, N::Error>;
        fn queue_mut(&mut self) -> &mut Self::Queue {
            &mut self.queue
        }
        fn queue(&self) -> &Self::Queue {
            &self.queue
        }
    }

    impl<N> GraphIterator<BfsQueue<N, N::Error>> for FastBfs<N>
    where
        N: FastNode,
    {
        fn from_split(&self, queue: BfsQueue<N, N::Error>) -> Self {
            Self {
                queue,
                max_depth: self.max_depth,
            }
        }
    }

    impl<N> rayon::iter::IntoParallelIterator for FastBfs<N>
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

    // impl<N, E> SplittableIterator for Bfs<N>
    // where
    //     N: Node<Error = E>,
    // {
    //     fn split(&mut self) -> Option<Self> {
    //         let len = self.queue.len();
    //         if len >= 2 {
    //             let split = self.queue.split_off(len / 2);
    //             Some(Self {
    //                 queue: split,
    //                 max_depth: self.max_depth,
    //             })
    //         } else {
    //             None
    //         }
    //     }
    // }
}

#[cfg(feature = "rayon")]
pub use par::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::*;
    use crate::utils::test::sync::*;
    use crate::utils::test::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use std::cmp::Ordering;

    #[test]
    fn test_bfs() -> Result<()> {
        let bfs: Bfs<TestNode> = Bfs::new(0, 3);
        let expected_depths = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3];

        let output = bfs.clone().collect::<Result<Vec<_>, _>>()?;
        println!("bfs output: {:?}", &output);
        let depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
        assert!(is_monotonic(&depths, Ordering::Greater));
        assert_eq!(depths, expected_depths);

        #[cfg(feature = "rayon")]
        {
            // use crate::sync::par::IntoParallelIterator;
            use rayon::iter::IntoParallelIterator;
            use rayon::iter::ParallelIterator;

            let output = bfs.clone().into_par_iter().collect::<Result<Vec<_>, _>>()?;
            println!("bfs parallel output: {:?}", &output);
            let mut depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
            assert_eq_vec!(depths, expected_depths);
        }

        let bfs: FastBfs<TestNode> = FastBfs::new(0, 3);
        let expected_depths = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3];

        let output = bfs.clone().collect::<Result<Vec<_>, _>>()?;
        println!("fast bfs output: {:?}", &output);
        let depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
        assert!(is_monotonic(&depths, Ordering::Greater));
        assert_eq!(depths, expected_depths);

        #[cfg(feature = "rayon")]
        {
            // use crate::sync::par::IntoParallelIterator;
            use rayon::iter::IntoParallelIterator;
            use rayon::iter::ParallelIterator;

            let output = bfs.clone().into_par_iter().collect::<Result<Vec<_>, _>>()?;
            println!("fast bfs parallel output: {:?}", &output);
            let mut depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
            assert_eq_vec!(depths, expected_depths);
        }

        // use rayon::iter::IntoParallelIterator;
        // use rayon::iter::ParallelIterator;
        // // use rayon::iter::ParallelBridge;
        // // into_par_iter()
        // // let output = dfs.clone().par_bridge().collect::<Result<Vec<_>, _>>()?;
        // let output = dfs.clone().into_par_iter().collect::<Result<Vec<_>, _>>()?;
        // println!("rayon output: {:?}", &output);
        // let depths: Vec<_> = output.into_iter().map(|item| item.0).collect();
        // assert_eq!(depths, expected_depths);

        // assert_eq!(0, 2);
        Ok(())
    }
}
