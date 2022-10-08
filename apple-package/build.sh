#!/bin/bash
# THIS SCRIPT MUST BE RUN FROM THE ROOT OF THE GIT REPO
cargo b --release --target aarch64-apple-darwin
cargo b --release --target x86_64-apple-darwin
lipo -create -output ./apple-package/bundle/Contents/MacOS/sortphoto ./target/x86_64-apple-darwin/release/sortphoto ./target/aarch64-apple-darwin/release/sortphoto