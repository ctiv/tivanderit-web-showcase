#!/usr/bin/env bash
#
# This script synchronizes version numbers and dependency versions
# across Cargo.toml and package.json.
# It ensures that there is a single source of truth for versioning.

set -euo pipefail

# --- Configuration ---
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
CARGO_TOML="${SCRIPT_DIR}/Cargo.toml"
PACKAGE_JSON="${SCRIPT_DIR}/end2end/package.json"

# --- Main Logic ---
echo "⚙️  Starting version synchronization..."

# Verify that the source files exist before proceeding
if [ ! -f "${CARGO_TOML}" ]; then
    echo "❌ Error: Cargo.toml not found at ${CARGO_TOML}"
    exit 1
fi
if [ ! -f "${PACKAGE_JSON}" ]; then
    echo "❌ Error: package.json not found at ${PACKAGE_JSON}"
    exit 1
fi

# 1. Read the single source of truth: version from Cargo.toml
PROJECT_VERSION=$(grep '^version =' "${CARGO_TOML}" | sed -E 's/version = "([^"]+)"/\1/')

if [ -z "${PROJECT_VERSION}" ]; then
    echo "❌ Error: Could not read project version from ${CARGO_TOML}"
    exit 1
fi
echo "✔️  Project version from Cargo.toml is: ${PROJECT_VERSION}"

# 2. Update package.json version to match Cargo.toml
echo "Updating version in ${PACKAGE_JSON} to ${PROJECT_VERSION}..."

# Use sed for in-place replacement.
sed -i -E "s/(\"version\":\s*)\".*\"/\1\"${PROJECT_VERSION}\"/" "${PACKAGE_JSON}"

# 3. Get versions of Nix-provided dev dependencies
PLAYWRIGHT_VERSION=$(playwright --version | sed 's/Version //')
TYPESCRIPT_VERSION=$(tsc --version | sed 's/Version //')
NODE_VERSION=$(node --version | sed 's/v//')

echo "✔️  Nix Environment provides Node.js version: ${NODE_VERSION}"
echo "✔️  Nix environment provides Playwright version: ${PLAYWRIGHT_VERSION}"
echo "✔️  Nix environment provides TypeScript version: ${TYPESCRIPT_VERSION}"

# 4. Update devDependencies in package.json
echo "Updating devDependencies in ${PACKAGE_JSON}..."

# Use jq to safely update the JSON file.
# It reads the file, updates the fields, and writes back to a temporary file, then replaces the original.
tmp=$(mktemp)
jq --arg node_ver "${NODE_VERSION}" \
   --arg pw_ver "${PLAYWRIGHT_VERSION}" \
   --arg ts_ver "${TYPESCRIPT_VERSION}" \
   '.devDependencies["@playwright/test"] = $pw_ver | .devDependencies["typescript"] = $ts_ver | .devDependencies["@types/node"] = $node_ver | .engines.node = $node_ver' \
   "${PACKAGE_JSON}" > "$tmp" && mv "$tmp" "${PACKAGE_JSON}"

echo "✅ Synchronization complete!"
