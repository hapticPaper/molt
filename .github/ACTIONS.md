# GitHub Actions Setup Summary

## Workflows Created

### 1. **CI** (`.github/workflows/ci.yml`)
Runs on every push and pull request to `main` and `develop` branches.

**Jobs:**
- `test-rust`: Run Cargo tests, clippy, and formatting checks
- `test-mcp`: Build and test the NPM MCP server
- `build-check`: Build Rust binaries on Linux, macOS, and Windows

### 2. **Release** (`.github/workflows/release.yml`)
Triggers on version tags (`v*`) or manual workflow dispatch.

**Rust Binary Matrix (8 platforms):**
- Linux: x86_64-gnu, x86_64-musl, aarch64-gnu, aarch64-musl
- macOS: x86_64, aarch64 (Apple Silicon)
- Windows: x86_64, aarch64

**Jobs:**
- `build-rust`: Build all platform binaries in parallel
- `publish-npm`: Publish MCP server to npm
- `create-release`: Create GitHub release with all artifacts

### 3. **Publish MCP** (`.github/workflows/publish-mcp.yml`)
Standalone NPM publishing on `mcp-v*` tags.

### 4. **Nightly** (`.github/workflows/nightly.yml`)
Daily builds at 2 AM UTC to catch regressions early.

## Required Secrets

Add these in GitHub Settings → Secrets and variables → Actions:

- `NPM_TOKEN`: npm access token for publishing `@hardclaw/mcp-server`
  - Get from: https://www.npmjs.com/settings/YOUR_USERNAME/tokens
  - Type: Automation token

## Creating a Release

### Full Release (Rust + MCP)
```bash
# Update versions
vim Cargo.toml  # Update version
vim hardclaw-mcp/package.json  # Update version

# Commit and tag
git add -A
git commit -m "Release v0.9.2"
git tag v0.9.2
git push origin main --tags
```

### Rust Binaries Only
```bash
git tag v0.9.2
git push origin v0.9.2
```

### MCP Server Only
```bash
git tag mcp-v0.1.1
git push origin mcp-v0.1.1
```

## Artifacts Generated

Each release creates:
- 8 Rust binary archives (4 Linux, 2 macOS, 2 Windows)
- 1 NPM package tarball
- GitHub release with all downloads
- NPM registry publication

## Platform Support

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| Linux (glibc) | x86_64 | hardclaw-linux-x86_64.tar.gz |
| Linux (glibc) | ARM64 | hardclaw-linux-aarch64.tar.gz |
| Linux (musl) | x86_64 | hardclaw-linux-x86_64-musl.tar.gz |
| Linux (musl) | ARM64 | hardclaw-linux-aarch64-musl.tar.gz |
| macOS | Intel | hardclaw-macos-x86_64.tar.gz |
| macOS | Apple Silicon | hardclaw-macos-aarch64.tar.gz |
| Windows | x86_64 | hardclaw-windows-x86_64.zip |
| Windows | ARM64 | hardclaw-windows-aarch64.zip |
| NPM (all) | Node 18+ | @hardclaw/mcp-server |
