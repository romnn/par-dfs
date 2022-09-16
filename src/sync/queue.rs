use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
#[cfg(feature = "rayon")]
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub(super) struct Queue<I, E> {
    inner: VecDeque<(usize, Result<I, E>)>,
    #[cfg(feature = "rayon")]
    visited: Arc<RwLock<HashSet<I>>>,
    #[cfg(not(feature = "rayon"))]
    visited: HashSet<I>,
    allow_circles: bool,
}

#[cfg(feature = "rayon")]
#[inline]
fn unvisited<I>(visited: &mut Arc<RwLock<HashSet<I>>>, item: &I) -> bool
where
    I: Hash + Eq + Clone,
{
    if visited.read().unwrap().contains(item) {
        false
    } else {
        visited.write().unwrap().insert(item.clone());
        true
    }
}

#[cfg(not(feature = "rayon"))]
#[inline]
fn unvisited<I>(visited: &mut HashSet<I>, item: &I) -> bool
where
    I: Hash + Eq + Clone,
{
    if visited.contains(item) {
        false
    } else {
        visited.insert(item.clone());
        true
    }
}

impl<I, E> super::Queue<I, E> for Queue<I, E>
where
    I: Hash + Eq + Clone,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    fn pop_back(&mut self) -> Option<(usize, Result<I, E>)> {
        self.inner.pop_back()
    }

    #[inline]
    fn pop_front(&mut self) -> Option<(usize, Result<I, E>)> {
        self.inner.pop_front()
    }

    #[inline]
    fn split_off(&mut self, at: usize) -> Self {
        let split = self.inner.split_off(at);
        Self {
            inner: split,
            visited: self.visited.clone(),
            allow_circles: self.allow_circles,
        }
    }

    #[inline]
    fn add(&mut self, depth: usize, item: Result<I, E>) {
        if self.allow_circles {
            self.inner.push_back((depth, item));
        } else {
            match item {
                Ok(item) => {
                    if unvisited(&mut self.visited, &item) {
                        self.inner.push_back((depth, Ok(item.clone())));
                    }
                }
                Err(err) => self.inner.push_back((depth, Err(err))),
            }
        }
    }

    #[inline]
    fn add_all<Iter>(&mut self, depth: usize, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>,
    {
        if self.allow_circles {
            self.inner.extend(iter.into_iter().map(|i| (depth, i)));
        } else {
            let not_visited = iter.into_iter().filter(|c| match c {
                Ok(item) => unvisited(&mut self.visited, item),
                Err(_) => true,
            });
            self.inner.extend(not_visited.map(|i| (depth, i)));
        }
    }
}

impl<I, E> Queue<I, E> {
    #[inline]
    #[must_use]
    pub fn new(allow_circles: bool) -> Self {
        Self {
            inner: VecDeque::new(),
            #[cfg(feature = "rayon")]
            visited: Arc::new(RwLock::new(HashSet::new())),
            #[cfg(not(feature = "rayon"))]
            visited: HashSet::new(),
            allow_circles,
        }
    }
}

impl<I, E> Default for Queue<I, E> {
    #[inline]
    fn default() -> Self {
        Self::new(false)
    }
}

pub(super) struct QueueWrapper<'a, Q> {
    inner: &'a mut Q,
    depth: usize,
}

impl<'a, Q> QueueWrapper<'a, Q> {
    #[inline]
    pub fn new(depth: usize, queue: &'a mut Q) -> Self {
        Self {
            inner: queue,
            depth,
        }
    }
}

impl<'a, I, E, Q> super::ExtendQueue<I, E> for QueueWrapper<'a, Q>
where
    Q: super::Queue<I, E>,
    I: Hash + Eq + Clone,
{
    #[inline]
    fn add(&mut self, item: Result<I, E>) {
        self.inner.add(self.depth, item);
    }

    #[inline]
    fn add_all<Iter>(&mut self, iter: Iter)
    where
        Iter: IntoIterator<Item = Result<I, E>>,
    {
        self.inner.add_all(self.depth, iter);
    }
}
