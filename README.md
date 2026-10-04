# Polyku

Modern, standalone, offline sudoku for Windows. Every puzzle is freshly
generated on your machine — always unique, always solvable by logic alone.

## Planned features

- **Procedural generation** with a strictly-unique-solution guarantee
  (DLX / exact-cover verification)
- **Variant rules you can stack**: Classic, Diagonal (X), Killer, Thermo,
  Non-Consecutive — combine them freely
- **6 difficulty tiers** graded by human logic techniques, no guessing required
- **Educational hints** that explain the deduction step by step
- 9-color cell marking, digit highlighting, dual candidate views
- Dark & light themes, silent by design, zero telemetry, fully offline

## Status

🚧 Early development — milestone M0 (project scaffold).

## Building from source

Prerequisites: [Rust](https://rustup.rs) (stable, MSVC), Node.js LTS.

```sh
npm install
npm run tauri dev
```

## Contributing

The engine lives in `crates/polyku-engine` (pure Rust, no UI dependencies).
A variant is a single Rust file implementing the `VariantRule` trait — a
contributing guide will land alongside the variant system (M2).

## License

[MIT](LICENSE)
