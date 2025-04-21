#!/usr/bin/env sh

cargo build --target wasm32-unknown-unknown

cp target/wasm32-unknown-unknown/debug/rl-strings.wasm .

basic-http-server .
