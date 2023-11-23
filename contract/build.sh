#!/bin/bash
set -ex
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/simple_bet.wasm res/simple_bet.wasm
ls -lh res
