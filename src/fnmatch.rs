use crate::{c_char, c_int};

const FNM_PATHNAME: c_int = 0x1;
const FNM_NOESCAPE: c_int = 0x2;
const FNM_PERIOD: c_int = 0x4;
const FNM_LEADING_DIR: c_int = 0x8;
const FNM_CASEFOLD: c_int = 0x10;
const FNM_NOMATCH: c_int = 1;

const END: c_int = 0;
const UNMATCHABLE: c_int = -2;
const BRACKET: c_int = -3;
const QUESTION: c_int = -4;
const STAR: c_int = -5;

#[no_mangle]
pub unsafe fn fnmatch(
    pattern: *const c_char,
    name: *const c_char,
    flags: c_int,
) -> c_int {
    if pattern.is_null() || name.is_null() {
        return FNM_NOMATCH;
    }
    let pat = unsafe { std::ffi::CStr::from_ptr(pattern) };
    let s = unsafe { std::ffi::CStr::from_ptr(name) };
    let pat_bytes = pat.to_bytes();
    let str_bytes = s.to_bytes();

    if (flags & FNM_PATHNAME) != 0 {
        let mut p_start = 0usize;
        let mut s_start = 0usize;
        loop {
            let s_end = str_bytes[s_start..].iter().position(|&c| c == b'/');
            let seg_len = match s_end {
                Some(pos) => pos,
                None => str_bytes.len() - s_start,
            };
            let mut p_end = p_start;
            let mut esc = false;
            while p_end < pat_bytes.len() {
                let c = pat_bytes[p_end];
                if !esc && c == b'\\' && (flags & FNM_NOESCAPE) == 0 {
                    esc = true;
                    p_end += 1;
                    continue;
                }
                esc = false;
                if c == b'/' { break; }
                p_end += 1;
            }
            if p_end >= pat_bytes.len() && s_start + seg_len < str_bytes.len() {
                if (flags & FNM_LEADING_DIR) == 0 {
                    return FNM_NOMATCH;
                }
            }
            if fnmatch_internal(
                &pat_bytes[p_start..p_end],
                &str_bytes[s_start..s_start + seg_len],
                flags,
            ) != 0
            {
                return FNM_NOMATCH;
            }
            if p_end >= pat_bytes.len() {
                return 0;
            }
            p_start = p_end + 1;
            if s_start + seg_len >= str_bytes.len() {
                return FNM_NOMATCH;
            }
            s_start = s_start + seg_len + 1;
        }
    } else if (flags & FNM_LEADING_DIR) != 0 {
        for i in 0..=str_bytes.len() {
            if i == str_bytes.len() || str_bytes[i] == b'/' {
                if fnmatch_internal(pat_bytes, &str_bytes[..i], flags) == 0 {
                    return 0;
                }
            }
        }
    }
    fnmatch_internal(pat_bytes, str_bytes, flags)
}

fn fnmatch_internal(pat: &[u8], s: &[u8], flags: c_int) -> c_int {
    if (flags & FNM_PERIOD) != 0 {
        if !s.is_empty() && s[0] == b'.' && (!pat.is_empty() && pat[0] != b'.') {
            return FNM_NOMATCH;
        }
    }

    let (pat, s) = match match_head(pat, s, flags) {
        Some(result) => result,
        None => return FNM_NOMATCH,
    };

    let m = pat.len();
    let n = s.len();
    let endpat = m;
    let mut ptail = 0usize;
    let mut tailcnt = 0usize;

    {
        let mut p = 0usize;
        while p < endpat {
            let (c, pinc) = pat_next(&pat[p..endpat], flags);
            match c {
                STAR => { tailcnt = 0; ptail = p + 1; }
                UNMATCHABLE => { return FNM_NOMATCH; }
                _ => { tailcnt += 1; }
            }
            p += pinc;
        }
    }

    let endstr = n;
    if n < tailcnt { return FNM_NOMATCH; }

    let mut stail = endstr;
    let mut tc = tailcnt;
    while stail > 0 && tc > 0 {
        if s[stail - 1] < 128 {
            stail -= 1;
        } else {
            loop {
                stail -= 1;
                if stail == 0 || (s[stail] as u8).wrapping_sub(0x80) >= 0x40 {
                    break;
                }
            }
        }
        tc -= 1;
    }
    if tc > 0 { return FNM_NOMATCH; }

    let mut p = ptail;
    let mut si = stail;
    loop {
        let (c, pinc) = pat_next(&pat[p..endpat], flags);
        p += pinc;
        let (k, sinc) = str_next(&s[si..endstr]);
        if k <= 0 {
            if c != END { return FNM_NOMATCH; }
            break;
        }
        si += sinc;
        let kfold = if (flags & FNM_CASEFOLD) != 0 { casefold(k) } else { k };
        if c == BRACKET {
            if !match_bracket(&pat[p - pinc..p], k, kfold) {
                return FNM_NOMATCH;
            }
        } else if c != QUESTION && k != c && kfold != c {
            return FNM_NOMATCH;
        }
    }

    let endstr2 = stail;
    let endpat2 = ptail;
    let mut pat_pos = 0usize;
    let mut str_pos = 0usize;

    while pat_pos < endpat2 {
        let mut p2 = pat_pos;
        let mut s2 = str_pos;
        loop {
            let (c, pinc) = pat_next(&pat[p2..endpat2], flags);
            p2 += pinc;
            if c == STAR {
                pat_pos = p2;
                str_pos = s2;
                break;
            }
            let (k, sinc) = str_next(&s[s2..endstr2]);
            if k == 0 { return FNM_NOMATCH; }
            let kfold = if (flags & FNM_CASEFOLD) != 0 { casefold(k) } else { k };
            if c == BRACKET {
                if !match_bracket(&pat[p2 - pinc..p2], k, kfold) {
                    break;
                }
            } else if c != QUESTION && k != c && kfold != c {
                break;
            }
            s2 += sinc;
        }
        let (last_c, _) = pat_next(&pat[p2 - 1..p2], flags);
        if last_c == STAR { continue; }
        let (k, sinc) = str_next(&s[str_pos..endstr2]);
        if k > 0 {
            str_pos += sinc;
        } else {
            loop {
                str_pos += 1;
                if str_pos >= endstr2 { break; }
                let (nk, _) = str_next(&s[str_pos..endstr2]);
                if nk >= 0 { break; }
            }
        }
    }

    0
}

fn match_head<'a>(mut pat: &'a [u8], mut s: &'a [u8], flags: c_int) -> Option<(&'a [u8], &'a [u8])> {
    loop {
        let (c, pinc) = pat_next(pat, flags);
        match c {
            UNMATCHABLE => { return None; }
            STAR => {}
            _ => {
                if c == END {
                    if s.is_empty() {
                        return Some((pat, s));
                    }
                    return None;
                }
                let (k, sinc) = str_next(s);
                if k <= 0 {
                    return None;
                }
                let kfold = if (flags & FNM_CASEFOLD) != 0 { casefold(k) } else { k };
                if c == BRACKET {
                    if !match_bracket(&pat[..pinc], k, kfold) {
                        return None;
                    }
                } else if c != QUESTION && k != c && kfold != c {
                    return None;
                }
                s = &s[sinc..];
                pat = &pat[pinc..];
                continue;
            }
        }
        break;
    }
    Some((pat, s))
}

fn str_next(s: &[u8]) -> (c_int, usize) {
    if s.is_empty() {
        return (0, 0);
    }
    if s[0] >= 128 {
        match std::str::from_utf8(s) {
            Ok(st) => match st.chars().next() {
                Some(ch) => {
                    let step = ch.len_utf8();
                    (ch as c_int, step)
                }
                None => (-1, 1),
            },
            Err(_) => (-1, 1),
        }
    } else {
        (s[0] as c_int, 1)
    }
}

fn pat_next(pat: &[u8], flags: c_int) -> (c_int, usize) {
    if pat.is_empty() {
        return (END, 0);
    }
    if pat[0] == b'\\' && pat.len() > 1 && (flags & FNM_NOESCAPE) == 0 {
        return (pat[1] as c_int, 2);
    }
    if pat[0] == b'[' {
        let mut k = 1usize;
        if k < pat.len() && (pat[k] == b'^' || pat[k] == b'!') { k += 1; }
        if k < pat.len() && pat[k] == b']' { k += 1; }
        while k < pat.len() && pat[k] != b']' {
            if k + 1 < pat.len() && pat[k] == b'['
                && (pat[k + 1] == b':' || pat[k + 1] == b'.' || pat[k + 1] == b'=')
            {
                let z = pat[k + 1];
                k += 2;
                if k < pat.len() { k += 1; }
                while k < pat.len() && !(pat[k - 1] == z && pat[k] == b']') {
                    k += 1;
                }
                if k >= pat.len() { break; }
            }
            k += 1;
        }
        if k >= pat.len() {
            return (b'[' as c_int, 1);
        }
        return (BRACKET, k + 1);
    }
    if pat[0] == b'*' { return (STAR, 1); }
    if pat[0] == b'?' { return (QUESTION, 1); }
    if pat[0] >= 128 {
        match std::str::from_utf8(pat) {
            Ok(st) => match st.chars().next() {
                Some(ch) => (ch as c_int, ch.len_utf8()),
                None => (UNMATCHABLE, 0),
            },
            Err(_) => (UNMATCHABLE, 0),
        }
    } else {
        (pat[0] as c_int, 1)
    }
}

fn casefold(k: c_int) -> c_int {
    let ch = char::from_u32(k as u32).unwrap_or('\0');
    let upper = ch.to_uppercase().next().unwrap_or(ch);
    if upper == ch { ch.to_lowercase().next().unwrap_or(ch) as c_int } else { upper as c_int }
}

fn match_bracket(pat: &[u8], k: c_int, kfold: c_int) -> bool {
    if pat.is_empty() || pat[0] != b'[' { return false; }
    let mut p = 1usize;
    let inv = if p < pat.len() && (pat[p] == b'^' || pat[p] == b'!') {
        p += 1;
        true
    } else {
        false
    };
    if p < pat.len() && pat[p] == b']' {
        if k == b']' as c_int { return !inv; }
        p += 1;
    } else if p < pat.len() && pat[p] == b'-' {
        if k == b'-' as c_int { return !inv; }
        p += 1;
    }
    while p < pat.len() && pat[p] != b']' {
        if p + 2 < pat.len() && pat[p + 1] == b'-' && pat[p + 2] != b']' {
            let lo = pat[p] as c_int;
            let hi = pat[p + 2] as c_int;
            if (lo <= k && k <= hi) || (lo <= kfold && kfold <= hi) {
                return !inv;
            }
            p += 3;
            continue;
        }
        if p + 1 < pat.len() && pat[p] == b'[' && (pat[p + 1] == b':' || pat[p + 1] == b'.' || pat[p + 1] == b'=') {
            let z = pat[p + 1];
            p += 2;
            let start = p;
            while p + 1 < pat.len() && !(pat[p] == z && pat[p + 1] == b']') {
                p += 1;
            }
            if z == b':' && p > start {
                let cls = &pat[start..p];
                let matched = if cls == b"alpha" {
                    (k >= b'a' as c_int && k <= b'z' as c_int) || (k >= b'A' as c_int && k <= b'Z' as c_int)
                } else if cls == b"digit" {
                    k >= b'0' as c_int && k <= b'9' as c_int
                } else if cls == b"alnum" {
                    (k >= b'a' as c_int && k <= b'z' as c_int) || (k >= b'A' as c_int && k <= b'Z' as c_int) || (k >= b'0' as c_int && k <= b'9' as c_int)
                } else if cls == b"space" {
                    k == b' ' as c_int || k == b'\t' as c_int || k == b'\n' as c_int || k == b'\r' as c_int || k == 0x0c || k == 0x0b
                } else if cls == b"blank" {
                    k == b' ' as c_int || k == b'\t' as c_int
                } else if cls == b"upper" {
                    k >= b'A' as c_int && k <= b'Z' as c_int
                } else if cls == b"lower" {
                    k >= b'a' as c_int && k <= b'z' as c_int
                } else if cls == b"punct" {
                    (k >= 33 && k <= 47) || (k >= 58 && k <= 64) || (k >= 91 && k <= 96) || (k >= 123 && k <= 126)
                } else if cls == b"xdigit" {
                    (k >= b'0' as c_int && k <= b'9' as c_int) || (k >= b'a' as c_int && k <= b'f' as c_int) || (k >= b'A' as c_int && k <= b'F' as c_int)
                } else if cls == b"print" {
                    k >= 32 && k <= 126
                } else if cls == b"graph" {
                    k >= 33 && k <= 126
                } else if cls == b"cntrl" {
                    (k >= 0 && k <= 31) || k == 127
                } else {
                    false
                };
                let folded_matched = if kfold != k {
                    if cls == b"alpha" {
                        (kfold >= b'a' as c_int && kfold <= b'z' as c_int) || (kfold >= b'A' as c_int && kfold <= b'Z' as c_int)
                    } else if cls == b"upper" {
                        kfold >= b'A' as c_int && kfold <= b'Z' as c_int
                    } else if cls == b"lower" {
                        kfold >= b'a' as c_int && kfold <= b'z' as c_int
                    } else { matched }
                } else { false };
                if matched || folded_matched { return !inv; }
            }
            if p + 1 < pat.len() { p += 2; } else { p = pat.len(); }
            continue;
        }
        let wc = pat[p] as c_int;
        if wc == k || wc == kfold { return !inv; }
        p += 1;
    }
    inv
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn do_match(pattern: &str, name: &str, flags: c_int) -> c_int {
        let p = CString::new(pattern).unwrap();
        let n = CString::new(name).unwrap();
        unsafe { fnmatch(p.as_ptr(), n.as_ptr(), flags) }
    }

    #[test]
    fn test_literal_match() {
        assert_eq!(do_match("hello", "hello", 0), 0);
        assert_eq!(do_match("hello", "world", 0), FNM_NOMATCH);
    }

    #[test]
    fn test_star() {
        assert_eq!(do_match("*", "anything", 0), 0);
        assert_eq!(do_match("*.c", "main.c", 0), 0);
        assert_eq!(do_match("*.c", "main.h", 0), FNM_NOMATCH);
        assert_eq!(do_match("foo*bar", "foobar", 0), 0);
        assert_eq!(do_match("foo*bar", "fooXbar", 0), 0);
        assert_eq!(do_match("foo*bar", "fooXXbar", 0), 0);
    }

    #[test]
    fn test_question() {
        assert_eq!(do_match("?", "a", 0), 0);
        assert_eq!(do_match("?", "ab", 0), FNM_NOMATCH);
        assert_eq!(do_match("?at", "cat", 0), 0);
        assert_eq!(do_match("?at", "bat", 0), 0);
        assert_eq!(do_match("?at", "at", 0), FNM_NOMATCH);
    }

    #[test]
    fn test_bracket() {
        assert_eq!(do_match("[abc]", "a", 0), 0);
        assert_eq!(do_match("[abc]", "b", 0), 0);
        assert_eq!(do_match("[abc]", "d", 0), FNM_NOMATCH);
        assert_eq!(do_match("[a-z]", "m", 0), 0);
        assert_eq!(do_match("[a-z]", "A", 0), FNM_NOMATCH);
        assert_eq!(do_match("[!a-z]", "A", 0), 0);
        assert_eq!(do_match("[!a-z]", "a", 0), FNM_NOMATCH);
    }

    #[test]
    fn test_fnm_pathname() {
        assert_eq!(do_match("*.c", "foo/bar.c", 0), 0);
        assert_eq!(do_match("*.c", "foo/bar.c", FNM_PATHNAME), FNM_NOMATCH);
        assert_eq!(do_match("*/*.c", "foo/bar.c", FNM_PATHNAME), 0);
    }

    #[test]
    fn test_fnm_period() {
        assert_eq!(do_match("*.c", ".foo.c", 0), 0);
        assert_eq!(do_match("*.c", ".foo.c", FNM_PERIOD), FNM_NOMATCH);
        assert_eq!(do_match(".*.c", ".foo.c", FNM_PERIOD), 0);
    }

    #[test]
    fn test_fnm_noescape() {
        assert_eq!(do_match("\\*", "*", 0), 0);
        assert_eq!(do_match("\\*", "x", 0), FNM_NOMATCH);
        assert_eq!(do_match("\\*", "\\*", FNM_NOESCAPE), 0);
    }

    #[test]
    fn test_fnm_casefold() {
        assert_eq!(do_match("HELLO", "hello", FNM_CASEFOLD), 0);
        assert_eq!(do_match("hello", "HELLO", FNM_CASEFOLD), 0);
        assert_eq!(do_match("HELLO", "hello", 0), FNM_NOMATCH);
    }

    #[test]
    fn test_null_pointers() {
        assert_eq!(unsafe { fnmatch(std::ptr::null(), b"x\0".as_ptr() as *const c_char, 0) }, FNM_NOMATCH);
        assert_eq!(unsafe { fnmatch(b"x\0".as_ptr() as *const c_char, std::ptr::null(), 0) }, FNM_NOMATCH);
    }
}
