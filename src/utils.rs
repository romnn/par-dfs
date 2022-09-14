#[cfg(test)]
pub mod test {
    use std::cmp::{Ord, Ordering};
    use std::iter::IntoIterator;

    macro_rules! assert_eq_vec {
        ($left:expr, $right:expr $(,)?) => {{
            let mut left = $left.clone();
            let mut right = $right.clone();
            left.sort();
            right.sort();
            assert_eq!(left, right);
        }};
        ($left:expr, $right:expr, $($arg:tt)+) => {{
            let mut left = $left.clone();
            let mut right = $right.clone();
            left.sort();
            right.sort();
            assert_eq!(left, right, $($arg)+);
        }};
    }
    pub(crate) use assert_eq_vec;

    #[derive(thiserror::Error, Clone, Debug)]
    #[error("error")]
    pub struct TestError;

    #[derive(Clone, Debug)]
    pub struct TestNode(pub usize);

    impl From<usize> for TestNode {
        fn from(depth: usize) -> Self {
            Self(depth)
        }
    }

    pub mod sync {
        use crate::sync::*;

        impl Node for super::TestNode {
            type Error = super::TestError;

            fn children(&self, depth: usize) -> NodeIter<Self, Self::Error> {
                let nodes = [depth, depth];
                Ok(Box::new(nodes.into_iter().map(|d| Self(d)).map(Result::Ok)))
            }
        }

        impl FastNode for super::TestNode {
            type Error = super::TestError;

            fn add_children<E>(&self, depth: usize, queue: &mut E) -> Result<(), Self::Error>
            where
                E: ExtendQueue<Self, Self::Error>,
            {
                queue.add(depth, Ok(Self(depth)));
                queue.add_all(depth, [Ok(Self(depth))]);
                Ok(())
            }
        }
    }

    pub(crate) fn is_monotonic<I, T>(iter: I, order: Ordering) -> bool
    where
        I: IntoIterator<Item = T>,
        <I as IntoIterator>::IntoIter: Clone,
        T: Ord,
    {
        let prev = iter.into_iter();
        let next = prev.clone().next();
        prev.zip(next).all(|(prev, next)| {
            let found = next.cmp(&prev);
            found == Ordering::Equal || found == order
        })
    }
}
