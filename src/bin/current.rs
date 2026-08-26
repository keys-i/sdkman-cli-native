use std::fs;
use std::path::Path;
use std::process;

use clap::Parser;
use colored::Colorize;

use sdkman_cli_native::constants::{CANDIDATES_DIR, CURRENT_DIR, CURRENT_VERSION_FILE};
use sdkman_cli_native::helpers::{
    infer_sdkman_dir, known_candidates, read_file_content, validate_candidate,
};

#[derive(Parser, Debug)]
#[command(
    bin_name = "sdk current",
    about = "sdk subcommand to display the current version in use for one or all candidates"
)]
struct Args {
    #[arg(required(false))]
    candidate: Option<String>,
}

fn main() {
    let args = Args::parse();
    let sdkman_dir = infer_sdkman_dir();
    let candidates = known_candidates(&sdkman_dir);

    if let Some(candidate) = args.candidate {
        validate_candidate(&candidates, &candidate);
        match get_current_version(&sdkman_dir, &candidate) {
            Some(version) => println!(
                "Current default {} version {}",
                candidate.bold(),
                version.bold()
            ),
            None => {
                eprintln!("No current version of {} configured.", candidate.bold());
                process::exit(1);
            }
        }
        return;
    }

    let current: Vec<_> = candidates
        .iter()
        .filter_map(|candidate| {
            get_current_version(&sdkman_dir, candidate).map(|version| (candidate, version))
        })
        .collect();

    if current.is_empty() {
        eprintln!("No candidates are in use.");
        return;
    }

    println!("{}", "Current default versions:".bold());
    for (candidate, version) in current {
        println!("{candidate} {version}");
    }
}

fn get_current_version(base_dir: &Path, candidate: &str) -> Option<String> {
    let candidate_dir = base_dir.join(CANDIDATES_DIR).join(candidate);
    if !candidate_dir.is_dir() {
        return None;
    }

    let current = candidate_dir.join(CURRENT_DIR);
    let metadata = fs::symlink_metadata(&current).ok()?;
    if metadata.file_type().is_symlink() {
        let candidate_dir = fs::canonicalize(candidate_dir).ok()?;
        let target = fs::canonicalize(current).ok()?;
        if target.parent()? != candidate_dir {
            return None;
        }
        return target.file_name()?.to_str().map(str::to_owned);
    }

    metadata
        .is_dir()
        .then(|| read_file_content(&current.join(CURRENT_VERSION_FILE)))
        .flatten()
}
