#!/bin/bash
# THIS SCRIPT MUST BE RUN FROM THE ROOT OF THE REPOSITORY
echo "Building for x86 linux.."
cargo b --release --target x86_64-unknown-linux-gnu
echo "Building for arm64 linux.."
cargo b --release --target aarch64-unknown-linux-gnu
echo "Downloading AppImageTool.."
curl -sLo ./appimagetool.AppImage https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x ./appimagetool.AppImage
echo "Building x86 appimage.."
cp ./target/x86_64-unknown-linux-gnu/release/sortphoto ./packaging/linux/usr/bin/sortphoto
ARCH=x86_64 ./appimagetool.AppImage -n ./packaging/linux/SortPhoto.AppDir ./packaging/linux/sortphoto-x64.AppImage
echo "Building arm64 appimage.."
cp ./target/aarch64-unknown-linux-gnu/release/sortphoto ./packaging/linux/usr/bin/sortphoto
ARCH=aarch64 ./appimagetool.AppImage -n ./packaging/linux/SortPhoto.AppDir ./packaging/linux/sortphoto-arm.AppImage
echo "Cleaning up.."
rm ./appimagetool.AppImage
