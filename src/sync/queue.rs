use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Queue<I, E> {
    inner: VecDeque<(usize, Result<I, E>)>,
    visited: HashSet<I>,
    allow_circles: bool,
}

impl<I, E> super::Queue<I, E> for Queue<I, E>
where
    I: Clone,
{
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn pop_back(&mut self) -> Option<(usize, Result<I, E>)> {
        self.inner.pop_back()
    }

    fn pop_front(&mut self) -> Option<(usize, Result<I, E>)> {
        self.inner.pop_front()
    }

    fn split_off(&mut self, at: usize) -> Self {
        let split = self.inner.split_off(at);
        // cannot find circles with parallel iterator
        self.allow_circles = true;
        self.visited.clear();
        Self {
            inner: split,
            visited: HashSet::new(),
            allow_circles: true,
        }
    }
}

impl<I, E> Queue<I, E> {
    #[inline]
    #[must_use]
    pub fn new(allow_circles: bool) -> Self {
        Self {
            inner: VecDeque::new(),
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

impl<I, E> super::ExtendQueue<I, E> for Queue<I, E>
where
    I: Hash + Eq + Clone,
    E: Hash + Eq + Clone,
{
    #[inline]
    fn add(&mut self, depth: usize, item: Result<I, E>) {
        if self.allow_circles {
            self.inner.push_back((depth, item));
        } else {
            match item {
                Ok(item) => {
                    if !self.visited.contains(&item) {
                        self.inner.push_back((depth, Ok(item.clone())));
                    }
                    self.visited.insert(item);
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
            let new = iter
                .into_iter()
                .filter(|c| match c {
                    Ok(item) => !self.visited.contains(item),
                    Err(_) => true,
                })
                .collect::<HashSet<_>>();
            self.visited
                .extend(new.iter().cloned().filter_map(Result::ok));
            self.inner.extend(new.into_iter().map(|i| (depth, i)));
        }
    }
}
