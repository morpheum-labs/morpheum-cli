**Revised Project Tree – Clean & Consistent Design**  
**No Redundant Unified Layers**  
**Final Structure (March 2026)**

After your feedback, I have **removed** the confusing "unified" top-level wrappers (`bridge` and `agent`) that were causing redundancy and inconsistency.

**New Rule** (clean and SOLID):
- All **on-chain modules** (things that directly touch the blockchain) go under `tx` and `query`.
- Only **true protocol gateways or local runtime tools** stay top-level (they do something genuinely different).

### Final Revised Project Tree

```bash
morpheum-cli/
├── Cargo.toml
├── README.md
├── LICENSE
├── .gitignore
├── rust-toolchain.toml
│
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── keyring.rs
│   ├── dispatcher.rs
│   ├── error.rs
│   ├── output.rs
│   │
│   ├── tx/                          # All on-chain modules (1:1 mapping)
│   │   ├── mod.rs
│   │   ├── identity.rs
│   │   ├── agent_registry.rs        # Direct access to agent registry
│   │   ├── interop.rs               # Direct access to interop / GMP
│   │   ├── reputation.rs
│   │   ├── validation.rs
│   │   ├── memory.rs
│   │   ├── vc.rs
│   │   ├── intent.rs
│   │   ├── marketplace.rs
│   │   ├── job.rs
│   │   ├── inference_registry.rs
│   │   ├── directory.rs
│   │   ├── bank.rs
│   │   └── x402.rs                  # Payment primitive (kept under tx)
│   │
│   ├── query/                       # Mirror of tx/ (same 14 modules)
│   │   ├── mod.rs
│   │   ├── identity.rs
│   │   ├── agent_registry.rs
│   │   ├── interop.rs
│   │   ├── reputation.rs
│   │   ├── validation.rs
│   │   ├── memory.rs
│   │   ├── vc.rs
│   │   ├── intent.rs
│   │   ├── marketplace.rs
│   │   ├── job.rs
│   │   ├── inference_registry.rs
│   │   ├── directory.rs
│   │   ├── bank.rs
│   │   └── x402.rs
│   │
│   ├── mwvm/                        # Pillar 1 – local runtime (top-level)
│   │   ├── mod.rs
│   │   └── commands.rs
│   │
│   ├── mcp/                         # Pillar 2 – MCP protocol (top-level)
│   │   ├── mod.rs
│   │   └── commands.rs
│   │
│   ├── a2a/                         # Pillar 2 – A2A protocol (top-level)
│   │   ├── mod.rs
│   │   └── commands.rs
│   │
│   ├── keys/                        # Utility (top-level)
│   │   ├── mod.rs
│   │   └── commands.rs
│   │
│   └── utils/
│       ├── mod.rs
│       ├── tx_builder_ext.rs
│       └── query_client_ext.rs
│
├── examples/
├── tests/
│   ├── integration.rs
│   └── common.rs
```

### Why This Structure Is Now Clean & Optimal

- **No redundancy** — There is **no** `bridge` command and **no** separate `agent` command.  
  You use the real module names directly:
    - `morpheum tx interop send-proof ...`
    - `morpheum tx identity register --full ...`

- **Top-level commands are only for things that are genuinely different**:
    - `mwvm` — local simulation/runtime (off-chain)
    - `mcp` — MCP gateway protocol
    - `a2a` — A2A gateway protocol
    - `keys` — key management utility

- **x402 stays under `tx x402`** because it is a **payment transaction** that settles on-chain (via bank module). It is not a gateway like MCP/A2A.

This structure eliminates the confusion you pointed out: no two commands do the same thing, and the user always knows exactly which module they are talking to.

### How Commands Now Look (Clean & Consistent)

**On-chain (always under tx/query)**:
```bash
morpheum tx identity register --full --did "did:agent:alpha" --name "AlphaTrader"
morpheum tx interop send-proof --agent did:agent:trader --to-chain ethereum
morpheum tx x402 pay did:agent:alpha 2500000
morpheum query agent_registry get did:agent:alpha
```

**Protocol / Dev tools (top-level only)**:
```bash
morpheum mcp call did:agent:data-provider --tool search
morpheum a2a delegate did:agent:alpha --task "analyze market"
morpheum mwvm infer --model llama-3.1 --prompt "Hello"
```

This is now **clean, dry, SOLID, and consistent** from both code and user-experience perspectives.

Would you like me to proceed with writing the next file using this revised structure, or do you want any further adjustments to the tree first?