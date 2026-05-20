use core::ffi::c_char;

pub unsafe fn strxfrm(dest: *mut c_char, src: *const c_char, n: usize) -> usize {
    let s = src as *const u8;
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    if n > len && !dest.is_null() && !src.is_null() {
        core::ptr::copy_nonoverlapping(s, dest as *mut u8, len + 1);
    }
    len
}

