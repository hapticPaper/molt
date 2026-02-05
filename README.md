# HardClaw

**Proof-of-Verification for the Autonomous Agent Economy**

*"We do not trust; we verify."*

![HardClaw Logo](claw_logo.jpeg)

## What is HardClaw?

HardClaw is a blockchain protocol where **verification is the work**. Instead of wasting compute on arbitrary puzzles, verifiers cryptographically check real task outputs and earn block rewards.

## Status

This repo is a working prototype with a local TUI, a CLI demo, and a libp2p node. Networking, verification, and tokenomics are implemented at the protocol level, while full production flows (marketplace, payouts, and persistent state across distributed peers) are still in progress.

### Protocol Roles

| Role | Action | Reward |
|------|--------|--------|
| **Requester** | Submits jobs with bounties | Gets verified work done |
| **Solver** | Executes tasks, submits solutions | 95% of bounty |
| **Verifier** | Verifies solutions, produces blocks | 4% of bounty |

1% of every bounty is burned to offset state bloat.

## Quick Start

```bash
# Install HardClaw
cargo install --path .

# Run the onboarding TUI (wallet + environment check)
hardclaw

# Run a node (full node by default)
hardclaw-node

# Run a verifier node
hardclaw-node --verifier

# Connect to a bootstrap peer
hardclaw-node --bootstrap /ip4/<IP>/tcp/9000/p2p/<PEER_ID>
```

## Validator Environment

Validators need a properly configured environment to execute verification code safely. When you run `hardclaw`, the onboarding TUI automatically:

### Automated Setup
- **Python 3.12+ Sandbox**: Creates isolated venv at `~/.hardclaw/python-sandbox/`
  - Installs required packages (cryptography, requests, numpy)
  - Tests verification code execution
  - Saves config for all validator nodes
  
- **AI Models** (optional): Installs Ollama and downloads models
  - llama3.2 for code safety review
  - codellama for specialized analysis
  - Alternative: Use GPT-4/Claude API instead
  
- **JavaScript/TypeScript**: Embedded Deno runtime (always available)

All environments are isolated, tested, and persisted to `~/.hardclaw/` so every validator node uses the same verified setup.

**No manual configuration needed** - just run `hardclaw` and the environment is created automatically.

## Features

- **Proof-of-Verification (PoV)** - Verifiers check real work instead of hashes
- **Honey Pot Defense** - Detects lazy verifiers and slashes stake
- **Schelling Point Voting** - Subjective tasks with commit/reveal voting
- **Stake & Slashing Model** - Verifier stake tracked with penalties
- **Tokenomics Module** - Minted/burned accounting and fee splits
- **66% Consensus Threshold** - $2/3$ majority for block validity
- **Libp2p Networking** - Gossipsub + Kademlia peer discovery
- **Onboarding TUI** - Wallet creation/loading with secure seed phrase displayhrase display

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Requester  │────▶│   Solver    │────▶│  Verifier   │
│  (Jobs)     │     │  (Work)     │     │  (Blocks)   │
└─────────────┘     └─────────────┘     └─────────────┘
      │                   │                   │
      └───────────────────┴───────────────────┘
                          │
                    ┌─────▼─────┐
                    │  HCLAW    │
                    │  Token    │
                    └───────────┘
```

## Token Economics

- **Token**: HCLAW
- **Decimals**: 18 (like ETH)
- **Supply**: Minted via block rewards, with burn tracking
- **Fee Split**: 95% solver / 4% verifier / 1% burn

## Security

- **Honey Pots**: Protocol injects fake solutions to catch cheaters
- **Slashing**: Approving a honey pot = 100% stake slashed
- **Burn-to-Request**: Small burn required to submit jobs (anti-spam)

## Binaries

- **hardclaw**: Onboarding TUI (create/load wallet)
- **hardclaw-node**: Full node / verifier node (libp2p)
- **hardclaw-cli**: Interactive CLI for job submission (offline demo)
- **@hardclaw/mcp-server**: MCP server for agentic interaction (npm package)

## Development

```bash
# Run tests
cargo test

# Build release
cargo build --release

# Binaries
./target/release/hardclaw        # Onboarding TUI
./target/release/hardclaw-node   # Full node
./target/release/hardclaw-cli    # CLI tools

## CLI Notes

The CLI runs locally and prints what would be broadcast to the network. Full network submission and job lifecycle integration are still in progress.
```

## License

MIT License - see [LICENSE](LICENSE)
