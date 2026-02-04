#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_error() { echo -e "${RED}✗ $1${NC}"; }
print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_info() { echo -e "${YELLOW}➜ $1${NC}"; }

if [ ! -f "Cargo.toml" ] || [ ! -f "hardclaw-mcp/package.json" ]; then
  print_error "Must run from repo root"
  exit 1
fi

if ! git diff-index --quiet HEAD --; then
  print_error "Uncommitted changes"
  exit 1
fi

bump_version() {
  local v=$1
  local type=${2:-patch}
  IFS='.' read -r major minor patch <<< "$v"
  
  case $type in
    major) echo "$((major+1)).0.0" ;;
    minor) echo "$major.$((minor+1)).0" ;;
    patch) echo "$major.$minor.$((patch+1))" ;;
  esac
}

RUST=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
MCP=$(grep '"version"' hardclaw-mcp/package.json | head -1 | cut -d'"' -f4)

echo ""
print_info "Current: Rust $RUST | MCP $MCP"
echo ""
echo "Bump type: (1=patch, 2=minor, 3=major)"
read -p "Choice [1]: " choice
choice=${choice:-1}

case $choice in
  1) type="patch" ;;
  2) type="minor" ;;
  3) type="major" ;;
  *) print_error "Invalid choice"; exit 1 ;;
esac

NEW_RUST=$(bump_version "$RUST" "$type")
NEW_MCP=$(bump_version "$MCP" "$type")

echo ""
print_info "New: Rust $NEW_RUST | MCP $NEW_MCP"
read -p "OK? (y/n) [y]: " ok
ok=${ok:-y}
[[ ! $ok =~ ^[Yy]$ ]] && { print_error "Cancelled"; exit 1; }

sed -i '' "0,/^version = \"$RUST\"/s//version = \"$NEW_RUST\"/" Cargo.toml
sed -i '' "s/\"version\": \"$MCP\"/\"version\": \"$NEW_MCP\"/" hardclaw-mcp/package.json

git add Cargo.toml hardclaw-mcp/package.json
git commit -m "Release v$NEW_RUST / mcp-v$NEW_MCP"
git tag "v$NEW_RUST" "mcp-v$NEW_MCP"
git push origin main --tags

print_success "Released! Actions deploying...""
