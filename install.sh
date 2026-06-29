#!/usr/bin/env sh
set -e

REPO="zamkara/pagespeed"
BIN_NAME="pagespeed"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

log()  { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
err()  { printf '\033[1;31mError:\033[0m %s\n' "$*" >&2; exit 1; }

# detect OS
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      armv7*)  TARGET="armv7-unknown-linux-gnueabihf" ;;
      *)       err "Arsitektur tidak didukung: $ARCH" ;;
    esac
    EXT="tar.gz"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *)       err "Arsitektur tidak didukung: $ARCH" ;;
    esac
    EXT="tar.gz"
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-pc-windows-msvc" ;;
      aarch64) TARGET="aarch64-pc-windows-msvc" ;;
      *)       err "Arsitektur tidak didukung: $ARCH" ;;
    esac
    EXT="zip"
    BIN_NAME="pagespeed.exe"
    ;;
  *) err "OS tidak didukung: $OS" ;;
esac

# fetch latest release tag
log "Mengambil versi terbaru..."
if command -v curl >/dev/null 2>&1; then
  LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
elif command -v wget >/dev/null 2>&1; then
  LATEST=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
else
  err "curl atau wget diperlukan"
fi

[ -z "$LATEST" ] && err "Gagal mendapatkan versi terbaru"

log "Versi: $LATEST | Target: $TARGET"

ASSET="pagespeed-${LATEST}-${TARGET}.${EXT}"
URL="https://github.com/$REPO/releases/download/$LATEST/$ASSET"
TMP=$(mktemp -d)

log "Mengunduh $ASSET..."
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$TMP/$ASSET"
else
  wget -qO "$TMP/$ASSET" "$URL"
fi

log "Mengekstrak..."
if [ "$EXT" = "tar.gz" ]; then
  tar -xzf "$TMP/$ASSET" -C "$TMP"
else
  unzip -q "$TMP/$ASSET" -d "$TMP"
fi

# install
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  chmod +x "$INSTALL_DIR/$BIN_NAME"
else
  log "Membutuhkan sudo untuk install ke $INSTALL_DIR"
  sudo mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  sudo chmod +x "$INSTALL_DIR/$BIN_NAME"
fi

rm -rf "$TMP"

log "pagespeed $LATEST berhasil dipasang di $INSTALL_DIR/$BIN_NAME"
log "Jalankan: pagespeed --help"
