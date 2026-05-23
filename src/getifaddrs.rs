use core::mem::{size_of, zeroed};

use libc::{
    c_char, c_int, c_uchar, c_uint, c_ushort, c_void, free, calloc, close,
    recv, send, socket, sockaddr, sockaddr_in, sockaddr_in6, sa_family_t,
    ifaddrs, ifinfomsg, in6_addr, nlmsghdr,
    AF_INET, AF_INET6, AF_PACKET, AF_UNSPEC,
    MSG_DONTWAIT, PF_NETLINK, RTM_GETADDR, RTM_GETLINK, RTM_NEWLINK,
    SOCK_CLOEXEC, SOCK_RAW, IFNAMSIZ,
};

const IFADDRS_HASH_SIZE: usize = 64;

#[repr(C)]
#[derive(Copy, Clone)]
struct sockaddr_ll_hack {
    sll_family: c_ushort,
    sll_protocol: c_ushort,
    sll_ifindex: c_int,
    sll_hatype: c_ushort,
    sll_pkttype: c_uchar,
    sll_halen: c_uchar,
    sll_addr: [c_uchar; 24],
}

impl Default for sockaddr_ll_hack {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
union sockany {
    sa: sockaddr,
    ll: sockaddr_ll_hack,
    v4: sockaddr_in,
    v6: sockaddr_in6,
}

impl Default for sockany {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[repr(C)]
struct ifaddrs_storage {
    ifa: ifaddrs,
    hash_next: *mut ifaddrs_storage,
    addr: sockany,
    netmask: sockany,
    ifu: sockany,
    index: c_uint,
    name: [c_char; IFNAMSIZ as usize + 1],
}

#[repr(C)]
struct ifaddrs_ctx {
    first: *mut ifaddrs,
    last: *mut ifaddrs,
    hash: [*mut ifaddrs_storage; IFADDRS_HASH_SIZE],
}

impl Default for ifaddrs_ctx {
    fn default() -> Self {
        Self {
            first: core::ptr::null_mut(),
            last: core::ptr::null_mut(),
            hash: [core::ptr::null_mut(); IFADDRS_HASH_SIZE],
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct rtgenmsg {
    rtgen_family: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct rtattr {
    rta_len: c_ushort,
    rta_type: c_ushort,
}

impl Default for rtattr {
    fn default() -> Self {
        Self { rta_len: 0, rta_type: 0 }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ifaddrmsg {
    ifa_family: c_uchar,
    ifa_prefixlen: c_uchar,
    ifa_flags: c_uchar,
    ifa_scope: c_uchar,
    ifa_index: c_uint,
}

impl Default for ifaddrmsg {
    fn default() -> Self {
        Self {
            ifa_family: 0,
            ifa_prefixlen: 0,
            ifa_flags: 0,
            ifa_scope: 0,
            ifa_index: 0,
        }
    }
}

const NLM_F_REQUEST: u16 = 1;
const NLM_F_ROOT: u16 = 0x100;
const NLM_F_MATCH: u16 = 0x200;
const NLM_F_DUMP: u16 = NLM_F_ROOT | NLM_F_MATCH;
const NLMSG_DONE: u16 = 3;
const NLMSG_ERROR: u16 = 2;

const IFLA_ADDRESS: c_ushort = 1;
const IFLA_BROADCAST: c_ushort = 2;
const IFLA_IFNAME: c_ushort = 3;
const IFLA_STATS: c_ushort = 7;
const IFA_ADDRESS: c_ushort = 1;
const IFA_LOCAL: c_ushort = 2;
const IFA_LABEL: c_ushort = 3;
const IFA_BROADCAST: c_ushort = 4;

#[inline]
fn nlmsg_align_u32(len: u32) -> u32 {
    (len + 3) & !3
}

#[inline]
fn nlmsg_align_usize(len: usize) -> usize {
    (len + 3) & !3
}

#[inline]
fn nlmsg_ok(h: *const nlmsghdr, end: *const u8) -> bool {
    unsafe { end.offset_from(h as *const u8) >= size_of::<nlmsghdr>() as isize }
}

#[inline]
fn nlmsg_next(h: *const nlmsghdr) -> *const nlmsghdr {
    unsafe {
        let len = (*h).nlmsg_len;
        (h as *const u8).add(nlmsg_align_u32(len) as usize) as *const nlmsghdr
    }
}

#[inline]
fn nlmsg_data(h: *mut nlmsghdr) -> *mut u8 {
    unsafe { (h as *mut u8).add(size_of::<nlmsghdr>()) }
}

#[inline]
fn nlmsg_dataend(h: *mut nlmsghdr) -> *const u8 {
    unsafe { (h as *const u8).add((*h).nlmsg_len as usize) }
}

#[inline]
fn nlmsg_rta(h: *mut nlmsghdr, offset: usize) -> *mut rtattr {
    unsafe { nlmsg_data(h).add(nlmsg_align_usize(offset)) as *mut rtattr }
}

#[inline]
fn rta_data(rta: *const rtattr) -> *mut u8 {
    unsafe { (rta as *const u8).add(size_of::<rtattr>()) as *mut u8 }
}

#[inline]
fn rta_datalen(rta: *const rtattr) -> usize {
    unsafe { (*rta).rta_len as usize - size_of::<rtattr>() }
}

#[inline]
fn rta_next(rta: *const rtattr) -> *mut rtattr {
    unsafe { (rta as *const u8).add(nlmsg_align_usize((*rta).rta_len as usize)) as *mut rtattr }
}

#[inline]
fn rta_ok(rta: *const rtattr, end: *const u8) -> bool {
    unsafe {
        end.offset_from(rta as *const u8) >= size_of::<rtattr>() as isize
            && (*rta).rta_len as usize >= size_of::<rtattr>()
    }
}

#[inline]
fn in6_is_addr_linklocal(a: *const in6_addr) -> bool {
    unsafe {
        let b = a as *const u8;
        *b == 0xfe && *b.add(1) & 0xc0 == 0x80
    }
}

#[inline]
fn in6_is_addr_mc_linklocal(a: *const in6_addr) -> bool {
    unsafe {
        let b = a as *const u8;
        *b == 0xff && *b.add(1) & 0x0f == 0x02
    }
}

unsafe fn copy_addr(
    r: *mut *mut sockaddr,
    af: c_int,
    sa: *mut sockany,
    addr: *const u8,
    addrlen: usize,
    ifindex: c_int,
) {
    match af {
        AF_INET => {
            let dst = &mut (*sa).v4.sin_addr as *mut _ as *mut u8;
            let len = 4usize;
            if addrlen < len {
                return;
            }
            (*sa).sa.sa_family = af as sa_family_t;
            core::ptr::copy_nonoverlapping(addr, dst, len);
            *r = &mut (*sa).sa;
        }
        AF_INET6 => {
            let dst = &mut (*sa).v6.sin6_addr as *mut _ as *mut u8;
            let len = 16usize;
            if addrlen < len {
                return;
            }
            (*sa).sa.sa_family = af as sa_family_t;
            core::ptr::copy_nonoverlapping(addr, dst, len);
            if in6_is_addr_linklocal(addr as *const in6_addr)
                || in6_is_addr_mc_linklocal(addr as *const in6_addr)
            {
                (*sa).v6.sin6_scope_id = ifindex as u32;
            }
            *r = &mut (*sa).sa;
        }
        _ => {}
    }
}

unsafe fn gen_netmask(r: *mut *mut sockaddr, af: c_int, sa: *mut sockany, prefixlen: c_int) {
    let mut addr: [u8; 16] = [0u8; 16];
    let prefixlen = if prefixlen > 128 { 128 } else { prefixlen };
    let i = (prefixlen / 8) as usize;
    for j in 0..i {
        addr[j] = 0xff;
    }
    if i < 16 && prefixlen % 8 != 0 {
        addr[i] = 0xffu8 << (8 - (prefixlen % 8));
    }
    copy_addr(r, af, sa, addr.as_ptr(), 16, 0);
}

unsafe fn copy_lladdr(
    r: *mut *mut sockaddr,
    sa: *mut sockany,
    addr: *const u8,
    addrlen: usize,
    ifindex: c_int,
    hatype: c_ushort,
) {
    if addrlen > (*sa).ll.sll_addr.len() {
        return;
    }
    (*sa).ll.sll_family = AF_PACKET as c_ushort;
    (*sa).ll.sll_ifindex = ifindex;
    (*sa).ll.sll_hatype = hatype;
    (*sa).ll.sll_halen = addrlen as c_uchar;
    core::ptr::copy_nonoverlapping(addr, (*sa).ll.sll_addr.as_mut_ptr(), addrlen);
    *r = &mut (*sa).sa;
}

unsafe fn __netlink_enumerate(
    fd: c_int,
    seq: u32,
    msg_type: c_int,
    af: c_int,
    cb: Option<unsafe fn(*mut c_void, *mut nlmsghdr) -> c_int>,
    ctx: *mut c_void,
) -> c_int {
    const BUF_SIZE: usize = 8192;
    let req_size = size_of::<nlmsghdr>() + size_of::<rtgenmsg>();

    let mut buf = [0u8; BUF_SIZE];

    let req = buf.as_mut_ptr() as *mut nlmsghdr;
    (*req).nlmsg_len = req_size as u32;
    (*req).nlmsg_type = msg_type as u16;
    (*req).nlmsg_flags = NLM_F_DUMP | NLM_F_REQUEST;
    (*req).nlmsg_seq = seq;
    (*req).nlmsg_pid = 0;

    let gen = (req as *mut u8).add(size_of::<nlmsghdr>()) as *mut rtgenmsg;
    (*gen).rtgen_family = af as u8;

    let r = send(fd, req as *const c_void, req_size, 0);
    if r < 0 {
        return r as c_int;
    }

    loop {
        let r = recv(fd, buf.as_mut_ptr() as *mut c_void, BUF_SIZE, MSG_DONTWAIT);
        if r <= 0 {
            return -1;
        }
        let end = buf.as_ptr().add(r as usize);
        let mut h = buf.as_ptr() as *const nlmsghdr;
        while nlmsg_ok(h, end) {
            let msg_type_h = (*h).nlmsg_type;
            if msg_type_h == NLMSG_DONE {
                return 0;
            }
            if msg_type_h == NLMSG_ERROR {
                return -1;
            }
            let cb_fn = match cb {
                Some(f) => f,
                None => return -1,
            };
            let ret = cb_fn(ctx, h as *mut nlmsghdr);
            if ret != 0 {
                return ret;
            }
            h = nlmsg_next(h);
        }
    }
}

type NetlinkCallback = Option<unsafe fn(*mut c_void, *mut nlmsghdr) -> c_int>;

unsafe fn __rtnetlink_enumerate(
    link_af: c_int,
    addr_af: c_int,
    cb: NetlinkCallback,
    ctx: *mut c_void,
) -> c_int {
    let fd = socket(PF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, 0);
    if fd < 0 {
        return -1;
    }
    let mut r = __netlink_enumerate(fd, 1, RTM_GETLINK as c_int, link_af, cb, ctx);
    if r == 0 {
        r = __netlink_enumerate(fd, 2, RTM_GETADDR as c_int, addr_af, cb, ctx);
    }
    close(fd);
    r
}

unsafe fn netlink_msg_to_ifaddr(pctx: *mut c_void, h: *mut nlmsghdr) -> c_int {
    let ctx = pctx as *mut ifaddrs_ctx;
    let mut stats_len: usize = 0;
    let msg_type = (*h).nlmsg_type;

    if msg_type == RTM_NEWLINK as u16 {
        let _ifi = nlmsg_data(h) as *const ifinfomsg;
        let mut rta = nlmsg_rta(h, size_of::<ifinfomsg>());
        let end = nlmsg_dataend(h);
        while rta_ok(rta, end) {
            if (*rta).rta_type == IFLA_STATS {
                stats_len = rta_datalen(rta);
                break;
            }
            rta = rta_next(rta);
        }
    } else {
        let ifa_msg = nlmsg_data(h) as *const ifaddrmsg;
        let idx = (*ifa_msg).ifa_index as c_uint;
        let mut ifs0: *mut ifaddrs_storage = (*ctx).hash[(idx as usize) % IFADDRS_HASH_SIZE];
        while !ifs0.is_null() {
            if (*ifs0).index == idx {
                break;
            }
            ifs0 = (*ifs0).hash_next;
        }
        if ifs0.is_null() {
            return 0;
        }
    }

    let ifs: *mut ifaddrs_storage =
        calloc(1, size_of::<ifaddrs_storage>() + stats_len) as *mut ifaddrs_storage;
    if ifs.is_null() {
        return -1;
    }

    if msg_type == RTM_NEWLINK as u16 {
        let ifi = nlmsg_data(h) as *const ifinfomsg;
        (*ifs).index = (*ifi).ifi_index as c_uint;
        (*ifs).ifa.ifa_flags = (*ifi).ifi_flags;

        let mut rta = nlmsg_rta(h, size_of::<ifinfomsg>());
        let end = nlmsg_dataend(h);
        while rta_ok(rta, end) {
            match (*rta).rta_type {
                IFLA_IFNAME => {
                    let dlen = rta_datalen(rta);
                    if dlen < (*ifs).name.len() {
                        core::ptr::copy_nonoverlapping(
                            rta_data(rta),
                            (*ifs).name.as_mut_ptr() as *mut u8,
                            dlen,
                        );
                        (*ifs).ifa.ifa_name = (*ifs).name.as_mut_ptr();
                    }
                }
                IFLA_ADDRESS => {
                    copy_lladdr(
                        &mut (*ifs).ifa.ifa_addr,
                        &mut (*ifs).addr,
                        rta_data(rta),
                        rta_datalen(rta),
                        (*ifi).ifi_index,
                        (*ifi).ifi_type,
                    );
                }
                IFLA_BROADCAST => {
                    copy_lladdr(
                        &mut (*ifs).ifa.ifa_ifu,
                        &mut (*ifs).ifu,
                        rta_data(rta),
                        rta_datalen(rta),
                        (*ifi).ifi_index,
                        (*ifi).ifi_type,
                    );
                }
                IFLA_STATS => {
                    let data_ptr = ifs.add(1) as *mut c_void;
                    (*ifs).ifa.ifa_data = data_ptr;
                    core::ptr::copy_nonoverlapping(
                        rta_data(rta) as *const c_void,
                        data_ptr,
                        rta_datalen(rta),
                    );
                }
                _ => {}
            }
            rta = rta_next(rta);
        }

        if !(*ifs).ifa.ifa_name.is_null() {
            let bucket = (*ifs).index as usize % IFADDRS_HASH_SIZE;
            (*ifs).hash_next = (*ctx).hash[bucket];
            (*ctx).hash[bucket] = ifs;
        }
    } else {
        let ifa_msg = nlmsg_data(h) as *const ifaddrmsg;
        let idx = (*ifa_msg).ifa_index as c_uint;
        let mut ifs0: *mut ifaddrs_storage = (*ctx).hash[(idx as usize) % IFADDRS_HASH_SIZE];
        while !ifs0.is_null() {
            if (*ifs0).index == idx {
                break;
            }
            ifs0 = (*ifs0).hash_next;
        }

        (*ifs).ifa.ifa_name = (*ifs0).ifa.ifa_name;
        (*ifs).ifa.ifa_flags = (*ifs0).ifa.ifa_flags;

        let mut rta = nlmsg_rta(h, size_of::<ifaddrmsg>());
        let end = nlmsg_dataend(h);
        while rta_ok(rta, end) {
            match (*rta).rta_type {
                IFA_ADDRESS => {
                    if !(*ifs).ifa.ifa_addr.is_null() {
                        copy_addr(
                            &mut (*ifs).ifa.ifa_ifu,
                            (*ifa_msg).ifa_family as c_int,
                            &mut (*ifs).ifu,
                            rta_data(rta),
                            rta_datalen(rta),
                            (*ifa_msg).ifa_index as c_int,
                        );
                    } else {
                        copy_addr(
                            &mut (*ifs).ifa.ifa_addr,
                            (*ifa_msg).ifa_family as c_int,
                            &mut (*ifs).addr,
                            rta_data(rta),
                            rta_datalen(rta),
                            (*ifa_msg).ifa_index as c_int,
                        );
                    }
                }
                IFA_BROADCAST => {
                    copy_addr(
                        &mut (*ifs).ifa.ifa_ifu,
                        (*ifa_msg).ifa_family as c_int,
                        &mut (*ifs).ifu,
                        rta_data(rta),
                        rta_datalen(rta),
                        (*ifa_msg).ifa_index as c_int,
                    );
                }
                IFA_LOCAL => {
                    if !(*ifs).ifa.ifa_addr.is_null() {
                        (*ifs).ifu = (*ifs).addr;
                        (*ifs).ifa.ifa_ifu = &mut (*ifs).ifu.sa;
                        (*ifs).addr = zeroed();
                    }
                    copy_addr(
                        &mut (*ifs).ifa.ifa_addr,
                        (*ifa_msg).ifa_family as c_int,
                        &mut (*ifs).addr,
                        rta_data(rta),
                        rta_datalen(rta),
                        (*ifa_msg).ifa_index as c_int,
                    );
                }
                IFA_LABEL => {
                    let dlen = rta_datalen(rta);
                    if dlen < (*ifs).name.len() {
                        core::ptr::copy_nonoverlapping(
                            rta_data(rta),
                            (*ifs).name.as_mut_ptr() as *mut u8,
                            dlen,
                        );
                        (*ifs).ifa.ifa_name = (*ifs).name.as_mut_ptr();
                    }
                }
                _ => {}
            }
            rta = rta_next(rta);
        }

        if !(*ifs).ifa.ifa_addr.is_null() {
            gen_netmask(
                &mut (*ifs).ifa.ifa_netmask,
                (*ifa_msg).ifa_family as c_int,
                &mut (*ifs).netmask,
                (*ifa_msg).ifa_prefixlen as c_int,
            );
        }
    }

    if !(*ifs).ifa.ifa_name.is_null() {
        if (*ctx).first.is_null() {
            (*ctx).first = &mut (*ifs).ifa;
        }
        if !(*ctx).last.is_null() {
            (*(*ctx).last).ifa_next = &mut (*ifs).ifa;
        }
        (*ctx).last = &mut (*ifs).ifa;
    } else {
        free(ifs as *mut c_void);
    }

    0
}

#[no_mangle]
pub unsafe fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int {
    let mut _ctx: ifaddrs_ctx = Default::default();
    let ctx: *mut ifaddrs_ctx = &mut _ctx as *mut ifaddrs_ctx;
    let cb: Option<unsafe fn(*mut c_void, *mut nlmsghdr) -> c_int> =
        Some(netlink_msg_to_ifaddr);
    let r = __rtnetlink_enumerate(AF_UNSPEC as c_int, AF_UNSPEC as c_int, cb, ctx as *mut c_void);
    if r == 0 {
        *ifap = (*ctx).first;
    } else {
        freeifaddrs((*ctx).first);
    }
    r
}

#[no_mangle]
pub unsafe fn freeifaddrs(mut ifp: *mut ifaddrs) {
    let mut n: *mut ifaddrs;
    while !ifp.is_null() {
        n = (*ifp).ifa_next as *mut ifaddrs;
        free(ifp as *mut c_void);
        ifp = n;
    }
}
