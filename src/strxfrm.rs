pub unsafe fn strxfrm(dest: *mut u8, src: *const u8, n: usize) -> usize {
    let mut len = 0usize;
    while *src.add(len) != 0 {
        len += 1;
    }
    if n > len && !dest.is_null() && !src.is_null() {
        core::ptr::copy_nonoverlapping(src, dest, len + 1);
    }
    len
}
