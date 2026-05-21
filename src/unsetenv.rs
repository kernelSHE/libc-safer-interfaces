use core::ffi::{c_char, c_int};

fn get_environ() -> *mut *mut c_char {
    unsafe {
        let sym = crate::dlsym(
            crate::RTLD_DEFAULT,
            b"environ\0".as_ptr() as *const c_char,
        );
        if sym.is_null() {
            return core::ptr::null_mut();
        }
        (sym as *const *mut *mut c_char).read()
    }
}

unsafe fn strchrnul(s: *const u8, c: u8) -> *const u8 {
    let mut p = s;
    while *p != 0 && *p != c {
        p = p.add(1);
    }
    p
}

unsafe fn my_strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
        if a == 0 {
            break;
        }
        i += 1;
    }
    0
}

fn unsetenv_impl(name: *const u8) -> i32 {
    unsafe {
        if name.is_null() || *name == 0 {
            return -1;
        }
        let l = strchrnul(name, b'=').offset_from(name) as usize;
        if l == 0 || *name.add(l) != 0 {
            return -1;
        }
        let mut e = get_environ();
        if e.is_null() {
            return 0;
        }
        let mut eo = e;
        while !(*e).is_null() {
            let entry = *e as *const u8;
            if my_strncmp(name, entry, l) == 0 && *entry.add(l) == b'=' {
            } else if eo != e {
                *eo = *e;
                eo = eo.add(1);
            } else {
                eo = eo.add(1);
            }
            e = e.add(1);
        }
        if eo != e {
            *eo = core::ptr::null_mut();
        }
        0
    }
}

pub fn unsetenv(name: *const c_char) -> c_int {
    unsetenv_impl(name as *const u8)
}
