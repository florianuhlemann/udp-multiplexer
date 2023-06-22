#/bin/bash

# clear all
rm -rf ./target

# Mac OS / local
cargo build --release --target=x86_64-apple-darwin --verbose

# Windows
cargo build --release --target=x86_64-pc-windows-gnu --verbose

# Linux
cargo build --release --target=x86_64-unknown-linux-gnu --verbose