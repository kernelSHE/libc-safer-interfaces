# Dry Run Test Report

**Date**: 2026-05-19T23:55:16Z

## Summary

- **Total tests**: 10000
- **Passed**: 10000
- **Failed**: 0
- **Known limitations**: 0
- **Effective pass rate**: 100.00%
- **Verdict**: `consistent`

## Per-Function Results

| Function | Total | Passed | Failed | Known | Rate |
|----------|-------|--------|--------|-------|------|
| strxfrm | 10000 | 10000 | 0 | 0 | 100.00% |

## Per-Category Results

| Category | Total | Passed | Failed | Rate |
|----------|-------|--------|--------|------|
| boundary | 16 | 16 | 0 | 100.00% |
| normal | 9984 | 9984 | 0 | 100.00% |

## Failed Tests

No failures. All tests passed.

## Sample Passed Tests (first 10)

| # | Name | Category | C ret | Rust ret |
|---|------|----------|-------|----------|
| 1 | `strxfrm__0` | `boundary` | `0` | `0` |
| 2 | `strxfrm__1` | `boundary` | `0` | `0` |
| 3 | `strxfrm__0` | `boundary` | `0` | `0` |
| 4 | `strxfrm__1` | `boundary` | `0` | `0` |
| 5 | `strxfrm__0` | `boundary` | `0` | `0` |
| 6 | `strxfrm_61_0` | `boundary` | `1` | `1` |
| 7 | `strxfrm_61_1` | `normal` | `1` | `1` |
| 8 | `strxfrm_61_1` | `normal` | `1` | `1` |
| 9 | `strxfrm_61_2` | `normal` | `1` | `1` |
| 10 | `strxfrm_61_2` | `normal` | `1` | `1` |

## Layer 3 Coverage

### strxfrm

- **Cases executed**: 10000/10000
- **Passed**: 10000
- **Failed**: 0
- **Coverage score**: 1.000
- **Suggestions**:
  - Missing source encodings (tested 0/20)
  - Missing target encodings (tested 0/20)
  - No boundary value tests
  - No edge case tests
  - No error path tests executed

