# Publishing Guide

Publish crates in dependency order:

```bash
cargo publish -p axl_core
cargo publish -p axl_config
cargo publish -p axl_aria
cargo publish -p axl_contrast
cargo publish -p axl_patterns
cargo publish -p axl_parser
cargo publish -p axl_rules
cargo publish -p axl_cli
```

If crates.io index propagation is slow between publishes, wait 30-60 seconds and retry the next crate.

For dry runs:

```bash
cargo publish -p axl_core --dry-run
```
