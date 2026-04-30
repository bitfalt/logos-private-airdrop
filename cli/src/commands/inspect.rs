use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use super::{ManifestFile, read_json};

#[derive(Debug, Args)]
pub struct InspectManifestArgs {
    pub manifest: PathBuf,
}

pub fn inspect_manifest(args: InspectManifestArgs) -> Result<()> {
    let manifest: ManifestFile = read_json(&args.manifest)?;
    println!("name={}", manifest.name);
    println!("distribution_id={}", manifest.distribution_id);
    println!("merkle_root={}", manifest.merkle_root);
    println!("allocation_policy={}", manifest.allocation_policy);
    println!("entries={}", manifest.entries.len());
    Ok(())
}
