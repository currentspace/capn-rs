#!/bin/bash

# Patch script to fix navigator.userAgent issue in capnweb for Node.js compatibility

DIST_FILE="./capnweb-github/dist/index.js"

# Portable in-place substitution
sed_inplace() {
    local pattern="$1"
    local target="$2"
    if sed --version >/dev/null 2>&1; then
        # GNU sed
        sed -i.bak "$pattern" "$target"
    else
        # BSD sed (macOS)
        sed -i '.bak' "$pattern" "$target"
    fi
}

echo "🔧 Patching capnweb for Node.js compatibility..."

if [ ! -f "$DIST_FILE" ]; then
    echo "❌ Error: $DIST_FILE not found. Run setup-capnweb.sh first."
    exit 1
fi

# Create a backup
cp "$DIST_FILE" "$DIST_FILE.backup"

# Patch the navigator.userAgent reference to check if navigator exists first
sed_inplace 's/var workersModuleName = navigator\.userAgent === "Cloudflare-Workers" ? "cloudflare:workers" : null;/var workersModuleName = (typeof navigator !== "undefined" \&\& navigator.userAgent === "Cloudflare-Workers") ? "cloudflare:workers" : null;/' "$DIST_FILE"

echo "✅ Patched capnweb for Node.js compatibility"
echo "📁 Original file backed up to: $DIST_FILE.backup"