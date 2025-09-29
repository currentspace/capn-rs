#!/bin/bash

# Script to bump version numbers across all Cap'n Web crates

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if version argument is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: Version number required${NC}"
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION="$1"

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

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
    echo -e "${RED}Error: Invalid version format${NC}"
    echo "Version must be in format X.Y.Z or X.Y.Z-suffix"
    exit 1
fi

echo -e "${GREEN}Bumping version to ${VERSION}${NC}"

# Function to update version in a Cargo.toml file
update_cargo_toml() {
    local file="$1"
    local crate_name=$(basename $(dirname "$file"))

    if [ ! -f "$file" ]; then
        echo -e "${RED}Error: $file not found${NC}"
        return 1
    fi

    # Update the crate version
    sed_inplace "s/^version = \".*\"/version = \"$VERSION\"/" "$file"

    # Update internal dependencies
    sed_inplace "s/capnweb-core = { version = \"[^\"]*\"/capnweb-core = { version = \"$VERSION\"/" "$file"
    sed_inplace "s/capnweb-transport = { version = \"[^\"]*\"/capnweb-transport = { version = \"$VERSION\"/" "$file"
    sed_inplace "s/capnweb-server = { version = \"[^\"]*\"/capnweb-server = { version = \"$VERSION\"/" "$file"
    sed_inplace "s/capnweb-client = { version = \"[^\"]*\"/capnweb-client = { version = \"$VERSION\"/" "$file"

    # Also update path+version dependencies
    sed_inplace "s/version = \"[^\"]*\", path = \"..\/capnweb-core\"/version = \"$VERSION\", path = \"..\/capnweb-core\"/" "$file"
    sed_inplace "s/version = \"[^\"]*\", path = \"..\/capnweb-transport\"/version = \"$VERSION\", path = \"..\/capnweb-transport\"/" "$file"
    sed_inplace "s/version = \"[^\"]*\", path = \"..\/capnweb-server\"/version = \"$VERSION\", path = \"..\/capnweb-server\"/" "$file"
    sed_inplace "s/version = \"[^\"]*\", path = \"..\/capnweb-client\"/version = \"$VERSION\", path = \"..\/capnweb-client\"/" "$file"

    # Remove backup file
    rm -f "$file.bak"

    echo -e "${GREEN}✓${NC} Updated $crate_name"
}

# Update all crate Cargo.toml files
echo -e "${YELLOW}Updating Cargo.toml files...${NC}"
update_cargo_toml "capnweb-core/Cargo.toml"
update_cargo_toml "capnweb-transport/Cargo.toml"
update_cargo_toml "capnweb-server/Cargo.toml"
update_cargo_toml "capnweb-client/Cargo.toml"
update_cargo_toml "capnweb-interop-tests/Cargo.toml"

# Update the workspace Cargo.toml if it has a version
if grep -q "^version = " "Cargo.toml"; then
    sed_inplace "s/^version = \".*\"/version = \"$VERSION\"/" "Cargo.toml"
    rm -f "Cargo.toml.bak"
    echo -e "${GREEN}✓${NC} Updated workspace Cargo.toml"
fi

# Update Cargo.lock
echo -e "${YELLOW}Updating Cargo.lock...${NC}"
cargo update --workspace
echo -e "${GREEN}✓${NC} Updated Cargo.lock"

# Verify the changes compile
echo -e "${YELLOW}Verifying build...${NC}"
cargo build --workspace
echo -e "${GREEN}✓${NC} Build successful"

# Show what changed
echo -e "\n${YELLOW}Changes made:${NC}"
git diff --stat

echo -e "\n${GREEN}Version bump complete!${NC}"
echo -e "Next steps:"
echo -e "  1. Review changes: ${YELLOW}git diff${NC}"
echo -e "  2. Commit changes: ${YELLOW}git commit -am \"Release v$VERSION\"${NC}"
echo -e "  3. Create tag: ${YELLOW}git tag -a \"v$VERSION\" -m \"Release version $VERSION\"${NC}"
echo -e "  4. Push to origin: ${YELLOW}git push origin main --tags${NC}"