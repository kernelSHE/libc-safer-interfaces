use core::ffi::c_void;
use core::ptr;

#[repr(C)]
#[derive(Debug)]
pub struct StrftimeTm {
    pub tm_sec: i32,
    pub tm_min: i32,
    pub tm_hour: i32,
    pub tm_mday: i32,
    pub tm_mon: i32,
    pub tm_year: i32,
    pub tm_wday: i32,
    pub tm_yday: i32,
    pub tm_isdst: i32,
    pub __tm_gmtoff: i64,
    pub __tm_zone: *const u8,
}

const ABDAY_1: u32 = 0x20000;
const DAY_1: u32 = 0x20007;
const ABMON_1: u32 = 0x2000E;
const MON_1: u32 = 0x2001A;
const AM_STR: u32 = 0x20026;
const PM_STR: u32 = 0x20027;
const D_T_FMT: u32 = 0x20028;
const D_FMT: u32 = 0x20029;
const T_FMT: u32 = 0x2002A;
const T_FMT_AMPM: u32 = 0x2002B;

const C_TIME: &[u8] = b"\
Sun\0Mon\0Tue\0Wed\0Thu\0Fri\0Sat\0\
Sunday\0Monday\0Tuesday\0Wednesday\0Thursday\0Friday\0Saturday\0\
Jan\0Feb\0Mar\0Apr\0May\0Jun\0Jul\0Aug\0Sep\0Oct\0Nov\0Dec\0\
January\0February\0March\0April\0May\0June\0July\0August\0September\0October\0November\0December\0\
AM\0PM\0\
%a %b %e %T %Y\0\
%m/%d/%y\0\
%H:%M:%S\0\
%I:%M:%S %p\0";

unsafe fn nl_langinfo_l(item: u32, _loc: *const c_void) -> *const u8 {
    let cat = (item >> 16) as usize;
    let idx = (item & 0xFFFF) as usize;
    if cat != 2 || idx > 0x31 {
        return b"\0".as_ptr();
    }
    let mut pos = 0usize;
    for _ in 0..idx {
        while pos < C_TIME.len() && C_TIME[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    C_TIME.as_ptr().add(pos)
}

unsafe fn my_strlen(s: *const u8) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

fn is_leap(y: i32) -> bool {
    let y = if y > i32::MAX - 1900 { y - 2000 } else { y };
    let y = y + 1900;
    (y % 4 == 0) && ((y % 100 != 0) || (y % 400 == 0))
}

unsafe fn week_num(t: *const StrftimeTm) -> i32 {
    let yday = (*t).tm_yday;
    let wday = (*t).tm_wday;
    let mut val = (yday + 7 - (wday + 6) % 7) / 7;
    if (wday + 371 - yday - 2) % 7 <= 2 {
        val += 1;
    }
    if val == 0 {
        val = 52;
        let dec31 = (wday + 7 - yday - 1) % 7;
        if dec31 == 4 || (dec31 == 5 && is_leap((*t).tm_year % 400 - 1)) {
            val += 1;
        }
    } else if val == 53 {
        let jan1 = (wday + 371 - yday) % 7;
        if jan1 != 4 && (jan1 != 3 || !is_leap((*t).tm_year)) {
            val = 1;
        }
    }
    val
}

fn year_to_secs(year: i64, is_leap_out: &mut i32) -> i64 {
    if (year as u64).wrapping_sub(2) <= 136 {
        let y = year as i32;
        let leaps = (y - 68) >> 2;
        if (y - 68) & 3 == 0 {
            *is_leap_out = 1;
            31536000i64 * (y as i64 - 70) + 86400i64 * (leaps - 1) as i64
        } else {
            *is_leap_out = 0;
            31536000i64 * (y as i64 - 70) + 86400i64 * leaps as i64
        }
    } else {
        let mut cycles = ((year - 100) / 400) as i32;
        let mut rem = ((year - 100) % 400) as i32;
        if rem < 0 {
            cycles -= 1;
            rem += 400;
        }
        let (centuries, leaps);
        if rem == 0 {
            *is_leap_out = 1;
            centuries = 0;
            leaps = 0;
        } else {
            let (c2, r2);
            if rem >= 200 {
                if rem >= 300 { c2 = 3; r2 = rem - 300; }
                else { c2 = 2; r2 = rem - 200; }
            } else {
                if rem >= 100 { c2 = 1; r2 = rem - 100; }
                else { c2 = 0; r2 = rem; }
            }
            centuries = c2;
            if r2 == 0 { *is_leap_out = 0; leaps = 0; }
            else {
                let l2 = r2 / 4;
                let r3 = r2 % 4;
                *is_leap_out = if r3 != 0 { 0 } else { 1 };
                leaps = l2;
            }
        }
        let total_leaps = leaps + 97 * cycles + 24 * centuries - *is_leap_out;
        (year - 100) * 31536000 + total_leaps as i64 * 86400 + 946684800 + 86400
    }
}

fn month_to_secs(month: i32, is_leap: i32) -> i64 {
    static SECS: [i64; 12] = [0, 31*86400, 59*86400, 90*86400, 120*86400, 151*86400, 181*86400, 212*86400, 243*86400, 273*86400, 304*86400, 334*86400];
    let mut t = SECS[month as usize];
    if is_leap != 0 && month >= 2 { t += 86400; }
    t
}

unsafe fn tm_to_secs(t: *const StrftimeTm) -> i64 {
    let mut year = (*t).tm_year as i64;
    let mut month = (*t).tm_mon;
    if month >= 12 || month < 0 {
        let adj = month / 12;
        month %= 12;
        if month < 0 { month += 12; year -= 1; }
        year += adj as i64;
    }
    let mut il = 0;
    let secs = year_to_secs(year, &mut il);
    secs + month_to_secs(month, il)
        + 86400 * ((*t).tm_mday as i64 - 1)
        + 3600 * (*t).tm_hour as i64
        + 60 * (*t).tm_min as i64
        + (*t).tm_sec as i64
}

unsafe fn tm_to_tzname(t: *const StrftimeTm) -> *const u8 {
    if !(*t).__tm_zone.is_null() { (*t).__tm_zone } else { b"UTC\0".as_ptr() }
}

fn format_i64(buf: &mut [u8], val: i64) -> usize {
    if val == 0 { if !buf.is_empty() { buf[0] = b'0'; } return 1; }
    let negative = val < 0;
    let mut uval = if negative { (-val) as u64 } else { val as u64 };
    let mut tmp = [0u8; 20];
    let mut pos = 20usize;
    while uval > 0 && pos > 0 { pos -= 1; tmp[pos] = b'0' + (uval % 10) as u8; uval /= 10; }
    let nd = 20 - pos;
    let mut out = 0;
    if negative { if out < buf.len() { buf[out] = b'-'; } out += 1; }
    for i in 0..nd { if out < buf.len() { buf[out] = tmp[pos + i]; } out += 1; }
    out
}

fn format_padded(buf: &mut [u8], val: i64, width: usize, pad: u8) -> usize {
    let negative = val < 0;
    let mut uval = if negative { (-val) as u64 } else { val as u64 };
    let mut tmp = [0u8; 20]; let mut pos = 20usize;
    if uval == 0 { pos -= 1; tmp[pos] = b'0'; }
    else { while uval > 0 && pos > 0 { pos -= 1; tmp[pos] = b'0' + (uval%10) as u8; uval /= 10; } }
    let nd = 20 - pos;
    let ep = if pad != 0 { pad } else { b'0' };
    let mut out = 0;
    if negative { if out < buf.len() { buf[out] = b'-'; } out += 1; }
    if ep != b'-' {
        let fill = if ep == b'_' { b' ' } else { b'0' };
        for _ in 0..width.saturating_sub(nd) { if out < buf.len() { buf[out] = fill; } out += 1; }
    }
    for i in 0..nd { if out < buf.len() { buf[out] = tmp[pos + i]; } out += 1; }
    out
}

unsafe fn strftime_fmt_1(s: &mut [u8; 100], l: &mut usize, f: u8, tm: *const StrftimeTm, loc: *const c_void, pad: u8) -> *const u8 {
    match f {
        b'a' => { if (*tm).tm_wday>6{return b"\0".as_ptr();} let p=nl_langinfo_l(ABDAY_1+(*tm).tm_wday as u32,loc); *l=my_strlen(p); p }
        b'A' => { if (*tm).tm_wday>6{return b"\0".as_ptr();} let p=nl_langinfo_l(DAY_1+(*tm).tm_wday as u32,loc); *l=my_strlen(p); p }
        b'h'|b'b' => { if (*tm).tm_mon>11{return b"\0".as_ptr();} let p=nl_langinfo_l(ABMON_1+(*tm).tm_mon as u32,loc); *l=my_strlen(p); p }
        b'B' => { if (*tm).tm_mon>11{return b"\0".as_ptr();} let p=nl_langinfo_l(MON_1+(*tm).tm_mon as u32,loc); *l=my_strlen(p); p }
        b'c' => { let fmt=nl_langinfo_l(D_T_FMT,loc); *l=strftime_l(s.as_mut_ptr(),100,fmt,tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'C' => { *l=format_padded(s.as_mut_slice(),(1900i64+(*tm).tm_year as i64)/100,2,pad); s.as_ptr() }
        b'e' => { *l=format_padded(s.as_mut_slice(),(*tm).tm_mday as i64,2,if pad!=0{pad}else{b' '}); s.as_ptr() }
        b'd' => { *l=format_padded(s.as_mut_slice(),(*tm).tm_mday as i64,2,pad); s.as_ptr() }
        b'D' => { *l=strftime_l(s.as_mut_ptr(),100,b"%m/%d/%y\0".as_ptr(),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'F' => { *l=strftime_l(s.as_mut_ptr(),100,b"%Y-%m-%d\0".as_ptr(),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'g' => { let mut v=(*tm).tm_year as i64+1900; if (*tm).tm_yday<3&&week_num(tm)!=1{v-=1}else if (*tm).tm_yday>360&&week_num(tm)==1{v+=1} *l=format_padded(s.as_mut_slice(),((v%100)+100)%100,2,pad); s.as_ptr() }
        b'G' => { let mut v=(*tm).tm_year as i64+1900; if (*tm).tm_yday<3&&week_num(tm)!=1{v-=1}else if (*tm).tm_yday>360&&week_num(tm)==1{v+=1} *l=format_padded(s.as_mut_slice(),v,4,pad); s.as_ptr() }
        b'H' => { *l=format_padded(s.as_mut_slice(),(*tm).tm_hour as i64,2,pad); s.as_ptr() }
        b'I' => { let mut v=(*tm).tm_hour as i64; if v==0{v=12}else if v>12{v-=12} *l=format_padded(s.as_mut_slice(),v,2,pad); s.as_ptr() }
        b'j' => { *l=format_padded(s.as_mut_slice(),((*tm).tm_yday+1) as i64,3,pad); s.as_ptr() }
        b'm' => { *l=format_padded(s.as_mut_slice(),((*tm).tm_mon+1) as i64,2,pad); s.as_ptr() }
        b'M' => { *l=format_padded(s.as_mut_slice(),(*tm).tm_min as i64,2,pad); s.as_ptr() }
        b'n' => { *l=1; b"\n".as_ptr() }
        b'p' => { let p=nl_langinfo_l(if (*tm).tm_hour>=12{PM_STR}else{AM_STR},loc); *l=my_strlen(p); p }
        b'r' => { *l=strftime_l(s.as_mut_ptr(),100,nl_langinfo_l(T_FMT_AMPM,loc),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'R' => { *l=strftime_l(s.as_mut_ptr(),100,b"%H:%M\0".as_ptr(),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b's' => { *l=format_i64(s.as_mut_slice(),tm_to_secs(tm)-(*tm).__tm_gmtoff); s.as_ptr() }
        b'S' => { *l=format_padded(s.as_mut_slice(),(*tm).tm_sec as i64,2,pad); s.as_ptr() }
        b't' => { *l=1; b"\t".as_ptr() }
        b'T' => { *l=strftime_l(s.as_mut_ptr(),100,b"%H:%M:%S\0".as_ptr(),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'u' => { *l=format_padded(s.as_mut_slice(),if (*tm).tm_wday!=0{(*tm).tm_wday as i64}else{7},1,pad); s.as_ptr() }
        b'U' => { *l=format_padded(s.as_mut_slice(),((*tm).tm_yday+7-(*tm).tm_wday) as i64/7,2,pad); s.as_ptr() }
        b'W' => { *l=format_padded(s.as_mut_slice(),((*tm).tm_yday+7-((*tm).tm_wday+6)%7) as i64/7,2,pad); s.as_ptr() }
        b'V' => { *l=format_padded(s.as_mut_slice(),week_num(tm) as i64,2,pad); s.as_ptr() }
        b'w' => { *l=format_padded(s.as_mut_slice(),(*tm).tm_wday as i64,1,pad); s.as_ptr() }
        b'x' => { *l=strftime_l(s.as_mut_ptr(),100,nl_langinfo_l(D_FMT,loc),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'X' => { *l=strftime_l(s.as_mut_ptr(),100,nl_langinfo_l(T_FMT,loc),tm,loc); if *l==0{return ptr::null();} s.as_ptr() }
        b'y' => { let mut v=((*tm).tm_year as i64+1900)%100; if v<0{v=-v} *l=format_padded(s.as_mut_slice(),v,2,pad); s.as_ptr() }
        b'Y' => { let v=(*tm).tm_year as i64+1900; if v>=10000{s[0]=b'+';let n=format_i64(&mut s[1..],v);*l=1+n;return s.as_ptr();} *l=format_padded(s.as_mut_slice(),v,4,pad); s.as_ptr() }
        b'z' => { if (*tm).tm_isdst<0{*l=0;return b"\0".as_ptr();} let off=(*tm).__tm_gmtoff; let v=off/3600*100+off%3600/60; s[0]=if v>=0{b'+'}else{b'-'}; *l=1+format_padded(&mut s[1..],v.abs(),4,b'0'); s.as_ptr() }
        b'Z' => { if (*tm).tm_isdst<0{*l=0;return b"\0".as_ptr();} let p=tm_to_tzname(tm); *l=my_strlen(p); p }
        b'%' => { *l=1; b"%".as_ptr() }
        _ => ptr::null(),
    }
}

unsafe fn strftime_l(s: *mut u8, n: usize, mut f: *const u8, tm: *const StrftimeTm, loc: *const c_void) -> usize {
    let mut l = 0usize;
    let mut buf = [0u8; 100];
    while l < n {
        if *f==0 { *s.add(l)=0; return l; }
        if *f!=b'%' { *s.add(l)=*f; l+=1; f=f.add(1); continue; }
        f=f.add(1);
        let mut pad=0u8;
        if *f==b'-'||*f==b'_'||*f==b'0' { pad=*f; f=f.add(1); }
        let plus=*f==b'+';
        if plus { f=f.add(1); }
        let mut width=0usize; let p: *const u8;
        if *f>=b'0'&&*f<=b'9' { let mut val=0usize; let mut pp=f; while *pp>=b'0'&&*pp<=b'9'{val=val*10+(*pp-b'0') as usize;pp=pp.add(1);} width=val; p=pp; } else { p=f; }
        if *p==b'C'||*p==b'F'||*p==b'G'||*p==b'Y' { if width==0&&p!=f{width=1;} } else { width=0; }
        f=p;
        if *f==b'E'||*f==b'O' { f=f.add(1); }
        let mut k=0usize;
        let t=strftime_fmt_1(&mut buf,&mut k,*f,tm,loc,pad);
        if t.is_null(){break;}
        let mut t=t; let mut k=k;
        if width>0 {
            if *t==b'+'||*t==b'-' { t=t.add(1); k-=1; }
            while *t==b'0'&&*t.add(1)>=b'0'&&*t.add(1)<=b'9' { t=t.add(1); k-=1; }
            if width<k{width=k;}
            let mut d=0usize; while *t.add(d)>=b'0'&&*t.add(d)<=b'9'{d+=1;}
            if (*tm).tm_year < -1900 {*s.add(l)=b'-'; l+=1; width-=1;}
            else if plus&&d+(width-k)>=if *p==b'C'{3}else{5} {*s.add(l)=b'+'; l+=1; width-=1;}
            while width>k&&l<n {*s.add(l)=b'0'; l+=1; width-=1;}
        }
        let copy=if k>n-l{n-l}else{k};
        ptr::copy_nonoverlapping(t,s.add(l),copy);
        l+=copy;
        f=f.add(1);
    }
    if n>0 { if l==n{l=n-1;} *s.add(l)=0; }
    0
}

pub unsafe fn strftime(s: *mut u8, n: usize, f: *const u8, tm: *const StrftimeTm) -> usize {
    strftime_l(s, n, f, tm, ptr::null())
}
