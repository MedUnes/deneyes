#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$PROJECT_ROOT/target/debian"
PACKAGE_NAME="dneyes"

cd "$PROJECT_ROOT"

VERSION="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys;print(json.load(sys.stdin)["packages"][0]["version"])')"
ARCH="$(dpkg --print-architecture)"

cargo build --release

STAGING_DIR="$TARGET_DIR/${PACKAGE_NAME}_${VERSION}_$ARCH"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/DEBIAN"
mkdir -p "$STAGING_DIR/usr/bin"
mkdir -p "$STAGING_DIR/etc/dneyes"
mkdir -p "$STAGING_DIR/lib/systemd/system"
mkdir -p "$STAGING_DIR/usr/share/doc/$PACKAGE_NAME"

install -Dm755 "target/release/$PACKAGE_NAME" "$STAGING_DIR/usr/bin/$PACKAGE_NAME"
install -Dm644 "packaging/systemd/dneyes.service" "$STAGING_DIR/lib/systemd/system/dneyes.service"
install -Dm644 "packaging/config/default.yaml" "$STAGING_DIR/etc/dneyes/config.yaml"
install -Dm644 "README.md" "$STAGING_DIR/usr/share/doc/$PACKAGE_NAME/README.md"
install -Dm644 "LICENSE" "$STAGING_DIR/usr/share/doc/$PACKAGE_NAME/LICENSE"

cat > "$STAGING_DIR/DEBIAN/control" <<CONTROL
Package: $PACKAGE_NAME
Version: $VERSION
Section: net
Priority: optional
Architecture: $ARCH
Depends: libc6 (>= 2.31), systemd
Maintainer: DNeyeS Maintainers <maintainers@example.com>
Description: DNS resolution monitoring radar written in Rust
 Monitors DNS resolvers world-wide and exposes metrics via REST and ClickHouse.
CONTROL

install -Dm755 "packaging/debian/postinst" "$STAGING_DIR/DEBIAN/postinst"
install -Dm755 "packaging/debian/prerm" "$STAGING_DIR/DEBIAN/prerm"

fakeroot dpkg-deb --build "$STAGING_DIR"

echo "Debian package created at $STAGING_DIR.deb"
