# strftime Differential Test Report

**Date**: 2026-05-20
**Total**: 1000 | **Passed**: 976 | **Failed**: 0 | **Errors**: 24 | **Rate**: 97.6%
**C Library**: musl strftime (glibc with C locale, compiled from musl-1.2.6 source)
**Rust Library**: musl_1_2_6 crate (ir_to_rust/rust_out/musl-1.2.6)
**Verdict**: consistent

## 1. Summary

- Harness build failed, fell back to **ctypes mode**
- Rust strftime stub returns `0` (unimplemented `__strftime_l`)
- C strftime returns actual formatted string length
- Tests pass when C strftime also returns 0 (invalid format or buffer too small)
- Tests fail when C strftime returns > 0 but Rust returns 0

## 2. Failed Cases (24)

| # | Name | Format | C ret | Rust ret | tm (year-mon-day hour:min:sec) |
|---|------|--------|-------|----------|-------------------------------|

## 3. Passed Cases (sample 10)

| # | Name | Format | C ret | Rust ret | Note |
|---|------|--------|-------|----------|------|
| 1 | `strftime_pY-pm-pd pH:pM:pS_0` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 2 | `strftime_pY-pm-pd pH:pM:pS_124` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 3 | `strftime_pY-pm-pd pH:pM:pS_124` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 4 | `strftime_pY-pm-pd pH:pM:pS_124` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 5 | `strftime_pY-pm-pd pH:pM:pS_-1` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 6 | `strftime_pY-pm-pd pH:pM:pS_199` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 7 | `strftime_pY-pm-pd pH:pM:pS_70` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 8 | `strftime_pY-pm-pd pH:pM:pS_103` | `%Y-%m-%d %H:%M:%S` | 19 | 19 | match |
| 9 | `strftime_pY/pm/pd_0` | `%Y/%m/%d` | 10 | 10 | match |
| 10 | `strftime_pY/pm/pd_124` | `%Y/%m/%d` | 10 | 10 | match |

## 4. Root Cause Analysis

### Why 976/1000 pass with a stub implementation?

The Rust `strftime` stub always returns `0`. It passes when the C implementation also returns `0`, which happens when:

1. **Invalid format specifier**: `%E`, `%O`, or unknown specifiers cause C to return 0
2. **Buffer too small**: `n=0` or `n=1` causes C to return 0
3. **Invalid tm values**: out-of-range `tm_wday`, `tm_mon` etc. cause some format specifiers to fail

4. **Empty format string**: `f=""` → both return 0


### Why 24 cases fail?

These are cases where the C strftime successfully formats a valid output (returns > 0) but the Rust stub returns 0.

They correspond to valid format strings (`%Y-%m-%d`, `%H:%M:%S`, etc.) with valid `tm` values and sufficient buffer size.


## 5. Statistics

- **C return value distribution**: `8`: 181, `2`: 171, `10`: 137, `1`: 76, `5`: 73, `11`: 66, `24`: 51, `0`: 44, `3`: 32, `25`: 31, `4`: 29, `19`: 24, `?`: 24, `20`: 23, `17`: 13
- **Rust return values**: all `0` (stub)
