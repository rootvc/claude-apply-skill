#!/usr/bin/env bash
set -e

if ! command -v cargo &> /dev/null; then
    echo "Rust not installed. Install from https://rustup.rs"
    exit 1
fi

cd vui
cargo build --release
./target/release/vui
