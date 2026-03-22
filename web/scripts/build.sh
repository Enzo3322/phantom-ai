#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WEB_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$WEB_DIR")"

echo "==> Building Tauri app..."
cd "$ROOT_DIR"
pnpm bundle

echo "==> Copying .dmg to web/public/downloads..."
mkdir -p "$WEB_DIR/public/downloads"

DMG_FILE=$(find "$ROOT_DIR/src-tauri/target/release/bundle/macos" -name "Phantom_*.dmg" -not -name "rw.*" | head -1)

if [ -z "$DMG_FILE" ]; then
  echo "Error: No .dmg file found in src-tauri/target/release/bundle/macos/"
  exit 1
fi

cp "$DMG_FILE" "$WEB_DIR/public/downloads/Phantom.dmg"
echo "==> Copied $(basename "$DMG_FILE") -> public/downloads/Phantom.dmg"

echo "==> Building Docker image..."
cd "$WEB_DIR"
docker build -t phantom-web .

echo "==> Done! Run with: docker run -p 3000:3000 phantom-web"
