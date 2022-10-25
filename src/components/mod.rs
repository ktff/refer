pub mod alloc;
pub mod alloc_box;
pub mod inline_shell;
pub mod inline_vec;

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
