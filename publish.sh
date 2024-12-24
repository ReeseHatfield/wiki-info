#!/bin/sh
cargo test
cargo login
cargo package
cargo publish