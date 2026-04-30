use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use super::{ManifestFile, claim_from_manifest, read_json, write_json};

#[derive(Debug, Args)]
pub struct ExportClaimArgs {
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long)]
    pub index: usize,
    #[arg(long)]
    pub out: PathBuf,
}

pub fn export_claim(args: ExportClaimArgs) -> Result<()> {
    let manifest: ManifestFile = read_json(&args.manifest)?;
    let claim = claim_from_manifest(&manifest, args.index)?;
    write_json(&args.out, &claim)?;
    println!(
        "claim={} index={} nullifier={}",
        args.out.display(),
        args.index,
        claim.nullifier
    );
    Ok(())
}
