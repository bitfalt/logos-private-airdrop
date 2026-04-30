use std::{path::Path, process::Command};

const ELIGIBILITY_30_ROOT: &str =
    "7474e7b554a2542ccb6886dcda9cc3b7d92b1a92336e08f4225d0bef3ec0782e";

#[test]
fn manifest_and_claim_commands_emit_locked_root() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli crate has a workspace parent");
    let out_dir = repo_root
        .join("target")
        .join(format!("lp0003-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();

    let manifest = out_dir.join("eligibility-30.manifest.json");
    let claim = out_dir.join("claim-0.json");
    let fixture = repo_root.join("tests/fixtures/eligibility-30.json");

    let build = lp0003(repo_root)
        .args([
            "build-tree",
            "--input",
            fixture.to_str().unwrap(),
            "--out",
            manifest.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "build-tree failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(manifest.exists());
    assert!(String::from_utf8_lossy(&build.stdout).contains(ELIGIBILITY_30_ROOT));

    let export = lp0003(repo_root)
        .args([
            "export-claim",
            "--manifest",
            manifest.to_str().unwrap(),
            "--index",
            "0",
            "--out",
            claim.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        export.status.success(),
        "export-claim failed: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    assert!(claim.exists());

    let inspect = lp0003(repo_root)
        .args(["inspect-manifest", manifest.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        inspect.status.success(),
        "inspect-manifest failed: {}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    assert!(String::from_utf8_lossy(&inspect.stdout).contains(ELIGIBILITY_30_ROOT));
}

fn lp0003(repo_root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lp0003"));
    command.current_dir(repo_root);
    command
}
