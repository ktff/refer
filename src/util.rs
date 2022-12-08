use std::{
    cmp::Ordering::*,
    iter::{Peekable, Rev},
};

/// Tries to pair up two sorted slices.
/// Returns pair of slices that are one of:
/// - eq => (Some(a),Some(a))
/// - diff => (Some(a),None) or (None,Some(a))
pub fn pair_up<'a, T: Ord + Eq>(
    a: &'a [T],
    b: &'a [T],
) -> impl Iterator<Item = (Option<&'a T>, Option<&'a T>)> + 'a {
    let mut a_iter = a.iter();
    let mut b_iter = b.iter();
    let mut a_next = a_iter.next();
    let mut b_next = b_iter.next();
    std::iter::from_fn(move || match (a_next, b_next) {
        (Some(a), Some(b)) => match a.cmp(b) {
            Less => {
                a_next = a_iter.next();
                Some((Some(a), None))
            }
            Greater => {
                b_next = b_iter.next();
                Some((None, Some(b)))
            }
            Equal => {
                a_next = a_iter.next();
                b_next = b_iter.next();
                Some((Some(a), Some(b)))
            }
        },
        (Some(a), None) => {
            a_next = a_iter.next();
            Some((Some(a), None))
        }
        (None, Some(b)) => {
            b_next = b_iter.next();
            Some((None, Some(b)))
        }
        (None, None) => None,
    })
}

/// Ascending iterator.
pub struct AscendingIterator<I: Iterator>(I)
where
    I::Item: Ord;

impl<I: Iterator> AscendingIterator<I>
where
    I::Item: Ord,
{
    /// Iter must be sorted in ascending order.
    pub fn ascending(iter: I) -> Self {
        AscendingIterator(iter)
    }

    /// Iter must be sorted in descending order.
    pub fn descending(iter: I) -> AscendingIterator<Rev<I>>
    where
        I: DoubleEndedIterator,
    {
        AscendingIterator(iter.rev())
    }

    /// Dedup next items.
    /// With items returns number of occurrence.
    pub fn dedup(self) -> DedupAscendingIterator<I>
    where
        I::Item: Eq,
    {
        DedupAscendingIterator(self.0.peekable())
    }

    pub fn map_internal<T: Iterator<Item = I::Item>>(
        self,
        map: impl FnOnce(I) -> T,
    ) -> AscendingIterator<T> {
        AscendingIterator(map(self.0))
    }
}

impl<I: Iterator> Iterator for AscendingIterator<I>
where
    I::Item: Ord,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for AscendingIterator<I>
where
    I::Item: Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for AscendingIterator<I>
where
    I::Item: Ord,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct DedupAscendingIterator<I: Iterator>(Peekable<I>)
where
    I::Item: Ord;

impl<I: Iterator> Iterator for DedupAscendingIterator<I>
where
    I::Item: Ord,
{
    type Item = (usize, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.0.next()?;
        let mut count = 1;
        while self.0.peek() == Some(&next) {
            count += 1;
            next = self.0.next()?;
        }
        Some((count, next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.0.size_hint();
        (min.min(1), max)
    }
}
