#!/usr/bin/env sh
set -e

# gpukill install script: fetch prebuilt GitHub release binary

REPO_OWNER="kagehq"
REPO_NAME="gpu-kill"
INSTALL_DIR_DEFAULT="$HOME/.local/bin"
BIN_NAME="gpukill"

# Flags
VERSION=""
BIN_DIR=""
YES="0"
INSECURE="0"

usage() {
  echo "Usage: curl -fsSL https://get.gpukill.sh | sh [-s] -- [--version vX.Y.Z] [--bin-dir DIR] [--yes] [--insecure]" >&2
}

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --bin-dir) BIN_DIR="$2"; shift 2 ;;
    --yes|-y) YES="1"; shift ;;
    --insecure) INSECURE="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

detect_os() {
  uname_s=$(uname -s 2>/dev/null || echo unknown)
  case "$uname_s" in
    Linux) echo linux ;;
    Darwin) echo macos ;;
    *) echo unsupported ;;
  esac
}

detect_arch() {
  uname_m=$(uname -m 2>/dev/null || echo unknown)
  case "$uname_m" in
    x86_64|amd64) echo x86_64 ;;
    aarch64|arm64) echo aarch64 ;;
    *) echo unsupported ;;
  esac
}

need_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }; }

need_cmd curl
need_cmd uname
need_cmd mkdir
need_cmd chmod

OS=$(detect_os)
ARCH=$(detect_arch)
if [ "$OS" = "unsupported" ] || [ "$ARCH" = "unsupported" ]; then
  echo "Unsupported platform: OS=$OS ARCH=$ARCH" >&2
  exit 1
fi

BIN_DIR=${BIN_DIR:-$INSTALL_DIR_DEFAULT}
mkdir -p "$BIN_DIR"

# Resolve version
API="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
if [ -n "$VERSION" ]; then
  API="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/tags/$VERSION"
fi

echo "Resolving release…"
TAG=$(curl -fsSL "$API" | sed -n 's/  \"tag_name\": \"\(.*\)\",/\1/p' | head -n1)
if [ -z "$TAG" ]; then
  echo "Failed to resolve release tag" >&2
  exit 1
fi

case "$OS-$ARCH" in
  linux-x86_64) ASSET="$REPO_NAME-$TAG-linux-x86_64" ;;
  linux-aarch64) ASSET="$REPO_NAME-$TAG-linux-aarch64" ;;
  macos-x86_64) ASSET="$REPO_NAME-$TAG-macos-x86_64" ;;
  macos-aarch64) ASSET="$REPO_NAME-$TAG-macos-aarch64" ;;
esac

URL_BASE="https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$TAG"
BIN_URL="$URL_BASE/$ASSET"
SUMS_URL="$URL_BASE/SHA256SUMS"

TMPDIR=${TMPDIR:-/tmp}
TMP_BIN="$TMPDIR/$ASSET"
TMP_SUMS="$TMPDIR/${REPO_NAME}_SHA256SUMS"

echo "Downloading binary: $BIN_URL"
curl -fsSL "$BIN_URL" -o "$TMP_BIN"

echo "Downloading checksums: $SUMS_URL"
curl -fsSL "$SUMS_URL" -o "$TMP_SUMS" || true

if [ -s "$TMP_SUMS" ]; then
  need_cmd shasum || need_cmd sha256sum
  if command -v shasum >/dev/null 2>&1; then
    SUM=$(shasum -a 256 "$TMP_BIN" | awk '{print $1}')
  else
    SUM=$(sha256sum "$TMP_BIN" | awk '{print $1}')
  fi
  if ! grep -q "$SUM" "$TMP_SUMS"; then
    if [ "$INSECURE" != "1" ]; then
      echo "Checksum verification failed" >&2
      exit 1
    else
      echo "WARNING: checksum verification skipped (--insecure)" >&2
    fi
  fi
else
  echo "WARNING: no checksum file found in release; proceeding" >&2
fi

DEST="$BIN_DIR/$BIN_NAME"
mv "$TMP_BIN" "$DEST"
chmod +x "$DEST"

if ! printf %s ":$PATH:" | grep -q ":$BIN_DIR:"; then
  echo "Installed to $DEST but $BIN_DIR is not in PATH" >&2
  echo "Add this to your shell rc: export PATH=\"$BIN_DIR:\$PATH\"" >&2
fi

echo "✅ Installed $BIN_NAME $TAG to $DEST"
"$DEST" --version || true

