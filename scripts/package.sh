#!/usr/bin/env bash
set -euo pipefail
version=${1:?usage: package.sh VERSION}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected numeric X.Y.Z version' >&2; exit 1; }
root=$(pwd)
stage="$root/target/package-root"
rm -rf "$stage"
mkdir -p "$stage/usr/share/icons/hicolor/128x128/apps" "$stage/usr/bin" "$stage/usr/share/applications" "$stage/usr/share/icons/hicolor/scalable/apps" "$stage/usr/share/licenses/stl-viewer-linux" dist
install -m755 target/release/stl-viewer-linux "$stage/usr/bin/"
install -m644 packaging/stl-viewer-linux.desktop "$stage/usr/share/applications/"
install -m644 packaging/stl-viewer-linux.png "$stage/usr/share/icons/hicolor/128x128/apps/"
install -m644 packaging/stl-viewer-linux.svg "$stage/usr/share/icons/hicolor/scalable/apps/"
install -m644 LICENSE "$stage/usr/share/licenses/stl-viewer-linux/"
tar -C "$stage" -czf "dist/stl-viewer-linux-${version}-x86_64.tar.gz" usr
mkdir -p "$stage/DEBIAN"
cat > "$stage/DEBIAN/control" <<CONTROL
Package: stl-viewer-linux
Version: $version
Architecture: amd64
Maintainer: STL Viewer contributors <contributors@example.invalid>
Depends: libc6 (>= 2.35), libgcc-s1, libegl1, libgl1, libx11-6, libxcursor1, libxi6, libxrandr2, libxkbcommon0, libxkbcommon-x11-0, libwayland-client0
Recommends: xdg-desktop-portal
Section: graphics
Priority: optional
Description: Native GPU accelerated STL viewer for Wayland and X11
CONTROL
dpkg-deb --root-owner-group --build "$stage" "dist/stl-viewer-linux-${version}-amd64.deb"
rm -rf "$stage/DEBIAN"
fpm -s dir -t rpm -n stl-viewer-linux -v "$version" --architecture x86_64 --license MIT --description 'Native GPU accelerated STL viewer' --maintainer 'STL Viewer contributors <contributors@example.invalid>' --depends 'libglvnd-egl' --depends 'libglvnd-glx' --depends libX11 --depends libXcursor --depends libXi --depends libXrandr --depends libxkbcommon --depends libxkbcommon-x11 --depends wayland-libs -C "$stage" -p "dist/stl-viewer-linux-${version}-x86_64.rpm" usr
# Arch package payload and metadata: directly installable using pacman -U.
rm -rf "$root/target/arch-root"
cp -a "$stage" "$root/target/arch-root"
arch="$root/target/arch-root"
cat > "$arch/.PKGINFO" <<INFO
pkgname = stl-viewer-linux
pkgbase = stl-viewer-linux
pkgver = $version-1
pkgdesc = Native GPU accelerated STL viewer
url = https://github.com
builddate = ${SOURCE_DATE_EPOCH:-0}
packager = STL Viewer contributors
size = $(du -sb "$stage/usr" | cut -f1)
arch = x86_64
license = MIT
depend = glibc
depend = gcc-libs
depend = libglvnd
depend = libx11
depend = libxcursor
depend = libxi
depend = libxrandr
depend = libxkbcommon
depend = libxkbcommon-x11
depend = wayland
optdepend = xdg-desktop-portal: native file chooser
INFO
bsdtar --uid 0 --gid 0 --uname root --gname root -C "$arch" -cf - .PKGINFO usr | zstd -T0 -o "dist/stl-viewer-linux-${version}-1-x86_64.pkg.tar.zst" -f
(cd dist && sha256sum ./*.deb ./*.rpm ./*.pkg.tar.zst ./*.tar.gz > SHA256SUMS)
