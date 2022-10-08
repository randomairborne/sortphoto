#!/bin/bash
# THIS SCRIPT MUST BE RUN FROM THE ROOT OF THE GIT REPO
echo "Building for arm64 mac"
cargo b --release --target aarch64-apple-darwin
echo "Building for x86 mac"
cargo b --release --target x86_64-apple-darwin
echo "Creating universal binary executable"
lipo -create -output ./packaging/mac/bundle/Contents/MacOS/sortphoto ./target/x86_64-apple-darwin/release/sortphoto ./target/aarch64-apple-darwin/release/sortphoto
echo "Copying bundle to app file"
cp -r ./packaging/mac/bundle ./packaging/mac/sortphoto.app
echo "Generating installer package"
pkgbuild --install-location /Applications --component ./packaging/mac/sortphoto.app ./packaging/mac/sortphoto.pkg
