#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Building release binary..."
cargo build --release

echo "==> Building .deb..."
cargo deb --output target/debian/

echo "==> Building .rpm..."
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cp packaging/finder.spec ~/rpmbuild/SPECS/
rpmbuild -bb ~/rpmbuild/SPECS/finder.spec

echo "==> Done!"
echo "  .deb: target/debian/finder_0.1.0_amd64.deb"
echo "  .rpm: ~/rpmbuild/RPMS/x86_64/finder-0.1.0-1.x86_64.rpm"
