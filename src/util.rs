use std::cmp::Ordering::*;

/// Tries to pair up two sorted slices.
/// Returns pair of slices that are one of:
/// - eq => (Some(a),Some(a))
/// - diff => (Some(a),None) or (None,Some(a))
pub fn merge<'a, T: Ord + Eq>(
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
