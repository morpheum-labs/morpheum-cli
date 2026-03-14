//! Morpheum CLI — human-friendly command-line tool for the Morpheum platform.
//!
//! Issue credentials, manage agents, trade markets, stake, govern — all from your terminal.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use comfy_table::{Cell, Color, Table};
use morpheum_sdk_native::prelude::*;
use morpheum_sdk_native::vc::{VcClaims, VcIssueBuilder};
use morpheum_sdk_native::signing::{signer::Signer, types::Signature};
use std::path::PathBuf;

use morpheum_cli::{config, parse_agent_hex, parse_permissions};

/// Prompts for y/N confirmation. Returns true if user enters y/yes.
fn confirm(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    let mut buf = String::new();
    print!("{}", prompt);
    io::stdout().flush().context("flush stdout")?;
    io::stdin().read_line(&mut buf).context("read stdin")?;
    let s = buf.trim().to_lowercase();
    Ok(s == "y" || s == "yes")
}

#[derive(Parser)]
#[command(
    name = "morpheum",
    about = "Morpheum CLI – issue credentials, manage agents, trade, stake, govern",
    version,
    author = "Morpheum Labs",
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    /// Path to config file (default: ~/.morpheum/config.toml)
    #[arg(long, env = "MORPHEUM_CONFIG", default_value = "~/.morpheum/config.toml")]
    config: PathBuf,

    /// Output format: human (default) or json
    #[arg(long, default_value = "human")]
    output: String,

    /// Verbose logging (sets RUST_LOG=debug if unset)
    #[arg(long, short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup wizard: mnemonic, network, etc.
    Setup,
    /// Verifiable Credentials commands
    #[command(subcommand)]
    Vc(VcCommands),
    /// Agent & delegation management
    #[command(subcommand)]
    Agent(AgentCommands),
    /// Market & trading commands
    #[command(subcommand)]
    Market(MarketCommands),
}

#[derive(Subcommand)]
enum VcCommands {
    /// Issue a new VC with TradingKeyClaim for an agent
    Issue(VcIssueArgs),
}

#[derive(Parser)]
struct VcIssueArgs {
    /// Agent address (recipient of delegation)
    #[arg(long)]
    agent: String,

    /// Permissions bitflags (e.g. TRADE,STAKE,VOTE)
    #[arg(long, default_value = "TRADE")]
    permissions: String,

    /// Max daily USD spend limit
    #[arg(long, default_value = "5000")]
    max_usd: u64,

    /// Expiry days from now
    #[arg(long, default_value = "30")]
    expiry_days: u64,

    /// Actually broadcast (default is dry-run only)
    #[arg(long)]
    broadcast: bool,

    /// Skip confirmation prompt when broadcasting (for scripts)
    #[arg(long, short = 'y')]
    yes: bool,
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Create and embed a new TradingKeyClaim (for delegation)
    CreateClaim(AgentCreateClaimArgs),
}

#[derive(Parser)]
struct AgentCreateClaimArgs {
    /// Agent address (subject of the claim)
    #[arg(long)]
    agent: String,

    /// Permissions (e.g. TRADE,STAKE)
    #[arg(long, default_value = "TRADE")]
    permissions: String,

    /// Max daily USD limit
    #[arg(long, default_value = "5000")]
    max_usd: u64,

    /// Expiry in days
    #[arg(long, default_value = "30")]
    expiry_days: u64,
}

#[derive(Subcommand)]
enum MarketCommands {
    /// Create a new order
    Order(MarketOrderArgs),
}

#[derive(Parser)]
struct MarketOrderArgs {
    /// Market ID
    #[arg(long)]
    market_id: u64,
    /// Side: buy / sell
    #[arg(long)]
    side: String,
    /// Size / quantity
    #[arg(long)]
    size: f64,
    /// Limit price
    #[arg(long)]
    price: f64,
}

fn is_json_output(output: &str) -> bool {
    output.eq_ignore_ascii_case("json")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose && std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "morpheum_cli=debug,info");
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config_path = PathBuf::from(
        shellexpand::tilde(&cli.config.to_string_lossy()).into_owned(),
    );

    let json_mode = is_json_output(&cli.output);

    if !json_mode {
        println!("{}", "Morpheum CLI v0.1.0".bright_green().bold());
    }

    match cli.command {
        Commands::Setup => run_setup(&config_path, json_mode).await,
        Commands::Vc(cmd) => run_vc(cmd, &config_path, &cli.output).await,
        Commands::Agent(cmd) => run_agent(cmd, &config_path, &cli.output).await,
        Commands::Market(cmd) => run_market(cmd, &config_path, &cli.output).await,
    }
}

async fn run_setup(config_path: &PathBuf, json_mode: bool) -> Result<()> {
    let cfg = config::Config::load(config_path)?;

    if json_mode {
        println!("{{\"config_path\":\"{}\"}}", config_path.display());
        return Ok(());
    }

    println!("Running setup wizard...");
    println!("  Config path: {}", config_path.display());

    println!("\nEnter your BIP-39 mnemonic to validate (input hidden, NOT saved):");
    let mnemonic = rpassword::prompt_password("Mnemonic: ")
        .context("Failed to read mnemonic")?;

    let mnemonic = mnemonic.trim();
    if mnemonic.split_whitespace().count() < 12 {
        anyhow::bail!("Mnemonic must be at least 12 words");
    }

    NativeSigner::from_mnemonic(mnemonic, "")
        .context("Invalid mnemonic - check word count and spelling")?;

    cfg.save(config_path)?;

    println!("{} Config saved (rpc, chain_id). Mnemonic NOT stored.", "✓".bright_green());
    println!();
    println!("{} For production, set the mnemonic in your environment:", "→".bright_cyan());
    println!("   export MORPHEUM_MNEMONIC=\"your twelve word mnemonic phrase here\"");
    println!();
    println!("   Or add to ~/.bashrc / ~/.zshrc (never commit!)");
    Ok(())
}

async fn run_vc(
    cmd: VcCommands,
    config_path: &PathBuf,
    output: &str,
) -> Result<()> {
    match cmd {
        VcCommands::Issue(args) => {
            let cfg = config::Config::load(config_path)?;
            let permissions = parse_permissions(&args.permissions);
            let dry_run = !args.broadcast;

            if dry_run {
                if is_json_output(output) {
                    println!(
                        "{{\"dry_run\":true,\"agent\":\"{}\",\"max_usd\":{},\"expiry_days\":{},\"permissions\":{}}}",
                        args.agent, args.max_usd, args.expiry_days, permissions
                    );
                } else {
                    println!(
                        "{} Issuing VC → Agent: {} | Limit: ${}/day | Expiry: {}d | Permissions: {}",
                        "→".bright_cyan(),
                        args.agent,
                        args.max_usd,
                        args.expiry_days,
                        args.permissions
                    );
                    println!("{} (Dry-run – TX not broadcast)", "✓".bright_yellow());
                }
                return Ok(());
            }

            if !is_json_output(output) {
                println!("\n{}", "WARNING: This will broadcast a real transaction!".bright_red().bold());
            }
            if !args.yes && !confirm("Continue? [y/N] ")? {
                if is_json_output(output) {
                    println!(r#"{{"cancelled":true,"reason":"user"}}"#);
                } else {
                    println!("Cancelled.");
                }
                return Ok(());
            }

            let mnemonic = cfg.mnemonic().filter(|s| !s.trim().is_empty()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Mnemonic required. Set MORPHEUM_MNEMONIC env (never stored in config). \
                     Example: export MORPHEUM_MNEMONIC=\"word1 word2 ... word12\""
                )
            })?;

            let signer = NativeSigner::from_mnemonic(&mnemonic, "")
                .context("Invalid mnemonic")?;

            let subject_bytes = parse_agent_hex(&args.agent)?;
            let subject = morpheum_sdk_native::signing::types::AccountId(subject_bytes);
            let issuer = signer.account_id();

            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let expiry_secs = now_secs + (args.expiry_days as u64) * 86_400;

            let sdk = MorpheumSdk::new(&cfg.rpc_endpoint, cfg.chain_id.as_str());

            // VcClaims for on-chain MsgIssue. Defaults: all pairs, 1% slippage, 10x position.
            // TODO: expose --max-slippage, --allowed-pairs as CLI args for v0.2
            let vc_claims = VcClaims {
                max_daily_usd: args.max_usd,
                allowed_pairs_bitflags: u64::MAX,
                max_slippage_bps: 100,
                max_position_usd: args.max_usd.saturating_mul(10),
                custom_constraints: None,
            };

            let claim_for_digest = TradingKeyClaim {
                issuer: issuer.clone(),
                subject: subject.clone(),
                permissions,
                max_daily_usd: args.max_usd,
                expiry_timestamp: expiry_secs,
                nonce_sub_range_start: 0,
                nonce_sub_range_end: u32::MAX,
                signature: Signature::Ed25519([0u8; 64]),
            };
            let digest = claim_for_digest.claim_digest();
            let sig = signer.sign_raw(&digest);
            let issuer_sig = match &sig {
                Signature::Ed25519(b) => b.to_vec(),
                Signature::Secp256k1(b) => b.to_vec(),
                Signature::Schnorr(b) => b.to_vec(),
            };

            let issue_req = VcIssueBuilder::new()
                .issuer(AccountId::new(issuer.0))
                .subject(AccountId::new(subject.0))
                .claims(vc_claims)
                .expiry(expiry_secs)
                .issuer_signature(issuer_sig)
                .build()
                .map_err(|e| anyhow::anyhow!("VC issue build failed: {}", e))?;

            let tx = TxBuilder::new(signer)
                .chain_id(cfg.chain_id.as_str())
                .memo("morpheum-cli vc issue")
                .add_message(issue_req.to_any())
                .sign()
                .await
                .map_err(|e| anyhow::anyhow!("Sign failed: {}", e))?;

            let result = sdk
                .transport()
                .broadcast_tx(tx.raw_bytes().to_vec())
                .await;

            match result {
                Ok(br) => {
                    if is_json_output(output) {
                        println!(
                            "{{\"success\":true,\"txhash\":\"{}\"}}",
                            br.txhash
                        );
                    } else {
                        println!("{} VC issued successfully!", "✓".bright_green());
                        println!("  TxHash: {}", br.txhash);
                    }
                }
                Err(e) => {
                    if is_json_output(output) {
                        println!("{{\"success\":false,\"error\":\"{}\"}}", e);
                    } else {
                        println!("{} Broadcast failed: {}", "✗".bright_red(), e);
                    }
                    anyhow::bail!("Broadcast failed: {}", e);
                }
            }
            Ok(())
        }
    }
}

async fn run_agent(
    cmd: AgentCommands,
    _config_path: &PathBuf,
    output: &str,
) -> Result<()> {
    let _ = _config_path;
    match cmd {
        AgentCommands::CreateClaim(args) => {
            if is_json_output(output) {
                println!(
                    "{{\"agent\":\"{}\",\"max_usd\":{},\"expiry_days\":{},\"permissions\":\"{}\"}}",
                    args.agent, args.max_usd, args.expiry_days, args.permissions
                );
                return Ok(());
            }

            let mut table = Table::new();
            table.set_header(vec![
                Cell::new("Field").fg(Color::Cyan),
                Cell::new("Value").fg(Color::Green),
            ]);
            table.add_row(vec!["Agent", &args.agent]);
            table.add_row(vec!["Max USD/day", &args.max_usd.to_string()]);
            table.add_row(vec!["Expiry (days)", &args.expiry_days.to_string()]);
            table.add_row(vec!["Permissions", &args.permissions]);
            println!("{}", table);
            println!("{} (Use `morpheum vc issue` for full issuance)", "→".bright_yellow());
            Ok(())
        }
    }
}

async fn run_market(
    cmd: MarketCommands,
    _config_path: &PathBuf,
    output: &str,
) -> Result<()> {
    let _ = _config_path;
    match cmd {
        MarketCommands::Order(args) => {
            if is_json_output(output) {
                println!(
                    "{{\"market_id\":{},\"side\":\"{}\",\"size\":{},\"price\":{}}}",
                    args.market_id, args.side, args.size, args.price
                );
                return Ok(());
            }

            let mut table = Table::new();
            table.set_header(vec![
                Cell::new("Field").fg(Color::Cyan),
                Cell::new("Value").fg(Color::Green),
            ]);
            table.add_row(vec!["Market ID", &args.market_id.to_string()]);
            table.add_row(vec!["Side", &args.side]);
            table.add_row(vec!["Size", &args.size.to_string()]);
            table.add_row(vec!["Price", &args.price.to_string()]);
            println!("{}", table);
            println!("{} (Order placement – implementation coming soon)", "→".bright_yellow());
            Ok(())
        }
    }
}
