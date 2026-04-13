# Testing Guide

## Test layers

- Unit tests inside each crate validate helpers and rule behavior.
- Integration tests in `crates/axl_cli/tests` validate end-to-end CLI output against fixtures.
- Shared fixtures live in `tests/fixtures`.

## Run all tests

```bash
cargo test
```

## Run focused suites

```bash
cargo test -p axl_rules
cargo test -p axl_cli --test cli_output
```

## Add new rule tests

1. Add focused unit tests in the rule module (`crates/axl_rules/src/**`).
2. Add or update fixtures under `tests/fixtures`.
3. Add CLI integration assertions if behavior should be user-visible.

## Coverage checklist

- Positive detection path
- False-positive guard path
- Category filtering behavior
- Output format checks (`text` and `json`)
