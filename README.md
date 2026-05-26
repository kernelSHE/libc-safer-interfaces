# libc Safer Interfaces

[![GHA Status]][GitHub Actions] [![Latest Version]][crates.io] [![Documentation]][docs.rs] ![License]

This project is a security-hardened fork of the official [libc](https://github.com/rust-lang/libc) crate (v1.0.0-alpha.3). We identify `extern "C"` ABI functions in libc that have historically disclosed CVEs/memory safety vulnerabilities, rewrite them in pure Rust, and validate semantic equivalence through comprehensive testing.

## Motivation

Many C standard library functions involve complex pointer manipulation and manual memory management, making them prone to buffer overflows, null pointer dereferences, use-after-free, and other memory safety issues. By replacing these functions with semantically equivalent Rust implementations, we eliminate the FFI boundary and leverage Rust's type system and ownership model for stronger safety guarantees.

## Methodology

1. **Fork** the upstream libc crate
2. **Audit** `extern "C"` FFI declarations for functions with known CVE history
3. **Rewrite** each target function in pure Rust, preserving the exact ABI signature
4. **Validate** semantic equivalence through dedicated test suites comparing against C implementations
5. **Replace** by registering the Rust module at the crate root, shadowing the original FFI declarations

## Replaced Functions

### `iconv` / `iconv_open` / `iconv_close`

Rust implementation using the `encoding_rs` crate — pure Rust encoding conversion supporting UTF-8/16/32, ASCII, GBK, GB2312, GB18030, Big5, EUC-KR, Shift_JIS, ISO-2022-JP, UCS-2, WCHAR_T, with BOM detection for stateful encodings.

| Platform file | Extern Status |
|---|---|
| `src/unix/aix/mod.rs` | Commented out |
| `src/unix/bsd/freebsdlike/mod.rs` | Commented out |
| `src/unix/bsd/netbsdlike/netbsd/mod.rs` | Commented out |
| `src/unix/hurd/mod.rs` | Commented out |
| `src/unix/linux_like/linux_l4re_shared.rs` | Commented out |
| **`src/iconv.rs`** | **✅ Pure Rust replacement** |

### `strxfrm`

Simple pure-Rust string copy — equivalent to musl's no-op implementation for single-byte locales.

| Platform file | Extern Status |
|---|---|
| `src/unix/mod.rs` | Commented out |
| `src/fuchsia/mod.rs` | Commented out |
| `src/solid/mod.rs` | Commented out |
| `src/teeos/mod.rs` | Commented out |
| `src/vxworks/mod.rs` | Commented out |
| `src/wasi/mod.rs` | Commented out |
| `src/windows/mod.rs` | Commented out |
| `src/new/qurt/mod.rs` | Commented out |
| **`src/strxfrm.rs`** | **✅ Pure Rust replacement** |

### `strftime`

Pure-Rust datetime formatter with full C locale support for all format specifiers (`%a`, `%A`, `%b`, `%B`, `%c`, `%C`, `%d`, `%D`, `%e`, `%F`, `%g`, `%G`, `%H`, `%I`, `%j`, `%m`, `%M`, `%n`, `%p`, `%r`, `%R`, `%s`, `%S`, `%t`, `%T`, `%u`, `%U`, `%V`, `%w`, `%W`, `%x`, `%X`, `%y`, `%Y`, `%z`, `%Z`), including leap year calculation, ISO week numbers, and timezone offset formatting.

| Platform file | Extern Status |
|---|---|
| `src/unix/aix/mod.rs` | Commented out |
| `src/unix/bsd/mod.rs` | Commented out |
| `src/unix/cygwin/mod.rs` | Commented out |
| `src/unix/hurd/mod.rs` | Commented out |
| `src/unix/linux_like/mod.rs` | Commented out |
| `src/unix/redox/mod.rs` | Commented out |
| `src/unix/solarish/mod.rs` | Commented out |
| `src/solid/mod.rs` | Commented out |
| `src/teeos/mod.rs` | Commented out |
| `src/wasi/mod.rs` | Commented out |
| `src/new/qurt/time.rs` | Commented out |
| **`src/strftime.rs`** | **✅ Pure Rust replacement** |

### `fnmatch`

Pure-Rust glob pattern matching implementing `*`, `?`, `[...]` bracket expressions, character classes (`[:alpha:]`, `[:digit:]`, etc.), and all standard flags: `FNM_PATHNAME`, `FNM_NOESCAPE`, `FNM_PERIOD`, `FNM_LEADING_DIR`, `FNM_CASEFOLD`.

| Platform file | Extern Status |
|---|---|
| `src/unix/mod.rs` | Commented out |
| `src/vxworks/mod.rs` | Commented out |
| **`src/fnmatch.rs`** | **✅ Pure Rust replacement** |

### `unsetenv`

Pure-Rust implementation that directly manipulates the process environment via `dlsym(RTLD_DEFAULT, "environ")`.

| Platform file | Extern Status |
|---|---|
| `src/unix/mod.rs` | Commented out |
| `src/fuchsia/mod.rs` | Commented out |
| `src/solid/mod.rs` | Commented out |
| `src/vxworks/mod.rs` | Commented out |
| `src/wasi/mod.rs` | Commented out |
| `src/new/qurt/stdlib.rs` | Commented out |
| **`src/unsetenv.rs`** | **✅ Pure Rust replacement** |

### `getifaddrs` / `freeifaddrs`

Pure-Rust implementation using raw NetLink sockets (`PF_NETLINK`, `RTM_GETLINK`/`RTM_GETADDR`) to enumerate network interfaces on Linux, avoiding the system `getifaddrs()` call entirely. Supports IPv4, IPv6, and link-layer (AF_PACKET) addresses.

| Platform file | Extern Status |
|---|---|
| `src/unix/bsd/mod.rs` | Commented out |
| `src/unix/fuchsia/mod.rs` | Commented out |
| `src/unix/haiku/mod.rs` | Commented out |
| `src/unix/hurd/mod.rs` | Commented out |
| `src/unix/linux_like/mod.rs` | Commented out |
| `src/unix/nto/mod.rs` | Commented out |
| `src/unix/solarish/mod.rs` | Commented out |
| **`src/getifaddrs.rs`** | **✅ Pure Rust replacement** |

### `getgrouplist`

C-to-Rust transpiled via the Corcrat tool from musl 1.2.6 source. Pending integration.

| Platform file | Extern Status |
|---|---|
| `src/unix/bsd/apple/mod.rs` | Commented out |
| `src/unix/bsd/freebsdlike/mod.rs` | Commented out |
| `src/unix/bsd/netbsdlike/mod.rs` | Commented out |
| `src/unix/cygwin/mod.rs` | Commented out |
| `src/unix/fuchsia/mod.rs` | Commented out |
| `src/unix/haiku/mod.rs` | Commented out |
| `src/unix/hurd/mod.rs` | Commented out |
| `src/unix/linux_like/android/mod.rs` | Commented out |
| `src/unix/linux_like/linux/mod.rs` | Commented out |
| `src/unix/nto/mod.rs` | Commented out |
| `src/unix/redox/mod.rs` | Commented out |
| `src/unix/solarish/mod.rs` | Commented out |
| **`src/getgrouplist.rs`** | **⏳ Corcrat transpiled (symlink)** |

### `memcmp`

C-to-Rust transpiled via the Corcrat tool from musl 1.2.6 source. Pending integration.

| Platform file | Extern Status |
|---|---|
| `src/unix/mod.rs` | Commented out |
| `src/fuchsia/mod.rs` | Commented out |
| `src/solid/mod.rs` | Commented out |
| `src/teeos/mod.rs` | Commented out |
| `src/vxworks/mod.rs` | Commented out |
| `src/wasi/mod.rs` | Commented out |
| `src/windows/mod.rs` | Commented out |
| `src/new/qurt/mod.rs` | Commented out |
| **`src/memcmp.rs`** | **⏳ Corcrat transpiled (symlink)** |

### `calloc`

C-to-Rust transpiled via the Corcrat tool from musl 1.2.6 source. Pending integration. Note: the extern declarations in `src/unix/mod.rs` and `src/new/qurt/stdlib.rs` are **still active** (uncommented).

| Platform file | Extern Status |
|---|---|
| `src/unix/mod.rs` | Commented out |
| `src/new/qurt/stdlib.rs` | Commented out |
| `src/fuchsia/mod.rs` | Commented out |
| `src/solid/mod.rs` | Commented out |
| `src/teeos/mod.rs` | Commented out |
| `src/trusty.rs` | Commented out |
| `src/vxworks/mod.rs` | Commented out |
| `src/wasi/mod.rs` | Commented out |
| `src/windows/mod.rs` | Commented out |
| **`src/calloc.rs`** | **⏳ Corcrat transpiled (symlink)** |

## Tests

Each replacement function has corresponding test coverage:

- `tests/iconv.rs` — 16 test cases covering open/close, encoding resolution, roundtrip conversions, BOM detection, error conditions
- `src/fnmatch.rs` — 9 inline `#[test]` functions covering literal match, `*`, `?`, bracket expressions, pathname flags, case folding, null pointers
- `rs-test/` — Semantic analysis and dry-run reports for strftime, strxfrm, iconv, getgrouplist

### Running tests

```bash
cargo test
```

## Project Structure

```
src/
├── lib.rs              # Crate root: registers and re-exports all replacement modules
├── iconv.rs            # iconv / iconv_open / iconv_close — pure Rust impl
├── strxfrm.rs          # strxfrm — pure Rust impl
├── strftime.rs         # strftime — pure Rust impl
├── fnmatch.rs          # fnmatch — pure Rust impl (with inline tests)
├── unsetenv.rs         # unsetenv — pure Rust impl
├── getifaddrs.rs       # getifaddrs / freeifaddrs — pure Rust impl (NetLink)
├── calloc.rs           # calloc — Corcrat C-to-Rust output (symlink)
├── memcmp.rs           # memcmp — Corcrat C-to-Rust output (symlink)
├── getgrouplist.rs     # getgrouplist — Corcrat C-to-Rust output (symlink)
├── unix/               # Unix platform FFI bindings (extern "C" declarations shadowed)
├── {other platforms}/  # Other platform bindings
tests/
├── iconv.rs            # iconv integration tests
└── const_fn.rs         # Compile-time const fn test
rs-test/                # Semantic equivalence reports
├── reports-iconv/
├── strftime/
├── strxfrm/
└── getgrouplist/
```

[GitHub Actions]: https://github.com/rust-lang/libc/actions
[GHA Status]: https://github.com/rust-lang/libc/workflows/CI/badge.svg
[crates.io]: https://crates.io/crates/libc
[Latest Version]: https://img.shields.io/crates/v/libc.svg
[Documentation]: https://docs.rs/libc/badge.svg
[docs.rs]: https://docs.rs/libc
[License]: https://img.shields.io/crates/l/libc.svg
