//! Clash Chain Patcher CLI
//!
//! A complete command-line interface for managing Clash proxy chains.
//!
//! Usage:
//!   ccp info <config.yaml>              - Show rules groups and proxy info
//!   ccp apply <config.yaml> [options]   - Apply chain proxies + rewrite rules
//!   ccp rules <config.yaml> [options]   - Rewrite rules only (no chain creation)

use clap::{Parser, Subcommand};
use clash_chain_patcher::merger::{ClashConfigMerger, MergerConfig};
use clash_chain_patcher::patcher;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "ccp",
    about = "Clash Chain Patcher - Add SOCKS5 proxy chains to Clash configurations",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show config info: proxy groups referenced in rules, proxy nodes count
    Info {
        /// Path to Clash YAML config file
        config: PathBuf,
    },

    /// Apply full chain patch: add SOCKS5 proxy, create relay chains, rewrite rules
    Apply {
        /// Path to Clash YAML config file
        config: PathBuf,

        /// SOCKS5 proxy (formats: host:port:user:pass or user:pass@host:port or host:port)
        #[arg(short, long)]
        proxy: String,

        /// Rewrite rules: replace proxy group with Chain-Selector or Chain-Auto
        /// Format: "GroupName=Chain-Selector" or "GroupName=Chain-Auto"
        /// Use "auto" to auto-detect main group and replace with Chain-Selector
        #[arg(short, long)]
        rewrite: Option<Vec<String>>,

        /// Skip creating backup
        #[arg(long)]
        no_backup: bool,

        /// Chain suffix (default: "-Chain")
        #[arg(long, default_value = "-Chain")]
        suffix: String,
    },

    /// Rewrite rules only (no chain proxy creation)
    Rules {
        /// Path to Clash YAML config file
        config: PathBuf,

        /// Rule replacements: "GroupName=Chain-Selector" or "GroupName=Chain-Auto"
        /// Use "auto" to auto-detect and replace main group with Chain-Selector
        #[arg(short, long, required = true)]
        rewrite: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { config } => cmd_info(&config),
        Commands::Apply { config, proxy, rewrite, no_backup, suffix } => {
            cmd_apply(&config, &proxy, rewrite, no_backup, &suffix);
        }
        Commands::Rules { config, rewrite } => cmd_rules(&config, rewrite),
    }
}

/// Show config info
fn cmd_info(config_path: &PathBuf) {
    let content = read_config(config_path);

    // Rule groups
    let groups = patcher::extract_rule_groups(&content);
    let total_rules: usize = groups.iter().map(|g| g.count).sum();

    println!("Config: {}", config_path.display());
    println!();
    println!("Rules: {} groups, {} total rules", groups.len(), total_rules);
    println!("{:<40} {:>8}", "Group", "Rules");
    println!("{}", "-".repeat(50));
    for group in &groups {
        println!("{:<40} {:>8}", group.name, group.count);
    }

    // Proxy nodes
    let config: serde_yaml::Value = serde_yaml::from_str(&content).unwrap_or_default();
    if let Some(proxies) = config.get("proxies").and_then(|v| v.as_sequence()) {
        println!();
        println!("Proxy nodes: {}", proxies.len());
    }

    // Proxy groups
    if let Some(pg) = config.get("proxy-groups").and_then(|v| v.as_sequence()) {
        println!("Proxy groups: {}", pg.len());
        for g in pg {
            let name = g.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let gtype = g.get("type").and_then(|v| v.as_str()).unwrap_or("?");
            let count = g.get("proxies").and_then(|v| v.as_sequence()).map(|s| s.len()).unwrap_or(0);
            println!("  {} ({}, {} proxies)", name, gtype, count);
        }
    }
}

/// Apply full chain patch
fn cmd_apply(config_path: &PathBuf, proxy_str: &str, rewrite: Option<Vec<String>>, no_backup: bool, suffix: &str) {
    // Parse proxy
    let proxy = patcher::parse_proxy_string(proxy_str).unwrap_or_else(|| {
        eprintln!("Error: Invalid proxy format: {}", proxy_str);
        eprintln!("Formats: host:port:user:pass | user:pass@host:port | host:port");
        process::exit(1);
    });

    println!("Config: {}", config_path.display());
    println!("Proxy:  {}:{}", proxy.host, proxy.port);
    if let Some(ref u) = proxy.username {
        println!("Auth:   {}:***", u);
    }
    println!();

    // Step 1: Apply chain merge
    let merger_config = MergerConfig {
        proxy_name: "Local-Chain-Proxy".to_string(),
        proxy_host: proxy.host.clone(),
        proxy_port: proxy.port,
        proxy_username: proxy.username.clone(),
        proxy_password: proxy.password.clone(),
        create_backup: !no_backup,
        insert_at_beginning: true,
        chain_suffix: suffix.to_string(),
    };

    let merger = ClashConfigMerger::with_config(merger_config);
    match merger.merge(config_path) {
        Ok(result) => {
            println!("Chain merge:");
            println!("  Proxy added: {}", result.proxy_added);
            println!("  Chains created: {}", result.chains_created);
            println!("  Groups updated: {}", result.groups_updated);
            if let Some(backup) = result.backup_path {
                println!("  Backup: {}", backup.display());
            }
            for w in &result.warnings {
                println!("  Warning: {}", w);
            }
        }
        Err(e) => {
            eprintln!("Error: Chain merge failed: {}", e);
            process::exit(1);
        }
    }

    // Step 2: Rewrite rules if requested
    if let Some(rewrite_args) = rewrite {
        println!();
        let replacements = parse_rewrite_args(&rewrite_args, config_path);
        apply_rule_rewrites(config_path, &replacements);
    }

    println!();
    println!("Done.");
}

/// Rewrite rules only
fn cmd_rules(config_path: &PathBuf, rewrite_args: Vec<String>) {
    println!("Config: {}", config_path.display());
    println!();

    let replacements = parse_rewrite_args(&rewrite_args, config_path);
    apply_rule_rewrites(config_path, &replacements);

    println!();
    println!("Done.");
}

/// Parse rewrite arguments into a replacement map
fn parse_rewrite_args(args: &[String], config_path: &PathBuf) -> HashMap<String, String> {
    let mut replacements = HashMap::new();

    for arg in args {
        if arg == "auto" {
            // Auto-detect: find main proxy group (not DIRECT/REJECT, highest count)
            let content = read_config(config_path);
            let groups = patcher::extract_rule_groups(&content);
            if let Some(main) = groups.iter().find(|g| g.name != "DIRECT" && g.name != "REJECT") {
                println!("Auto-detected main group: {} ({} rules)", main.name, main.count);
                replacements.insert(main.name.clone(), "Chain-Selector".to_string());
            } else {
                eprintln!("Warning: No non-DIRECT/REJECT rule group found for auto-rewrite");
            }
        } else if let Some((from, to)) = arg.split_once('=') {
            let to = to.trim();
            if to != "Chain-Selector" && to != "Chain-Auto" {
                eprintln!("Warning: Invalid target '{}', use Chain-Selector or Chain-Auto", to);
                continue;
            }
            replacements.insert(from.trim().to_string(), to.to_string());
        } else {
            eprintln!("Warning: Invalid rewrite format '{}', use 'GroupName=Chain-Selector'", arg);
        }
    }

    replacements
}

/// Apply rule rewrites to a config file
fn apply_rule_rewrites(config_path: &PathBuf, replacements: &HashMap<String, String>) {
    if replacements.is_empty() {
        println!("No rule rewrites to apply.");
        return;
    }

    let content = read_config(config_path);

    println!("Rules rewrite:");
    for (from, to) in replacements {
        println!("  {} -> {}", from, to);
    }

    let (output, count) = patcher::rewrite_rules_text(&content, replacements);
    if count > 0 {
        std::fs::write(config_path, output).unwrap_or_else(|e| {
            eprintln!("Error: Failed to write config: {}", e);
            process::exit(1);
        });
        println!("  Rewritten: {} rules", count);
    } else {
        println!("  No rules matched for rewrite.");
    }
}

/// Read config file content or exit with error
fn read_config(path: &PathBuf) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error: Cannot read {}: {}", path.display(), e);
        process::exit(1);
    })
}
