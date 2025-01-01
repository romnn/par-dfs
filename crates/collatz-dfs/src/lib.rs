use std::collections::{HashSet, VecDeque};
use std::iter::Iterator;

type Queue = VecDeque<(usize, Result<u32, std::convert::Infallible>)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollatzNode(pub u32);

impl From<CollatzNode> for u32 {
    #[inline]
    fn from(n: CollatzNode) -> Self {
        n.0
    }
}

impl From<u32> for CollatzNode {
    #[inline]
    fn from(n: u32) -> Self {
        Self(n)
    }
}

impl CollatzNode {
    #[inline]
    pub fn collatz_children(
        &self,
    ) -> impl Iterator<Item = Result<CollatzNode, std::convert::Infallible>> {
        let n = self.0;
        let mut children = vec![];

        // n can be reached by dividing by two
        // as long as it doesn't overflow
        if let Some(even) = n.checked_mul(2) {
            children.push(even);
        }

        // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
        if n > 4 && n % 6 == 4 {
            children.push((n - 1) / 3);
        }
        children.into_iter().map(Self).map(Result::Ok)
    }
}

mod sync_collatz {
    use super::CollatzNode;
    use par_dfs::sync::{ExtendQueue, FastNode, Node, NodeIter};

    impl FastNode for CollatzNode {
        type Error = std::convert::Infallible;

        #[inline]
        fn add_children<E>(&self, _depth: usize, queue: &mut E) -> Result<(), Self::Error>
        where
            E: ExtendQueue<Self, Self::Error>,
        {
            let n = self.0;

            // n can be reached by dividing by two
            // as long as it doesn't overflow
            if let Some(even) = n.checked_mul(2) {
                queue.add(Ok(Self(even)));
            }

            // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
            if n > 4 && n % 6 == 4 {
                queue.add(Ok(Self((n - 1) / 3)));
            }
            Ok(())
        }
    }

    impl Node for CollatzNode {
        type Error = std::convert::Infallible;

        #[inline]
        fn children(&self, _depth: usize) -> NodeIter<Self, Self::Error> {
            Ok(Box::new(self.collatz_children()))
        }
    }
}

mod async_collatz {
    use super::CollatzNode;
    use futures::StreamExt;
    use par_dfs::r#async::{Node, NodeStream};
    use std::sync::Arc;

    #[async_trait::async_trait]
    impl Node for CollatzNode {
        type Error = std::convert::Infallible;

        #[inline]
        async fn children(
            self: Arc<Self>,
            _depth: usize,
        ) -> Result<NodeStream<Self, Self::Error>, Self::Error> {
            let stream = futures::stream::iter(self.collatz_children()).boxed();
            Ok(Box::pin(stream))
        }
    }
}

/// Enumerates the numbers that reach the given starting point when iterating
/// the [Collatz] map, by depth-first search over the [graph] of their orbits.
///
/// [Collatz]: https://en.wikipedia.org/wiki/Collatz_conjecture
/// [graph]: https://en.wikipedia.org/wiki/File:Collatz_orbits_of_the_all_integers_up_to_1000.svg
#[derive(Clone, Debug)]
pub struct CollatzDfs {
    max_depth: Option<usize>,
    queue: Queue,
    visited: HashSet<u32>,
    allow_circles: bool,
}

impl CollatzDfs {
    pub fn new<D: Into<Option<usize>>>(start: u32, max_depth: D, allow_circles: bool) -> Self {
        Self {
            max_depth: max_depth.into(),
            queue: VecDeque::from_iter([(0, Ok(start))]),
            visited: HashSet::from_iter([start]),
            allow_circles,
        }
    }
}

impl Iterator for CollatzDfs {
    type Item = Result<u32, std::convert::Infallible>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.queue.pop_back() {
            Some((depth, Ok(n))) => {
                self.visited.insert(n);

                if let Some(max_depth) = self.max_depth {
                    if depth >= max_depth {
                        return Some(Ok(n));
                    }
                }
                // n can be reached by dividing by two
                // as long as it doesn't overflow
                if let Some(even) = n.checked_mul(2) {
                    if self.allow_circles || !self.visited.contains(&even) {
                        self.queue.push_back((depth + 1, Ok(even)));
                    }
                }

                // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
                if n > 4 && n % 6 == 4 {
                    let odd = (n - 1) / 3;
                    if self.allow_circles || !self.visited.contains(&odd) {
                        self.queue.push_back((depth + 1, Ok(odd)));
                    }
                }
                Some(Ok(n))
            }
            Some((_, n)) => Some(n),
            None => None,
        }
    }
}

impl par_dfs::sync::par::SplittableIterator for CollatzDfs {
    fn split(&mut self) -> Option<Self> {
        let len = self.queue.len();
        if len >= 2 {
            let split = self.queue.split_off(len / 2);
            // cannot avoid circles when running in parallel
            self.visited.clear();
            self.allow_circles = true;
            Some(Self {
                queue: split,
                max_depth: self.max_depth,
                visited: HashSet::new(),
                allow_circles: true,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_collatz_correctness() {
        let start = 1;
        let limit = 10;
        let allow_circles = false;

        let plain: Vec<_> = super::CollatzDfs::new(start, limit, allow_circles)
            .map(Result::ok)
            .collect();
        let sync: Vec<_> =
            par_dfs::sync::FastDfs::<super::CollatzNode>::new(start, limit, allow_circles)
                .map(|n| n.ok().map(Into::into))
                .collect();
        similar_asserts::assert_eq!(plain, sync);
    }
}
