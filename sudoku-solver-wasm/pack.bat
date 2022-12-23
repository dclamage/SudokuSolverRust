@echo off
cargo install cargo-make
cargo make --makefile .\pack.toml pack
