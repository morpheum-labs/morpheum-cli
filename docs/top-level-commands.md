**Comprehensive Guide: Morpheum CLI Top-Level Commands**  
**Purpose, Usage, Use Cases & Flows**  
**Version**: 1.0 (March 2026)

This guide explains the **exact purpose**, **recommended usage**, **real-world use cases**, and **internal flows** for every top-level command group in the Morpheum CLI.

### CLI Command Philosophy (Why the Structure Exists)

The Morpheum CLI follows a deliberate, user-centric design:

- **`tx` / `query`** prefix → For **on-chain modules** (identity, reputation, job, x402, etc.). These are low-level, precise blockchain operations.
- **Top-level commands** (no prefix) → For **high-level protocol and developer tools** (mcp, a2a, mwvm, bridge, agent, keys). These provide intuitive, powerful workflows that often combine multiple steps.

This split gives developers both raw power and delightful UX.

### Final Command Structure Overview

| Command Group     | Prefix      | Purpose Category                  | Example Command |
|-------------------|-------------|-----------------------------------|-----------------|
| mwvm              | Top-level   | Local simulation & developer tools (Pillar 1) | `morpheum mwvm infer` |
| mcp               | Top-level   | Model Context Protocol tools (Pillar 2) | `morpheum mcp call` |
| a2a               | Top-level   | Agent-to-Agent collaboration (Pillar 2) | `morpheum a2a delegate` |
| bank (xchain)     | `tx bank`   | Cross-chain deposits/withdrawals via Hyperlane | `morpheum tx bank deposit --chain evm:sepolia` |
| gmp               | `query gmp` | Hyperlane message delivery status | `morpheum query gmp delivery` |
| agent             | Top-level   | Unified agent lifecycle (Pillar 2+3) | `morpheum agent register --full` |
| keys              | Top-level   | Secure key management | `morpheum keys add` |
| x402              | `tx x402`   | Native autonomous payments (Pillar 2) | `morpheum tx x402 pay` (also automatic) |

---

### 1. `mwvm` – Portable Off-Chain Runtime & Developer Tools (Pillar 1)

**Purpose**:  
Gives developers full access to the rich, non-deterministic mwvm runtime locally (inference, simulation, debugging, multi-agent orchestration) without touching the chain.

**Primary Use Cases**:
- Rapid prototyping and testing of agents locally
- Debugging WASM execution step-by-step
- Running large multi-agent swarms for testing
- Local model inference before deploying on-chain

**How to Use**:
```bash
morpheum mwvm infer --model llama-3.1-8b-q4 --prompt "Hello world"
morpheum mwvm simulate --agent did:agent:trader --steps 50 --verbose
morpheum mwvm orchestrate --count 20 --task "analyze market data"
morpheum mwvm status
```

**Internal Flow**:
1. CLI parses arguments
2. Calls mwvm runtime via SDK (local mode)
3. Runs inference/simulation in wasmtime with continuous batching
4. Returns rich output (streaming supported)

---

### 2. `mcp` – Model Context Protocol (Pillar 2)

**Purpose**:  
Allows any MCP client (Claude, Cursor, VS Code, etc.) to use Morpheum agents as standard MCP tools.

**Primary Use Cases**:
- Connecting Claude Desktop to your agents as tools
- Building VS Code extensions that call Morpheum agents
- Testing MCP compatibility locally

**How to Use**:
```bash
morpheum mcp call did:agent:data-provider --tool search --input "BTC price"
morpheum mcp list-tools did:agent:research-bot
morpheum mcp status did:agent:alpha-trader
```

**Internal Flow**:
1. CLI connects to public MCP gateway (or local mwvm)
2. Translates request to native agent execution
3. Returns result with optional TEE/zkML proof

---

### 3. `a2a` – Agent2Agent Protocol (Pillar 2)

**Purpose**:  
Enables direct collaboration between agents using the standard A2A protocol.

**Primary Use Cases**:
- Delegating complex tasks to specialized agents
- Building agent swarms that talk to each other
- Cross-framework interoperability (Google ADK, LangGraph, etc.)

**How to Use**:
```bash
morpheum a2a delegate did:agent:alpha-trader --task "execute this order"
morpheum a2a discover "high-frequency trading"
morpheum a2a collaborate did:agent:research-bot --goal "market analysis"
```

**Internal Flow**:
1. CLI resolves target via agent_registry
2. Sends A2A envelope (with optional x402 payment)
3. Receives response with proof

---

### 4. `tx bank deposit/withdraw` – Cross-Chain Token Transfers (Pillar 4)

**Purpose**:  
Transfers tokens between external chains (Ethereum, Solana, etc.) and Morpheum via Hyperlane Warp Routes. Cross-chain is a capability applied to modules, not a separate top-level concept.

**Primary Use Cases**:
- Depositing USDC, ETH, SOL from external chains into Morpheum bank accounts
- Withdrawing assets from Morpheum back to external chains
- Funding margin accounts for perpetual trading

**How to Use**:
```bash
# Deposit USDC from Ethereum Sepolia to Morpheum
morpheum tx bank deposit --chain evm:sepolia --token USDC --amount 100

# Deposit native SOL from Solana Devnet to Morpheum
morpheum tx bank deposit --chain svm:devnet --token SOL --amount 0.5

# Withdraw ETH from Morpheum to Ethereum Sepolia
morpheum tx bank withdraw --chain evm:sepolia --token ETH --recipient 0x... --amount 10000000000000

# Check delivery status of a Hyperlane message
morpheum query gmp delivery --message-id 0xabc123...
```

**Supported Chains and Tokens**:

| Chain              | Spec          | Tokens        |
|--------------------|---------------|---------------|
| Ethereum Sepolia   | `evm:sepolia` | USDC, ETH     |
| Base Sepolia       | `evm:base-sepolia` | USDC     |
| Polygon Amoy       | `evm:polygon-amoy` | USDC     |
| Arbitrum Sepolia   | `evm:arbitrum-sepolia` | USDC  |
| Solana Devnet      | `svm:devnet`  | USDC, SOL     |

**Key Flags**:
- `--chain`: Chain specification in `<type>:<network>` format
- `--token`: Token symbol (USDC, ETH, SOL)
- `--amount`: Human-readable amount for deposits, raw amount for withdrawals
- `--recipient`: 32-byte hex recipient address
- `--chain-rpc`: Override the external chain's RPC URL
- `--from`: Key name to sign with (default: "default")

**Internal Flow (Deposit)**:
1. CLI resolves chain + token from SDK chain registry
2. Derives signer from keyring (mnemonic or raw private key)
3. Calls `quoteDispatch` for Hyperlane fee (ERC-20 only)
4. Calls `transferRemote` on the Warp Route contract
5. Returns transaction hash and Hyperlane message ID

**Internal Flow (Withdraw)**:
1. CLI resolves warp route contract from SDK registry
2. Builds CosmWasm `transfer_remote` execute message
3. Signs and broadcasts via Morpheum gRPC
4. Returns transaction hash

---

### 5. `agent` – High-Level Agent Operations (Pillar 2 + 3)

**Purpose**:  
The unified, user-friendly interface for agent lifecycle management.

**Primary Use Cases**:
- One-click full registration (`--full`)
- Quick interaction with any agent
- Checking comprehensive agent status

**How to Use**:
```bash
morpheum agent register --full --did "did:agent:alpha" --name "AlphaTrader"
morpheum agent interact did:agent:research-bot --task "analyze news"
morpheum agent status did:agent:alpha
```

**Internal Flow** (with `--full`):
1. Creates identity + memory root + reputation seed
2. Triggers exports to ERC-8004, MCP, A2A, DID, x402
3. Returns single success message

---

### 6. `keys` – Secure Key Management

**Purpose**:  
Manage human wallets and agent delegation keys (NativeSigner + AgentSigner with TradingKeyClaim).

**Primary Use Cases**:
- Adding keys for signing
- Managing agent delegation credentials
- Secure key lifecycle

**How to Use**:
```bash
morpheum keys add my-trader --mnemonic "..."
morpheum keys add trading-key --agent --owner-mnemonic "..."
morpheum keys list
morpheum keys delete my-trader
```

---

### 7. `x402` – Native Autonomous Payments (Pillar 2)

**Purpose**:  
Native implementation of the x402 payment standard.

**Two Ways to Use**:

**Standalone**:
```bash
morpheum tx x402 pay did:agent:alpha-trader 2500000 --memo "data subscription"
```

**Automatic (most powerful)**:
```bash
morpheum mcp call data-provider --tool search
# → Automatically detects 402 → pays using your default key
```

**Internal Flow (Automatic)**:
1. Higher-level command receives 402 response
2. CLI automatically builds & signs `MsgX402Pay`
3. Settlement happens via bank module
4. Original request is retried with receipt

---

### Summary: How Everything Connects

The CLI is designed so you rarely need the low-level `tx` commands. Most work happens through high-level top-level commands (`mcp`, `a2a`, `mwvm`, `bridge`, `agent`), with `x402` and `keys` working silently in the background.

This structure gives you:
- **Power** when you need it (`tx` / `query`)
- **Simplicity** for daily use (top-level commands)
- **Seamless payments** via automatic x402

You now have the complete picture of how the Morpheum CLI is structured and how to use every major command group effectively.

Ready for the next file or any refinements? Just let me know. 🚀