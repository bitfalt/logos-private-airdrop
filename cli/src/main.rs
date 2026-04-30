use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use commands::{
    build_tree::{BuildTreeArgs, build_tree},
    export_claim::{ExportClaimArgs, export_claim},
    inspect::{InspectManifestArgs, inspect_manifest},
};

#[derive(Debug, Parser)]
#[command(name = "lp0003")]
#[command(about = "LP-0003 private allowlist / airdrop helper CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    BuildTree(BuildTreeArgs),
    ExportClaim(ExportClaimArgs),
    InspectManifest(InspectManifestArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::BuildTree(args) => build_tree(args),
        Command::ExportClaim(args) => export_claim(args),
        Command::InspectManifest(args) => inspect_manifest(args),
    }
}
