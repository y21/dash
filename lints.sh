#!/bin/sh

cargo b -p lints
RUSTC_WRAPPER="./target/debug/lints" cargo c -p dash-cli --all-features
