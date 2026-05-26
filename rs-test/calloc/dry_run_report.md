# Dry Run Test Report

**Date**: 2026-05-25T11:03:11Z

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
| fnmatch | 1000 | 1000 | 0 | 0 | 100.00% |

## Per-Category Results

| Category | Total | Passed | Failed | Rate |
|----------|-------|--------|--------|------|
| boundary | 50 | 50 | 0 | 100.00% |
| normal | 950 | 950 | 0 | 100.00% |

## Failed Tests

No failures. All tests passed.

## Sample Passed Tests (first 10)

| # | Name | Category | C ret | Rust ret |
|---|------|----------|-------|----------|
| 1 | `fnmatch_exact_match_none` | `normal` | `0` | `0` |
| 2 | `fnmatch_exact_match_FNM_PATHNAME` | `normal` | `0` | `0` |
| 3 | `fnmatch_exact_match_FNM_NOESCAPE` | `normal` | `0` | `0` |
| 4 | `fnmatch_exact_match_FNM_PERIOD` | `normal` | `0` | `0` |
| 5 | `fnmatch_exact_match_FNM_LEADING_DIR` | `normal` | `0` | `0` |
| 6 | `fnmatch_exact_match_FNM_CASEFOLD` | `normal` | `0` | `0` |
| 7 | `fnmatch_exact_match_PATHNAME|PERIOD` | `normal` | `0` | `0` |
| 8 | `fnmatch_exact_match_PATHNAME|CASEFOLD` | `normal` | `0` | `0` |
| 9 | `fnmatch_exact_match_NOESCAPE|CASEFOLD` | `normal` | `0` | `0` |
| 10 | `fnmatch_exact_match_PATHNAME|NOESCAPE|PERIOD` | `normal` | `0` | `0` |

## Layer 3 Coverage

### fnmatch

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

