#[cfg(test)]
use assert_cmd::Command;
use predicates::str::contains;
use support::{TestCandidate, VirtualEnv};

mod support;

#[test]
fn should_successfully_display_current_candidate_home() -> Result<(), Box<dyn std::error::Error>> {
    let env = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1"],
            current_version: "0.0.1",
        }],
    };

    let sdkman_dir = support::virtual_env(env);
    let expected_output = sdkman_dir.path().join("candidates/scala/0.0.1");
    Command::new(assert_cmd::cargo::cargo_bin!("home"))
        .env("SDKMAN_DIR", sdkman_dir.path())
        .arg("scala")
        .arg("0.0.1")
        .assert()
        .success()
        .stdout(expected_output.display().to_string())
        .code(0);

    Ok(())
}

#[test]
fn should_fail_if_candidate_home_is_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let env = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1"],
            current_version: "0.0.1",
        }],
    };

    let sdkman_dir = support::virtual_env(env);

    let expected_output = format!("{} {} is not installed on your system", "scala", "0.0.2");
    Command::new(assert_cmd::cargo::cargo_bin!("home"))
        .env("SDKMAN_DIR", sdkman_dir.path())
        .arg("scala")
        .arg("0.0.2")
        .assert()
        .failure()
        .stderr(contains(expected_output))
        .code(1);
    Ok(())
}

#[test]
fn should_reject_unsafe_version_paths() -> Result<(), Box<dyn std::error::Error>> {
    let env = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1"],
            current_version: "0.0.1",
        }],
    };
    let sdkman_dir = support::virtual_env(env);

    for version in ["..", "/tmp", "0.0.1/other"] {
        Command::new(assert_cmd::cargo::cargo_bin!("home"))
            .env("SDKMAN_DIR", sdkman_dir.path())
            .arg("scala")
            .arg(version)
            .assert()
            .failure()
            .stderr(contains(format!(
                "scala {version} is not installed on your system"
            )))
            .code(1);
    }

    assert!(sdkman_dir.path().join("candidates/scala/0.0.1").is_dir());
    Ok(())
}
