#!/bin/bash

# Patch script to fix navigator.userAgent issue in capnweb for Node.js compatibility

DIST_FILE="./capnweb-github/dist/index.js"

echo "🔧 Patching capnweb for Node.js compatibility..."

if [ ! -f "$DIST_FILE" ]; then
    echo "❌ Error: $DIST_FILE not found. Run setup-capnweb.sh first."
    exit 1
fi

# Create a backup
cp "$DIST_FILE" "$DIST_FILE.backup"

# Patch the navigator.userAgent reference to check if navigator exists first
sed -i.bak 's/var workersModuleName = navigator\.userAgent === "Cloudflare-Workers" ? "cloudflare:workers" : null;/var workersModuleName = (typeof navigator !== "undefined" \&\& navigator.userAgent === "Cloudflare-Workers") ? "cloudflare:workers" : null;/' "$DIST_FILE"

echo "✅ Patched capnweb for Node.js compatibility"
echo "📁 Original file backed up to: $DIST_FILE.backup"