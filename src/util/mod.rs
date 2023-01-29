#![allow(dead_code)]

pub mod ord_iter;
#[cfg(feature = "shard")]
pub mod shard_alloc;
#[cfg(feature = "shard")]
pub mod shard_box;
#[cfg(feature = "shard")]
pub mod shard_vec;

use std::cmp::Ordering::*;

// TODO: Util tests

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

/// Reads u64 and fills with higher zero if not enough bytes, discards the rest.
fn read_u64<const N: usize>(data: [u8; N]) -> u64 {
    let mut tmp = 0u64;
    // This is safe since they don't overlap and min size is copied.
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), &mut tmp as *mut _ as *mut u8, N.min(8));
    }
    tmp
}

/// Writes u64 to bytes and discards higher bytes if not enough space.
/// Panics if N is larger.
fn write_u64<const N: usize>(num: u64) -> [u8; N] {
    assert!(N <= 8);
    unsafe { std::ptr::read((&num) as *const _ as *const u8 as *const [u8; N]) }
}
