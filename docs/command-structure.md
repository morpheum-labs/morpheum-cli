**✅ Comprehensive Design Document: Final CLI Command Structure**

### Core Principle (Why the split exists)

- **On-chain modules** (identity, reputation, validation, memory, vc, intent, marketplace, job, inference_registry, agent_registry, directory, interop, bank, x402)  
  → Always under **`tx`** and **`query`**  
  These are low-level, precise operations that touch the blockchain directly.

- **Protocol & Developer Experience layers** (MCP, A2A, mwvm simulation, Bridge/GMP gateways)  
  → Top-level commands (no `tx`/`query` prefix)  
  These are high-level, user-friendly interfaces that often combine multiple steps or talk to gateways.

- **High-level agent operations**  
  → Top-level (`agent register --full`, `agent interact`, etc.)

This split gives the **best of both worlds**: power for developers + intuitive UX for everyday use.

### Final Command Hierarchy (Locked)

| Category                  | Prefix          | Example Commands                                      | Reason |
|---------------------------|-----------------|-------------------------------------------------------|--------|
| On-chain modules          | `tx` / `query`  | `morpheum tx job create ...`<br>`morpheum tx x402 pay ...`<br>`morpheum query job status ...` | Direct blockchain access |
| Protocol layers           | Top-level       | `morpheum mcp call ...`<br>`morpheum a2a delegate ...`<br>`morpheum bridge send-proof ...`<br>`morpheum mwvm infer ...` | High-level gateways & developer tools |
| Keys & Config             | Top-level       | `morpheum keys add ...`<br>`morpheum config` | Utility commands |

### How x402 Fits (The Key Question)

**x402** is **not** a full module like `job` or `identity`. It is a **payment primitive**.

- **Explicit use**: `morpheum tx x402 pay ...` (standalone)
- **Automatic use**: Triggered invisibly by other commands (MCP calls, A2A delegation, Job creation, Bridge exports, etc.) when a 402 response is received.

Example automatic flow:
```bash
morpheum mcp call did:agent:data-provider --tool search
# → CLI detects 402 → automatically runs x402 payment using your default key
```

This is the exact pattern described in pillar-2.md and thesis.md.

### Concrete Examples (How Commands Actually Look)

**On-chain (with `tx`)**:
```bash
morpheum tx job create --provider did:agent:evaluator --budget 5000000
morpheum tx x402 pay did:agent:alpha-trader 2500000 --memo "data subscription"
morpheum query job status <job-id>
```

**Protocol layers (no prefix)**:
```bash
morpheum bridge send-proof --agent did:agent:trader --to-chain ethereum
morpheum mcp call did:agent:data-provider --tool search
morpheum a2a delegate did:agent:alpha-trader --task "analyze market"
morpheum mwvm infer --model llama-3.1 --prompt "Hello"
```

**Agent high-level**:
```bash
morpheum agent register --full --did "did:agent:alpha-trader" --name "AlphaTrader"
morpheum agent interact did:agent:research-bot --task "summarize latest news"
```

