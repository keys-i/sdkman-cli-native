use clap::Parser;
use colored::Colorize;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::Path;
use symlink::{remove_symlink_dir, symlink_dir};

use sdkman_cli_native::constants::{CANDIDATES_DIR, CURRENT_DIR, CURRENT_VERSION_FILE, TMP_DIR};
use sdkman_cli_native::helpers::{
    infer_sdkman_dir, known_candidates, validate_candidate, validate_version_path,
};

#[derive(Parser, Debug)]
#[command(
    bin_name = "sdk default",
    about = "sdk subcommand to set the local default version of the candidate"
)]
struct Args {
    #[arg(required(true))]
    candidate: String,

    #[arg(required(true))]
    version: String,
}
fn main() {
    let args = Args::parse();
    let candidate = args.candidate;
    let version = args.version;
    let sdkman_dir = infer_sdkman_dir();
    let candidates = known_candidates(&sdkman_dir);
    validate_candidate(&candidates, &candidate);
    let version_path = validate_version_path(&sdkman_dir, &candidate, &version);
    let candidate_dir = sdkman_dir.join(CANDIDATES_DIR).join(&candidate);
    let current = candidate_dir.join(CURRENT_DIR);
    let next = candidate_dir.join("current-new");
    let previous = candidate_dir.join("current-old");
    let staging = sdkman_dir
        .join(TMP_DIR)
        .join(format!("current-{candidate}"));

    let copied = prepare_next(&version_path, &version, &next, &staging).and_then(|copied| {
        replace_current(&current, &next, &previous).inspect_err(|_| {
            let _ = remove_path(&next);
        })?;
        Ok(copied)
    });
    match copied {
        Ok(copied) => {
            println!(
                "setting {} {} as the {} version for all shells.",
                candidate.bold(),
                version.bold(),
                "default".italic()
            );
            if copied {
                println!(
                    "{}",
                    "cannot create current symlink, fall back to copy!".bold()
                );
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn prepare_next(version: &Path, name: &str, next: &Path, staging: &Path) -> Result<bool, String> {
    if fs::symlink_metadata(next).is_ok() {
        return Err(format!(
            "cannot prepare replacement: {} already exists",
            next.display()
        ));
    }
    if symlink_dir(version, next).is_ok() {
        return Ok(false);
    }

    if fs::symlink_metadata(staging).is_ok() {
        remove_path(staging)?;
    }
    fs::create_dir_all(staging)
        .map_err(|error| format!("cannot create temporary current directory: {error}"))?;
    let options = CopyOptions::new();
    copy_items(&[version], staging, &options)
        .map_err(|error| format!("cannot copy to temporary current directory: {error}"))?;
    let copied = staging.join(name);
    fs::write(copied.join(CURRENT_VERSION_FILE), name)
        .map_err(|error| format!("cannot write current version marker: {error}"))?;
    fs::rename(&copied, next).map_err(|error| format!("cannot prepare replacement: {error}"))?;
    fs::remove_dir(staging)
        .map_err(|error| format!("cannot remove temporary current directory: {error}"))?;
    Ok(true)
}

fn replace_current(current: &Path, next: &Path, previous: &Path) -> Result<(), String> {
    if fs::symlink_metadata(previous).is_ok() {
        if fs::symlink_metadata(current).is_ok() {
            remove_path(previous)?;
        } else {
            fs::rename(previous, current)
                .map_err(|error| format!("cannot restore current: {error}"))?;
        }
    }
    if fs::symlink_metadata(current).is_err() {
        return fs::rename(next, current)
            .map_err(|error| format!("cannot install current: {error}"));
    }

    fs::rename(current, previous).map_err(|error| format!("cannot preserve current: {error}"))?;
    if let Err(error) = fs::rename(next, current) {
        return match fs::rename(previous, current) {
            Ok(()) => Err(format!("cannot install current: {error}")),
            Err(rollback) => Err(format!(
                "cannot install current: {error}; rollback failed: {rollback}"
            )),
        };
    }
    if let Err(error) = remove_path(previous) {
        eprintln!("warning: {error}");
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        remove_symlink_dir(path)
    } else if metadata.is_file() {
        fs::remove_file(path)
    } else {
        fs::remove_dir_all(path)
    }
    .map_err(|error| format!("cannot remove {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::replace_current;

    #[test]
    fn restores_current_when_installing_the_replacement_fails() {
        let directory = tempfile::tempdir().unwrap();
        let current = directory.path().join("current");
        let previous = directory.path().join("current-old");
        let next = directory.path().join("missing-current-new");
        fs::create_dir(&current).unwrap();
        fs::write(current.join("sentinel"), "old").unwrap();

        assert!(replace_current(&current, &next, &previous).is_err());
        assert_eq!(fs::read_to_string(current.join("sentinel")).unwrap(), "old");
        assert!(fs::symlink_metadata(previous).is_err());
    }

    #[test]
    fn restores_a_stale_previous_before_replacing_current() {
        let directory = tempfile::tempdir().unwrap();
        let current = directory.path().join("current");
        let previous = directory.path().join("current-old");
        let next = directory.path().join("missing-current-new");
        fs::create_dir(&previous).unwrap();
        fs::write(previous.join("sentinel"), "old").unwrap();

        assert!(replace_current(&current, &next, &previous).is_err());
        assert_eq!(fs::read_to_string(current.join("sentinel")).unwrap(), "old");
        assert!(fs::symlink_metadata(previous).is_err());
    }
}
