use crate::support::TestCandidate;
use sdkman_cli_native::helpers::known_candidates;
use support::{prepare_sdkman_dir, VirtualEnv};

mod support;

#[test]
fn should_read_known_candidates() -> Result<(), Box<dyn std::error::Error>> {
    let env = VirtualEnv {
        cli_version: "0.0.1".to_string(),
        candidates: vec![TestCandidate {
            name: "scala",
            versions: vec!["0.0.1"],
            current_version: "0.0.1",
        }],
    };

    let sdkman_dir = support::virtual_env(env);
    let candidates = known_candidates(sdkman_dir.path());
    let expected_candidate = vec!["scala"];

    assert_eq!(candidates, expected_candidate);

    Ok(())
}

#[test]
fn should_parse_candidates_from_config() {
    let sdkman_dir = prepare_sdkman_dir();
    for (content, expected) in [
        (" scala, ,java ,, kotlin ", vec!["scala", "java", "kotlin"]),
        ("", vec![]),
    ] {
        support::write_file(
            sdkman_dir.path(),
            std::path::Path::new("var"),
            "candidates",
            content.to_string(),
        );
        assert_eq!(known_candidates(sdkman_dir.path()), expected);
    }
}

#[test]
fn should_fail_if_candidate_file_is_missing() {
    let sdkman_dir = prepare_sdkman_dir();
    assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("current"))
        .env("SDKMAN_DIR", sdkman_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cannot read SDKMAN candidates file",
        ));
}
