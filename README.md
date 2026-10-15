# axl

[![crates.io](https://img.shields.io/crates/v/axl_cli.svg)](https://crates.io/crates/axl_cli)

`axl` is a Rust workspace for accessibility linting of JSX/TSX code. **v0.4.0** is on crates.io.

Current v0.4 focus:
- AccName 1.2 subset in `axl_aria` (same-file traversal and precedence)
- additional naming rules consumed from that algorithm
- existing roles and focus rules unchanged
- runnable CLI, parser/discovery, shared diagnostics, contrast/pattern scaffolds

## Workspace crates

- `axl_cli`: command line entry point and output formatting
- `axl_parser`: TSX/JSX target collection and OXC parse validation
- `axl_core`: shared lint data model (`Diagnostic`, `Rule`, `Severity`, etc.)
- `axl_aria`: ARIA roles plus AccName 1.2 subset (`name_computation`, JSX tree)
- `axl_rules`: active lint rules and rule registry
- `axl_config`: `axl.config.json` loading
- `axl_contrast`: WCAG contrast math and color helpers (scaffold)
- `axl_patterns`: UI pattern detector API + dialog placeholder (scaffold)

## Install

```bash
cargo install axl_cli --locked
axl .
```

## Quick start

Requirements:
- Rust toolchain (`cargo`)

Run checks:

```bash
cargo test
```

Detailed docs:
- `docs/testing.md`
- `docs/publishing.md`

Lint current directory:

```bash
cargo run -p axl_cli -- .
```

JSON output:

```bash
cargo run -p axl_cli -- . --format json
```

Filter categories:

```bash
cargo run -p axl_cli -- . --only roles,naming,focus
```

Use explicit config path:

```bash
cargo run -p axl_cli -- src --config axl.config.json
```

Apply basic autofix (currently `focus/tabindex-positive` only):

```bash
cargo run -p axl_cli -- . --fix
```

Preview autofix without writing files:

```bash
cargo run -p axl_cli -- . --fix-dry-run
```

## AccName 1.2 subset (v0.4)

Same-file computation in `axl_aria::name_computation`, in spec order:

1. `aria-labelledby` (ID references in the same file; recursion guards)
2. `aria-label`
3. native `<label>` (`htmlFor`/`for` or wrapping)
4. name from content
5. `alt`
6. `title`

Whitespace is normalized. Dynamic JSX expressions are treated as unknown (not empty) so rules do not false-positive. Empty `aria-label` falls through; a present empty `alt` on `img` does not fall through to `title`.

## Current implemented rules

- `roles/abstract-role-usage` (error)
- `naming/empty-accessible-name` (error)
- `naming/img-missing-alt` (error)
- `naming/button-missing-name` (error)
- `naming/link-missing-name` (error)
- `naming/svg-missing-name` (error)
- `focus/tabindex-positive` (warning)

## Configuration

`axl` loads `axl.config.json` by default (or the file from `--config`).

Supported top-level fields (minimal v0.4 support):
- `extends`: string array
- `components`: object map
- `rules`: object map
- `ignore`: glob patterns to skip files

Example:

```json
{
  "extends": ["recommended"],
  "components": {
    "Link": "a"
  },
  "rules": {
    "focus/tabindex-positive": "warn"
  },
  "ignore": ["**/*.test.tsx"]
}
```

## Scope limitations (v0.4)

- AccName 1.2 is a same-file subset, not the full specification
- no complete implicit-role algorithm
- no CSS parsing/resolution (`:before`/`visibility`/`display`)
- no cross-file component tree analysis
- limited autofix scope (currently `focus/tabindex-positive` only)
