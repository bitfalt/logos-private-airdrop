use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use super::{build_manifest, write_json};

#[derive(Debug, Args)]
pub struct BuildTreeArgs {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
}

pub fn build_tree(args: BuildTreeArgs) -> Result<()> {
    let manifest = build_manifest(&args.input)?;
    write_json(&args.out, &manifest)?;
    println!(
        "manifest={} merkle_root={} entries={}",
        args.out.display(),
        manifest.merkle_root,
        manifest.entries.len()
    );
    Ok(())
}
