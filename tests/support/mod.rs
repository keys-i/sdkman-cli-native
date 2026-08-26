use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use symlink::symlink_dir;

use tempfile::{Builder, TempDir};

pub struct TestCandidate {
    pub name: &'static str,
    pub versions: Vec<&'static str>,
    pub current_version: &'static str,
}

#[derive(Default)]
pub struct VirtualEnv {
    pub cli_version: String,
    pub candidates: Vec<TestCandidate>,
}

pub fn virtual_env(virtual_env: VirtualEnv) -> TempDir {
    let sdkman_dir = prepare_sdkman_dir();
    let var_path = Path::new("var");

    // script version file
    write_file(
        sdkman_dir.path(),
        var_path,
        "version",
        virtual_env.cli_version,
    );

    // Write candidates to the candidates file
    let candidates_str = virtual_env
        .candidates
        .iter()
        .map(|c| c.name)
        .collect::<Vec<&str>>()
        .join(",");

    write_file(
        sdkman_dir.path(),
        Path::new("var"),
        "candidates",
        candidates_str,
    );

    // Process each candidate
    for candidate in &virtual_env.candidates {
        for version in &candidate.versions {
            let location = format!("candidates/{}/{}/bin/", candidate.name, version);
            let content = format!(
                "\
#!/bin/bash
echo Running {} {}
",
                candidate.name, version
            );
            write_file(
                sdkman_dir.path(),
                Path::new(&location),
                candidate.name,
                content,
            );
        }

        let version_location = PathBuf::from(candidate.current_version);
        let current_link_location = PathBuf::from(format!("candidates/{}/current", candidate.name));
        let absolute_current_link = sdkman_dir.path().join(current_link_location.as_path());
        symlink_dir(version_location, absolute_current_link)
            .expect("cannot create current symlink");
    }

    sdkman_dir
}

pub fn prepare_sdkman_dir() -> TempDir {
    Builder::new()
        .prefix(".sdkman-")
        .tempdir()
        .expect("could not prepare SDKMAN_DIR")
}

pub fn write_file(
    temp_dir: &Path,
    relative_path: &Path,
    file_name: &str,
    content: String,
) -> PathBuf {
    let absolute_path = temp_dir.join(relative_path);
    create_dir_all(&absolute_path).expect("could not create nested dirs");

    let file_path = absolute_path.join(file_name);
    write(&file_path, content).expect("could not write to file");

    file_path
}
