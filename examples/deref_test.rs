use std::collections::VecDeque;
use std::convert::Infallible;
use std::ops::Deref;

pub type NodeQueueType<I, E> = VecDeque<(usize, Result<I, E>)>;

// pub trait Queue<I, E> {
pub trait Queue {
    fn len(&self) -> usize;
    fn split_off(&mut self, at: usize) -> Self;
    // fn next(&mut self) -> Option<(usize, Result<I, E>)>;
}

#[derive(Clone)]
pub struct NodeQueue<I, E> {
    inner: NodeQueueType<I, E>,
}

impl<I, E> From<NodeQueueType<I, E>> for NodeQueue<I, E> {
    fn from(inner: NodeQueueType<I, E>) -> Self {
        Self { inner }
    }
}

impl<I, E> NodeQueue<I, E> {
    pub fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }
}

impl<I, E> std::ops::Deref for NodeQueue<I, E> {
    type Target = NodeQueueType<I, E>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I, E> std::ops::DerefMut for NodeQueue<I, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// impl<I, E> Queue for NodeQueue<I, E> {
//     fn len(&self) -> usize {
//         self.inner.len()
//     }

//     fn split_off(&mut self, at: usize) -> Self {
//         let split = self.inner.split_off(at);
//         Self { inner: split }
//     }
// }

// impl<T, I, E> Queue for T
// where
//     T: Deref<Target = NodeQueue<I, E>>,
// {
//     // fn new(inner: ) -> usize {
//     //     self.inner.len()
//     // }

//     // fn split_off(&mut self, at: usize) -> Self {
//     //     let split = self.inner.split_off(at);

//     fn len(&self) -> usize {
//         self.inner.len()
//     }

//     fn split_off(&mut self, at: usize) -> Self {
//         let split = self.inner.split_off(at);
//         // split.into()
//         Self::new(split)
//         // Self { inner: split }
//     }
// }

pub mod dfs {
    use super::*;

    #[derive(Clone)]
    pub struct DfsQueue<I, E> {
        inner: NodeQueue<I, E>,
    }

    impl<I, E> DfsQueue<I, E> {
        pub fn new() -> Self {
            Self {
                inner: NodeQueue::new(),
            }
        }
    }

    impl<I, E> std::ops::Deref for DfsQueue<I, E> {
        type Target = NodeQueue<I, E>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<I, E> std::ops::DerefMut for DfsQueue<I, E> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
}

struct IsQueue<T: Queue>(T);

fn main() {
    let mut queue: NodeQueue<usize, Infallible> = NodeQueue::new();
    queue.push_back((0, Ok(0)));
    queue.push_back((1, Ok(1)));
    // let test: Box<&dyn Queue> = Box::new(&queue.clone());
    // IsQueue(queue.clone());

    use dfs::*;
    let mut queue: DfsQueue<usize, Infallible> = DfsQueue::new();
    queue.push_back((0, Ok(0)));
    queue.push_back((1, Ok(1)));
    // IsQueue(queue.clone());
    // let test: Box<&dyn Queue> = Box::new(&queue.clone());
}
