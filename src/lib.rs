//! Morpheum CLI library — config, parsing, output formatting.

use anyhow::Context;

pub mod config;

/// Parse hex string to 32-byte AccountId (strips 0x, pads/truncates to 32 bytes).
pub fn parse_agent_hex(hex_str: &str) -> anyhow::Result<[u8; 32]> {
    let s = hex_str.strip_prefix("0x").unwrap_or(hex_str).trim();
    let bytes = hex::decode(s).context("Invalid hex in agent address")?;
    let mut out = [0u8; 32];
    let len = bytes.len().min(32);
    let start = if bytes.len() >= 32 { 0 } else { 32 - len };
    out[start..start + len].copy_from_slice(&bytes[..len]);
    Ok(out)
}

/// Parse permissions string (TRADE,STAKE,VOTE) to bitflags.
pub fn parse_permissions(s: &str) -> u64 {
    let mut flags = 0u64;
    for part in s.split(',') {
        let part = part.trim().to_uppercase();
        flags |= match part.as_str() {
            "TRADE" => 1 << 0,
            "STAKE" => 1 << 1,
            "VOTE" => 1 << 2,
            "EVALUATE" => 1 << 3,
            _ => 0,
        };
    }
    if flags == 0 {
        1 << 0 // default TRADE
    } else {
        flags
    }
}
