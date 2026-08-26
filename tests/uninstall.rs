use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::str::contains;
use sdkman_cli_native::constants::CURRENT_VERSION_FILE;
use support::{TestCandidate, VirtualEnv};
use symlink::{remove_symlink_dir, symlink_dir};
use tempfile::TempDir;

mod support;

const CANDIDATE: &str = "scala";
const CURRENT: &str = "current";
const UNUSED: &str = "0.0.1";
const ACTIVE: &str = "0.0.2";

fn sdkman_env() -> TempDir {
    support::virtual_env(VirtualEnv {
        cli_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: CANDIDATE,
            versions: vec![UNUSED, ACTIVE],
            current_version: ACTIVE,
        }],
    })
}

fn candidate_dir(sdkman_dir: &Path) -> std::path::PathBuf {
    sdkman_dir.join("candidates").join(CANDIDATE)
}

fn uninstall(sdkman_dir: &Path, version: &str, force: bool) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("uninstall"));
    command
        .env("SDKMAN_DIR", sdkman_dir)
        .arg(CANDIDATE)
        .arg(version);
    if force {
        command.arg("--force");
    }
    command
}

#[test]
fn removes_unused_version_and_preserves_current() {
    let sdkman_dir = sdkman_env();
    let candidate_dir = candidate_dir(sdkman_dir.path());

    uninstall(sdkman_dir.path(), UNUSED, false)
        .assert()
        .success()
        .stdout(contains("removed scala 0.0.1"));

    assert!(!candidate_dir.join(UNUSED).exists());
    assert!(candidate_dir.join(ACTIVE).is_dir());
    assert!(candidate_dir.join(CURRENT).exists());
}

#[test]
fn protects_and_forces_current_symlink_for_all_target_forms() {
    let forms = ["absolute", "version", "./version", "nested/../version"];

    for form in forms {
        let sdkman_dir = sdkman_env();
        let candidate_dir = candidate_dir(sdkman_dir.path());
        let current = candidate_dir.join(CURRENT);
        remove_symlink_dir(&current).unwrap();
        if form == "nested/../version" {
            fs::create_dir(candidate_dir.join("nested")).unwrap();
        }
        let target = match form {
            "absolute" => candidate_dir.join(ACTIVE),
            "version" => ACTIVE.into(),
            "./version" => format!("./{ACTIVE}").into(),
            "nested/../version" => format!("nested/../{ACTIVE}").into(),
            _ => unreachable!(),
        };
        symlink_dir(target, &current).unwrap();

        uninstall(sdkman_dir.path(), ACTIVE, false)
            .assert()
            .failure()
            .stderr(contains("scala 0.0.2 is the current version"));
        assert!(candidate_dir.join(ACTIVE).is_dir(), "{form}");
        assert!(current.exists(), "{form}");

        uninstall(sdkman_dir.path(), ACTIVE, true)
            .assert()
            .success()
            .stdout(contains("removed scala 0.0.2"));
        assert!(!candidate_dir.join(ACTIVE).exists(), "{form}");
        assert!(!current.exists(), "{form}");
    }
}

#[test]
fn protects_and_forces_current_copy_marked_with_its_version() {
    let sdkman_dir = sdkman_env();
    let candidate_dir = candidate_dir(sdkman_dir.path());
    let current = candidate_dir.join(CURRENT);
    remove_symlink_dir(&current).unwrap();
    fs::create_dir(&current).unwrap();
    fs::write(current.join(CURRENT_VERSION_FILE), ACTIVE).unwrap();

    uninstall(sdkman_dir.path(), ACTIVE, false)
        .assert()
        .failure()
        .stderr(contains("scala 0.0.2 is the current version"));
    assert!(candidate_dir.join(ACTIVE).is_dir());
    assert!(current.is_dir());

    uninstall(sdkman_dir.path(), ACTIVE, true)
        .assert()
        .success()
        .stdout(contains("removed scala 0.0.2"));
    assert!(!candidate_dir.join(ACTIVE).exists());
    assert!(!current.exists());
}

#[test]
fn rejects_invalid_candidate_and_missing_version_without_deleting_anything() {
    let sdkman_dir = sdkman_env();
    let candidate_dir = candidate_dir(sdkman_dir.path());

    uninstall(sdkman_dir.path(), "0.0.3", false)
        .assert()
        .failure()
        .stderr(contains("scala 0.0.3 is not installed on your system"));
    assert!(candidate_dir.join(UNUSED).is_dir());
    assert!(candidate_dir.join(ACTIVE).is_dir());
    assert!(candidate_dir.join(CURRENT).exists());

    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("uninstall"));
    command
        .env("SDKMAN_DIR", sdkman_dir.path())
        .arg("zcala")
        .arg(UNUSED)
        .assert()
        .failure()
        .stderr(contains("zcala is not a valid candidate"));
    assert!(candidate_dir.join(UNUSED).is_dir());
    assert!(candidate_dir.join(ACTIVE).is_dir());
    assert!(candidate_dir.join(CURRENT).exists());
}

#[test]
fn rejects_destructive_version_paths() {
    for version in [
        ".",
        "..",
        "../victim",
        "0.0.1/../victim",
        "0.0.1/victim",
        "/tmp/victim",
    ] {
        let sdkman_dir = sdkman_env();
        let candidate_dir = candidate_dir(sdkman_dir.path());
        let victim = tempfile::tempdir().unwrap();
        let sentinel = victim.path().join("sentinel");
        fs::write(&sentinel, "keep").unwrap();
        let external = victim.path().to_str().unwrap();
        let version = if version == "/tmp/victim" {
            external
        } else {
            version
        };

        uninstall(sdkman_dir.path(), version, true)
            .assert()
            .failure();
        assert!(sentinel.exists(), "{version}");
        assert!(candidate_dir.join(UNUSED).is_dir(), "{version}");
        assert!(candidate_dir.join(ACTIVE).is_dir(), "{version}");
        assert!(candidate_dir.join(CURRENT).exists(), "{version}");
    }
}

#[test]
fn rejects_symlinked_paths_outside_sdkman_dir() {
    for escaped_path in ["candidates", "candidate", "version"] {
        let sdkman_dir = sdkman_env();
        let candidate_dir = candidate_dir(sdkman_dir.path());
        let victim = tempfile::tempdir().unwrap();
        let external_version = match escaped_path {
            "candidates" => victim.path().join(CANDIDATE).join(UNUSED),
            _ => victim.path().join(UNUSED),
        };
        fs::create_dir_all(&external_version).unwrap();
        let sentinel = external_version.join("sentinel");
        fs::write(&sentinel, "keep").unwrap();

        match escaped_path {
            "candidates" => {
                remove_symlink_dir(candidate_dir.join(CURRENT)).unwrap();
                fs::remove_dir_all(sdkman_dir.path().join("candidates")).unwrap();
                symlink_dir(victim.path(), sdkman_dir.path().join("candidates")).unwrap();
            }
            "candidate" => {
                remove_symlink_dir(candidate_dir.join(CURRENT)).unwrap();
                fs::remove_dir_all(&candidate_dir).unwrap();
                symlink_dir(victim.path(), &candidate_dir).unwrap();
            }
            "version" => {
                fs::remove_dir_all(candidate_dir.join(UNUSED)).unwrap();
                symlink_dir(&external_version, candidate_dir.join(UNUSED)).unwrap();
            }
            _ => unreachable!(),
        }

        uninstall(sdkman_dir.path(), UNUSED, true)
            .assert()
            .failure()
            .code(1)
            .stderr(contains("scala 0.0.1 is not installed on your system"));
        assert!(sentinel.exists(), "{escaped_path}");
        assert!(external_version.is_dir(), "{escaped_path}");
    }
}

#[test]
fn rejects_same_root_symlink_aliases_without_deleting_targets() {
    for escaped_path in ["candidate", "version"] {
        let sdkman_dir = sdkman_env();
        let candidate_dir = candidate_dir(sdkman_dir.path());
        let target = match escaped_path {
            "candidate" => {
                remove_symlink_dir(candidate_dir.join(CURRENT)).unwrap();
                fs::remove_dir_all(&candidate_dir).unwrap();
                let target = sdkman_dir
                    .path()
                    .join("candidates")
                    .join("java")
                    .join(UNUSED);
                fs::create_dir_all(&target).unwrap();
                symlink_dir("java", &candidate_dir).unwrap();
                target
            }
            "version" => {
                fs::remove_dir_all(candidate_dir.join(UNUSED)).unwrap();
                symlink_dir(ACTIVE, candidate_dir.join(UNUSED)).unwrap();
                candidate_dir.join(ACTIVE)
            }
            _ => unreachable!(),
        };
        let sentinel = target.join("sentinel");
        fs::write(&sentinel, "keep").unwrap();

        uninstall(sdkman_dir.path(), UNUSED, true)
            .assert()
            .failure()
            .code(1)
            .stderr(contains("scala 0.0.1 is not installed on your system"));
        assert!(sentinel.exists(), "{escaped_path}");
        assert!(target.is_dir(), "{escaped_path}");
    }
}
