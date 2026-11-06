# Theme Generator

Generates `src/theme/catalog.rs` from `assets/vhs/themes.json`. Run from the repository root after updating the bundled catalog:

```sh
cargo run --locked --manifest-path tools/theme-generator/Cargo.toml
```

Requires `rustfmt`.
