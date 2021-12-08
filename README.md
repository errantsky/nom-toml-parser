# A Nom-based TOML parser in Rust

As an exercise to learn more about Rust, I decided to write a parser using [Nom](https://github.com/Geal/nom) to read
TOML files, particularly `Cargo.toml` files.

Parsers for different TOML values are kept in separate modules in `/src/parsers`. Unit tests for each module is located
in the bottom of each file. Tests in `/src/parsers/mod.rs` parse `Cargo.toml` files from a few different Rust crates,
including `Nom` and
`cargo-expand`. These files and more can be found in the `assets` folder.