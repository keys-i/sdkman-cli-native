use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use sdkman_cli_native::constants::CURRENT_VERSION_FILE;
use support::{TestCandidate, VirtualEnv};

mod support;

fn command(sdkman_dir: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("current"));
    command.env("SDKMAN_DIR", sdkman_dir);
    command
}

fn environment(candidates: Vec<TestCandidate>) -> tempfile::TempDir {
    support::virtual_env(VirtualEnv {
        cli_version: "5.0.0".to_owned(),
        candidates,
    })
}

#[test]
fn shows_the_specific_current_version() {
    let sdkman_dir = environment(vec![TestCandidate {
        name: "java",
        versions: vec!["11.0.15-tem", "17.0.3-tem"],
        current_version: "11.0.15-tem",
    }]);

    command(sdkman_dir.path())
        .arg("java")
        .assert()
        .success()
        .stdout("Current default java version 11.0.15-tem\n")
        .stderr("");
}

#[test]
fn shows_all_current_versions() {
    let sdkman_dir = environment(vec![
        TestCandidate {
            name: "java",
            versions: vec!["11.0.15-tem"],
            current_version: "11.0.15-tem",
        },
        TestCandidate {
            name: "kotlin",
            versions: vec!["1.7.22"],
            current_version: "1.7.22",
        },
    ]);

    command(sdkman_dir.path())
        .assert()
        .success()
        .stdout("Current default versions:\njava 11.0.15-tem\nkotlin 1.7.22\n");
}

#[test]
fn rejects_unknown_candidate() {
    let sdkman_dir = environment(vec![TestCandidate {
        name: "java",
        versions: vec!["17"],
        current_version: "17",
    }]);

    command(sdkman_dir.path())
        .arg("invalid")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr("invalid is not a valid candidate.\n");
}

#[test]
fn reports_no_current_version_for_missing_or_invalid_current() {
    for state in ["missing", "dangling", "nested", "outside"] {
        let sdkman_dir = environment(vec![TestCandidate {
            name: "kotlin",
            versions: vec!["1.7.22"],
            current_version: "1.7.22",
        }]);
        let current = sdkman_dir.path().join("candidates/kotlin/current");
        symlink::remove_symlink_dir(&current).unwrap();
        let outside = tempfile::tempdir().unwrap();
        match state {
            "missing" => {}
            "dangling" => symlink::symlink_dir("missing", &current).unwrap(),
            "nested" => {
                let nested = sdkman_dir.path().join("candidates/kotlin/nested/1.7.22");
                fs::create_dir_all(&nested).unwrap();
                symlink::symlink_dir("nested/1.7.22", &current).unwrap();
            }
            "outside" => {
                let target = outside.path().join("1.7.22");
                fs::create_dir(&target).unwrap();
                symlink::symlink_dir(target, &current).unwrap();
            }
            _ => unreachable!(),
        }

        command(sdkman_dir.path())
            .arg("kotlin")
            .assert()
            .failure()
            .code(1)
            .stdout("")
            .stderr("No current version of kotlin configured.\n");
    }
}

#[test]
fn reports_no_candidates_in_use_for_empty_or_inactive_inventory() {
    for candidates in ["", "java,kotlin"] {
        let sdkman_dir = support::prepare_sdkman_dir();
        support::write_file(
            sdkman_dir.path(),
            Path::new("var"),
            "candidates",
            candidates.to_owned(),
        );
        for candidate in candidates
            .split(',')
            .filter(|candidate| !candidate.is_empty())
        {
            fs::create_dir_all(sdkman_dir.path().join("candidates").join(candidate)).unwrap();
        }

        command(sdkman_dir.path())
            .assert()
            .success()
            .stdout("")
            .stderr("No candidates are in use.\n");
    }
}

#[test]
fn reads_the_copied_current_marker() {
    let sdkman_dir = environment(vec![TestCandidate {
        name: "scala",
        versions: vec!["3.3.0"],
        current_version: "3.3.0",
    }]);
    let current = sdkman_dir.path().join("candidates/scala/current");
    symlink::remove_symlink_dir(&current).unwrap();
    fs::create_dir(&current).unwrap();
    fs::write(current.join(CURRENT_VERSION_FILE), "3.3.0\n").unwrap();

    command(sdkman_dir.path())
        .arg("scala")
        .assert()
        .success()
        .stdout("Current default scala version 3.3.0\n");
}
