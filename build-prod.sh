#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-$(rustc -vV | awk '/host:/{print $2}')}"

echo "Building release binary for target: $TARGET"

rustup target add "$TARGET" 2>/dev/null || true

cargo build --release --target "$TARGET"

BINARY="target/$TARGET/release/pagespeed"

echo "Build complete: $BINARY"
echo "Size: $(du -sh "$BINARY" | cut -f1)"
