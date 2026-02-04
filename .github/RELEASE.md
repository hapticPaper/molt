# HardClaw Release Process

## Versioning

- Rust binaries: `v{major}.{minor}.{patch}` (e.g., `v0.9.1`)
- MCP server: `mcp-v{major}.{minor}.{patch}` (e.g., `mcp-v0.1.0`)

## Creating a Release

### Rust Binaries

1. Update version in `Cargo.toml`
2. Commit changes: `git commit -am "Bump version to X.Y.Z"`
3. Create and push tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. GitHub Actions will build for all platforms and create a release

### MCP Server

1. Update version in `hardclaw-mcp/package.json`
2. Commit changes: `git commit -am "Bump MCP version to X.Y.Z"`
3. Create and push tag: `git tag mcp-vX.Y.Z && git push origin mcp-vX.Y.Z`
4. GitHub Actions will publish to npm

### Combined Release

For a full release with both Rust and MCP:

1. Update both version files
2. Commit: `git commit -am "Release vX.Y.Z"`
3. Tag Rust: `git tag vX.Y.Z`
4. Tag MCP: `git tag mcp-vX.Y.Z`
5. Push: `git push origin main --tags`

## Supported Platforms

### Rust Binaries

**Linux**
- x86_64 (glibc)
- x86_64 (musl)
- aarch64 (glibc)
- aarch64 (musl)

**macOS**
- x86_64 (Intel)
- aarch64 (Apple Silicon)

**Windows**
- x86_64
- aarch64 (ARM64)

### MCP Server

- Cross-platform via npm (Node.js 18+)

## GitHub Actions Workflows

- **CI** (`ci.yml`): Runs tests on every push/PR
- **Release** (`release.yml`): Builds binaries for all platforms on version tags
- **Publish MCP** (`publish-mcp.yml`): Publishes MCP to npm on mcp-v* tags
- **Nightly** (`nightly.yml`): Daily builds to catch regressions

## Required Secrets

Set these in GitHub repository settings:

- `NPM_TOKEN`: npm access token for publishing @hardclaw/mcp-server
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions
