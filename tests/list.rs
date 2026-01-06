#[cfg(test)]
use assert_cmd::Command;
use predicates::str::contains;
use serial_test::serial;
use std::{env, error};
use support::{TestCandidate, VirtualEnv};

mod support;

#[test]
#[serial]
fn should_successfully_list_installed_offline_and_mark_current() -> Result<(), Box<dyn error::Error>>
{
    let env_cfg = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        native_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1", "0.0.2", "0.0.3"],
            current_version: "0.0.2",
        }],
    };

    let sdkman_dir = support::virtual_env(env_cfg);
    let dir_string = sdkman_dir.path().to_str().unwrap();

    env::set_var("SDKMAN_DIR", dir_string);
    env::set_var("SDKMAN_AVAILABLE", "false");

    Command::new(assert_cmd::cargo::cargo_bin!("list"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .arg("scala")
        .assert()
        .success()
        .stdout(contains("Offline: only showing installed scala versions"))
        .stdout(contains(" > 0.0.2"))
        .stdout(contains(" * 0.0.1"))
        .stdout(contains(" * 0.0.3"))
        .code(0);

    Ok(())
}

#[test]
#[serial]
fn should_show_none_installed_offline_when_candidate_not_present(
) -> Result<(), Box<dyn error::Error>> {
    let env_cfg = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        native_version: "0.0.1".to_string(),
        candidates: vec![], // nothing here
    };

    let sdkman_dir = support::virtual_env(env_cfg);
    let dir_string = sdkman_dir.path().to_str().unwrap();

    env::set_var("SDKMAN_DIR", dir_string);
    env::set_var("SDKMAN_AVAILABLE", "false");

    Command::new(assert_cmd::cargo::cargo_bin!("list"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .arg("scala")
        .assert()
        .success()
        .stdout(contains("Offline: only showing installed scala versions"))
        .stdout(contains("None installed!"))
        .code(0);

    Ok(())
}

#[test]
#[serial]
fn should_reject_list_candidates_while_offline() -> Result<(), Box<dyn error::Error>> {
    let env_cfg = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        native_version: "0.0.1".to_string(),
        candidates: vec![], // nothing here
    };

    let sdkman_dir = support::virtual_env(env_cfg);
    let dir_string = sdkman_dir.path().to_str().unwrap();

    env::set_var("SDKMAN_DIR", dir_string);
    env::set_var("SDKMAN_AVAILABLE", "false");

    Command::new(assert_cmd::cargo::cargo_bin!("list"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .arg("scala")
        .assert()
        .success()
        .stdout(contains("Offline: only showing installed scala versions"))
        .stdout(contains("None installed!"))
        .code(0);

    Ok(())
}

// Online tests (mock the candidates API)

#[test]
#[serial]
fn should_successfully_list_candidates_online() -> Result<(), Box<dyn std::error::Error>> {
    use mockito::Server;

    let env_cfg = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        native_version: "0.0.1".to_string(),
        candidates: vec![],
    };
    let sdkman_dir = support::virtual_env(env_cfg);
    let dir_string = sdkman_dir.path().to_str().unwrap();

    let mut server = Server::new();
    let body = "candidates-list-output\nline2\n";
    let m = server
        .mock("GET", "/candidates/list")
        .with_status(200)
        .with_body(body)
        .create();

    env::set_var("SDKMAN_DIR", dir_string);
    env::set_var("SDKMAN_AVAILABLE", "true");
    env::set_var("SDKMAN_CANDIDATES_API", server.url());
    env::set_var("SDKMAN_PLATFORM", "linuxx64");

    Command::new(assert_cmd::cargo::cargo_bin!("list"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .assert()
        .success()
        .stdout(contains("candidates-list-output"))
        .code(0);

    m.assert();
    Ok(())
}

#[test]
#[serial]
fn should_successfully_list_versions_online_and_pass_current_and_installed_query_params(
) -> Result<(), Box<dyn std::error::Error>> {
    use mockito::{Matcher, Server};

    let env_cfg = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        native_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1", "0.0.2", "0.0.3"],
            current_version: "0.0.2",
        }],
    };
    let sdkman_dir = support::virtual_env(env_cfg);
    let dir_string = sdkman_dir.path().to_str().unwrap();

    let mut server = Server::new();
    let body = "versions-list-output\n";
    let m = server
        .mock("GET", "/candidates/scala/linuxx64/versions/list")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("current".into(), "0.0.2".into()),
            // it should be ascending: 0.0.1..0.0.3
            Matcher::UrlEncoded("installed".into(), "0.0.1,0.0.2,0.0.3".into()),
        ]))
        .with_status(200)
        .with_body(body)
        .create();

    env::set_var("SDKMAN_DIR", dir_string);
    env::set_var("SDKMAN_AVAILABLE", "true");
    env::set_var("SDKMAN_CANDIDATES_API", server.url());
    env::set_var("SDKMAN_PLATFORM", "linuxx64");

    Command::new(assert_cmd::cargo::cargo_bin!("list"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .arg("scala")
        .assert()
        .success()
        .stdout(contains("versions-list-output"))
        .code(0);

    m.assert();
    Ok(())
}

#[test]
#[serial]
fn should_fail_when_candidates_api_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    use mockito::Server;

    let env_cfg = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        native_version: "0.0.1".to_string(),
        candidates: vec![],
    };
    let sdkman_dir = support::virtual_env(env_cfg);
    let dir_string = sdkman_dir.path().to_str().unwrap();

    let mut server = Server::new();
    let m = server
        .mock("GET", "/candidates/list")
        .with_status(500)
        .with_body("boom")
        .create();

    env::set_var("SDKMAN_DIR", dir_string);
    env::set_var("SDKMAN_AVAILABLE", "true");
    env::set_var("SDKMAN_CANDIDATES_API", server.url());
    env::set_var("SDKMAN_PLATFORM", "linuxx64");

    Command::new(assert_cmd::cargo::cargo_bin!("list"))
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .assert()
        .failure()
        .code(1);

    m.assert();
    Ok(())
}
