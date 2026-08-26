use std::{path::Path, process::Command};

use assert_cmd::prelude::*;
use predicates::prelude::*;
use support::VirtualEnv;

mod support;

#[test]
fn should_successfully_render_version() -> Result<(), Box<dyn std::error::Error>> {
    let cli_version = "5.0.0";
    let native_version = env!("CARGO_PKG_VERSION");
    let sdkman_dir = support::virtual_env(VirtualEnv {
        cli_version: cli_version.into(),
        ..Default::default()
    });
    let expected = format!(
        "\nSDKMAN!\nscript: {cli_version}\nnative: {native_version} ({} {})\n\n",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    Command::new(assert_cmd::cargo::cargo_bin!("version"))
        .env("SDKMAN_DIR", sdkman_dir.path())
        .assert()
        .success()
        .stdout(expected)
        .code(0);

    Ok(())
}

#[test]
fn should_report_missing_or_empty_version_metadata() {
    for contents in [None, Some("")] {
        let sdkman_dir = support::prepare_sdkman_dir();
        if let Some(contents) = contents {
            support::write_file(
                sdkman_dir.path(),
                Path::new("var"),
                "version",
                contents.into(),
            );
        }

        Command::new(assert_cmd::cargo::cargo_bin!("version"))
            .env("SDKMAN_DIR", sdkman_dir.path())
            .assert()
            .failure()
            .code(1)
            .stderr(predicate::str::contains(
                "Unable to read SDKMAN! version file:",
            ))
            .stderr(predicate::str::contains("panicked").not());
    }
}

#[test]
fn should_reject_unexpected_arguments() {
    Command::new(assert_cmd::cargo::cargo_bin!("version"))
        .arg("unexpected")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn should_render_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("version"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "sdk subcommand to display the installed SDKMAN version",
        ))
        .stdout(predicate::str::contains("Usage: sdk version"));
}
