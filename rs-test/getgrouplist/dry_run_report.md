# Dry Run Test Report

**Date**: 2026-05-23T07:07:55Z

## Summary

- **Total tests**: 1000
- **Passed**: 1000
- **Failed**: 0
- **Known limitations**: 0
- **Effective pass rate**: 100.00%
- **Verdict**: `consistent`

## Per-Function Results

| Function | Total | Passed | Failed | Known | Rate |
|----------|-------|--------|--------|-------|------|
| getgrouplist | 1000 | 1000 | 0 | 0 | 100.00% |

## Per-Category Results

| Category | Total | Passed | Failed | Rate |
|----------|-------|--------|--------|------|
| boundary | 24 | 24 | 0 | 100.00% |
| normal | 976 | 976 | 0 | 100.00% |

## Failed Tests

No failures. All tests passed.

## Sample Passed Tests (first 10)

| # | Name | Category | C ret | Rust ret |
|---|------|----------|-------|----------|
| 1 | `getgrouplist_yang_buf0` | `boundary` | `-1` | `-1` |
| 2 | `getgrouplist_yang_buf1_overflow` | `boundary` | `-1` | `-1` |
| 3 | `getgrouplist_yang_buf2_overflow` | `boundary` | `-1` | `-1` |
| 4 | `getgrouplist_yang_buf3_overflow` | `boundary` | `-1` | `-1` |
| 5 | `getgrouplist_yang_buf4_overflow` | `boundary` | `-1` | `-1` |
| 6 | `getgrouplist_yang_buf5_overflow` | `boundary` | `-1` | `-1` |
| 7 | `getgrouplist_yang_buf6_overflow` | `boundary` | `-1` | `-1` |
| 8 | `getgrouplist_yang_buf7_overflow` | `boundary` | `-1` | `-1` |
| 9 | `getgrouplist_yang_buf8_overflow` | `boundary` | `-1` | `-1` |
| 10 | `getgrouplist_yang_buf9_exact` | `boundary` | `9` | `9` |

## Layer 3 Coverage

### getgrouplist

- **Cases executed**: 1000/1000
- **Passed**: 1000
- **Failed**: 0
- **Coverage score**: 1.000
- **Suggestions**:
  - Missing source encodings (tested 0/20)
  - Missing target encodings (tested 0/20)
  - No boundary value tests
  - No edge case tests
  - No error path tests executed

