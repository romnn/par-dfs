use super::*;
use std::collections::VecDeque;
use std::iter::Iterator;

#[derive(Clone)]
struct BaseQueue<I, E> {
    inner: VecDeque<(usize, Result<I, E>)>,
}

impl<I, E> std::ops::Deref for BaseQueue<I, E> {
    type Target = VecDeque<(usize, Result<I, E>)>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I, E> std::ops::DerefMut for BaseQueue<I, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I, E> BaseQueue<I, E> {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            inner: self.inner.split_off(at),
        }
    }
}

// impl<I, E> Queue<I, E> for BaseQueue<I, E> {
//     fn next(&mut self) -> Option<(usize, Result<I, E>)> {
//         self.inner.pop_front()
//     }
// }

impl<I, E> ExtendQueue<I, E> for BaseQueue<I, E> {
    fn add(&mut self, depth: usize, item: Result<I, E>) {
        self.inner.push_back((depth, item));
    }

    fn extend<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>,
    {
        self.inner.extend(iter.into_iter().map(|i| (depth, i)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::sync::*;
    // use crate::utils::test::sync::*;
    // use crate::utils::test::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use std::convert::Infallible;
    // use std::cmp::Ordering;

    #[derive(Clone)]
    struct BfsQueue<I, E> {
        inner: BaseQueue<I, E>,
    }

    impl<I, E> std::ops::Deref for BfsQueue<I, E> {
        type Target = BaseQueue<I, E>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<I, E> std::ops::DerefMut for BfsQueue<I, E> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    impl<I, E> BfsQueue<I, E> {
        pub fn new() -> Self {
            Self {
                inner: BaseQueue::new(),
            }
        }

        fn next(&mut self) -> Option<(usize, Result<I, E>)> {
            self.inner.pop_front()
        }
    }

    // impl<I, E> Queue<I, E> for BfsQueue<I, E> {
    //     fn next(&mut self) -> Option<(usize, Result<I, E>)> {
    //         self.inner.pop_front()
    //     }
    // }

    #[test]
    fn test_bfs_queue() -> Result<()> {
        let mut queue: BfsQueue<u32, Infallible> = BfsQueue::new();
        queue.extend(0, [1, 2, 3, 4, 5, 6].map(Result::Ok));
        // queue.next()
        Ok(())
    }

    // impl<I, E> ExtendQueue<I, E> for BfsQueue<I, E> {
    //     fn add(&mut self, depth: usize, item: Result<I, E>) {
    //         self.inner.push_back((depth, item));
    //     }

    //     fn extend<Iter>(&mut self, depth: usize, iter: Iter)
    //     where
    //         Iter: IntoIterator<Item = Result<I, E>>,
    //     {
    //         self.inner.extend(iter.into_iter().map(|i| (depth, i)));
    //     }
    // }
}
