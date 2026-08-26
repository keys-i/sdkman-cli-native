use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use support::{TestCandidate, VirtualEnv};

mod support;

fn command(sdkman_dir: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("default"));
    command.env("SDKMAN_DIR", sdkman_dir);
    command
}

fn environment() -> tempfile::TempDir {
    support::virtual_env(VirtualEnv {
        cli_version: "0.0.1".to_owned(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1", "0.0.2"],
            current_version: "0.0.1",
        }],
    })
}

#[test]
fn sets_same_or_other_installed_version_as_default() {
    for (version, expected) in [
        ("0.0.1", "Running scala 0.0.1"),
        ("0.0.2", "Running scala 0.0.2"),
    ] {
        let sdkman_dir = environment();
        command(sdkman_dir.path())
            .args(["scala", version])
            .assert()
            .success()
            .stdout(format!(
                "setting scala {version} as the default version for all shells.\n"
            ))
            .stderr("");

        let current = sdkman_dir.path().join("candidates/scala/current");
        assert_eq!(
            fs::canonicalize(&current).unwrap(),
            fs::canonicalize(sdkman_dir.path().join("candidates/scala").join(version)).unwrap()
        );
        assert!(fs::read_to_string(current.join("bin/scala"))
            .unwrap()
            .contains(expected));
    }
}

#[test]
fn failed_default_leaves_the_old_current_in_place() {
    let sdkman_dir = environment();
    let current = sdkman_dir.path().join("candidates/scala/current");
    let old = fs::canonicalize(&current).unwrap();

    command(sdkman_dir.path())
        .args(["scala", "0.0.3"])
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr("scala 0.0.3 is not installed on your system\n");

    assert_eq!(fs::canonicalize(current).unwrap(), old);
}

#[test]
fn replaces_a_dangling_current_link() {
    let sdkman_dir = environment();
    let current = sdkman_dir.path().join("candidates/scala/current");
    fs::remove_dir_all(sdkman_dir.path().join("candidates/scala/0.0.1")).unwrap();

    command(sdkman_dir.path())
        .args(["scala", "0.0.2"])
        .assert()
        .success()
        .stdout("setting scala 0.0.2 as the default version for all shells.\n");

    assert_eq!(
        fs::canonicalize(current).unwrap(),
        fs::canonicalize(sdkman_dir.path().join("candidates/scala/0.0.2")).unwrap()
    );
}

#[test]
fn recovers_from_a_stale_backup() {
    let sdkman_dir = environment();
    let candidate = sdkman_dir.path().join("candidates/scala");
    let current = candidate.join("current");
    let backup = candidate.join("current-old");
    fs::create_dir(&backup).unwrap();

    command(sdkman_dir.path())
        .args(["scala", "0.0.2"])
        .assert()
        .success()
        .stdout("setting scala 0.0.2 as the default version for all shells.\n")
        .stderr("");

    assert_eq!(
        fs::canonicalize(&current).unwrap(),
        fs::canonicalize(candidate.join("0.0.2")).unwrap()
    );
    assert!(fs::symlink_metadata(&backup).is_err());
    assert!(!candidate.join("current-new").exists());
}
