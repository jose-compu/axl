# axl

`axl` is a Rust workspace for accessibility linting of JSX/TSX code.

Current v0.2 focus:
- runnable CLI (`axl_cli`)
- parser/discovery pipeline (`axl_parser`)
- shared diagnostics/rule engine interfaces (`axl_core`)
- initial ARIA/name/focus rules (`axl_rules`, `axl_aria`)
- early contrast and pattern analysis scaffolding (`axl_contrast`, `axl_patterns`)

## Workspace crates

- `axl_cli`: command line entry point and output formatting
- `axl_parser`: TSX/JSX target collection and OXC parse validation
- `axl_core`: shared lint data model (`Diagnostic`, `Rule`, `Severity`, etc.)
- `axl_aria`: ARIA role and accessible-name helpers
- `axl_rules`: active lint rules and rule registry
- `axl_config`: `axl.config.json` loading
- `axl_contrast`: WCAG contrast math and color helpers (scaffold)
- `axl_patterns`: UI pattern detector API + dialog placeholder (scaffold)

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

## Current implemented rules

- `roles/abstract-role-usage` (error)
- `naming/empty-accessible-name` (error)
- `focus/tabindex-positive` (warning)

## Configuration

`axl` loads `axl.config.json` by default (or the file from `--config`).

Supported top-level fields (minimal v0.1 support):
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

## Scope limitations (v0.2)

- no full AccName 1.2 implementation
- no context-dependent implicit role algorithm
- no CSS parsing/resolution
- no cross-file component tree analysis
- limited autofix scope (currently `focus/tabindex-positive` only)
