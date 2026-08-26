#[cfg(test)]
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn should_render_base_help() -> Result<(), Box<dyn std::error::Error>> {
    let header = "\nNAME\n    sdk - The command line interface (CLI) for SDKMAN!";
    Command::new(assert_cmd::cargo::cargo_bin!("help"))
        .assert()
        .success()
        .stdout(predicate::str::starts_with(header))
        .code(0);
    Ok(())
}

#[test]
fn should_render_help_for_all_subcommands() -> Result<(), Box<dyn std::error::Error>> {
    let commands = [
        ("config", "config"),
        ("current", "current"),
        ("c", "current"),
        ("default", "default"),
        ("d", "default"),
        ("env", "env"),
        ("e", "env"),
        ("flush", "flush"),
        ("home", "home"),
        ("h", "home"),
        ("install", "install"),
        ("i", "install"),
        ("list", "list"),
        ("ls", "list"),
        ("selfupdate", "selfupdate"),
        ("uninstall", "uninstall"),
        ("rm", "uninstall"),
        ("update", "update"),
        ("upgrade", "upgrade"),
        ("ug", "upgrade"),
        ("use", "use"),
        ("u", "use"),
        ("version", "version"),
        ("v", "version"),
    ];

    for (arg, command) in commands {
        let header = format!("\nNAME\n    sdk {command} - ");
        Command::new(assert_cmd::cargo::cargo_bin!("help"))
            .arg(arg)
            .assert()
            .success()
            .stdout(predicate::str::starts_with(&header))
            .code(0);
    }
    Ok(())
}
