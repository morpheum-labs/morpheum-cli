### The Two-Layer Architecture

The CLI is built with **two distinct layers** on purpose:

| Layer | Prefix | Purpose | Examples | Maps to |
|-------|--------|--------|----------|--------|
| **Low-level** (Plumbing) | `tx` / `query` | Direct, precise access to **individual on-chain modules** | `morpheum tx interop ...`<br>`morpheum query agentreg ...` | 1:1 with actual Mormcore modules (`interop`, `agentreg`, `identity`, etc.) |
| **High-level** (Porcelain / UX) | Top-level (no prefix) | User-friendly, workflow-oriented commands that **orchestrate** multiple modules | <br>`morpheum mcp ...`<br>`morpheum a2a ...`<br>`morpheum mwvm ...` | Convenience facades on top of low-level modules |

This is the **standard "porcelain vs plumbing"** pattern used in Git, Docker, Cargo, and modern CLIs. It is **not** redundancy — it is deliberate layering for two different audiences.

### Exact Mapping (No Overlap)

| Top-level CLI Command | What it actually does | Which on-chain module(s) it uses internally | Why it exists |
|-----------------------|-----------------------|---------------------------------------------|--------------|
| `mcp`                 | MCP protocol interaction | `agentreg` + `interop` + MCP gateway | Matches how people use Claude/Cursor |
| `a2a`                 | Agent-to-Agent collaboration | `agentreg` + `interop` + A2A gateway | Matches how people use LangGraph/ADK |
| `mwvm`                | Local simulation & developer tools | `mwvm` runtime (Pillar 1) | Developer experience (Pillar 1) |

### Why This Is Optimal (Not Redundant)

- **Low-level (`tx` / `query`)**: For power users, scripts, and automation. You can call `tx interop` directly when you need precise control.
- **High-level (top-level)**: For everyday users and quick workflows. You rarely want to type `tx interop send-proof` — `bridge send-proof` is much clearer.

This gives you **both**:
- Raw power when needed
- Intuitive UX for common tasks

**No redundancy** — they serve different purposes and call the same underlying modules where appropriate.

### Concrete Examples

**Low-level (direct module access)**:
```bash
morpheum tx interop send-proof --agent did:agent:trader --to-chain ethereum
morpheum query agentreg get did:agent:alpha
```

**High-level (convenient UX)**:
```bash
morpheum bridge send-proof --agent did:agent:trader --to-chain ethereum
morpheum agent register --full --did did:agent:alpha --name "AlphaTrader"
```

The high-level commands are **facades** — they orchestrate multiple modules and add smart defaults, automatic x402 payments, nice output, etc.

This is the same pattern used in `gh` (GitHub CLI), `cargo`, and every successful modern CLI.

### Summary

- `bridge` ≠ `interop` module → `bridge` is the **user-friendly wrapper** around the `interop` module.
- `agent` ≠ `agentreg` module → `agent` is the **unified orchestration** layer that uses `agentreg` + `identity` + others.
- **No redundancy** — two different abstraction levels for two different use cases.

This design is **intentional**, **optimal**, and follows industry best practices.

The project tree and all files I have written follow this exact layered pattern.

Would you like me to:
1. Continue with the next file using this confirmed structure, or
2. Adjust anything (e.g. rename `bridge` to `interop` for 1:1 mapping)?

Just say the word and I’ll proceed. 🚀