#!/bin/bash

# Setup script to clone and build latest capnweb from GitHub

CAPNWEB_DIR="./capnweb-github"

echo "🔧 Setting up latest capnweb from GitHub..."

# Remove old directory if exists
if [ -d "$CAPNWEB_DIR" ]; then
    echo "📦 Removing old capnweb directory..."
    rm -rf "$CAPNWEB_DIR"
fi

# Clone the repository
echo "📥 Cloning capnweb from GitHub..."
git clone https://github.com/cloudflare/capnweb.git "$CAPNWEB_DIR"

if [ $? -ne 0 ]; then
    echo "❌ Failed to clone capnweb repository"
    exit 1
fi

cd "$CAPNWEB_DIR"

# Install dependencies
echo "📦 Installing dependencies..."
pnpm install

if [ $? -ne 0 ]; then
    echo "❌ Failed to install dependencies"
    exit 1
fi

# Build the project
echo "🔨 Building capnweb..."
pnpm run build

if [ $? -ne 0 ]; then
    echo "❌ Failed to build capnweb"
    exit 1
fi

# Check if dist directory was created
if [ -d "dist" ]; then
    echo "✅ Successfully built capnweb!"
    echo "📁 Build output in: $(pwd)/dist"
else
    echo "❌ Build completed but dist directory not found"
    exit 1
fi

cd ..

# Link it locally
echo "🔗 Linking capnweb for local use..."
cd "$CAPNWEB_DIR"
pnpm link --global
cd ..
pnpm link --global capnweb

echo "✅ Setup complete! You can now use the latest capnweb in your tests."