use std::fs;
use std::fs::remove_dir_all;
use std::process;

use clap::Parser;
use colored::Colorize;
use symlink::remove_symlink_dir;

use sdkman_cli_native::constants::{CANDIDATES_DIR, CURRENT_DIR, CURRENT_VERSION_FILE};
use sdkman_cli_native::helpers::{
    infer_sdkman_dir, known_candidates, validate_candidate, validate_version_path,
};

#[derive(Parser, Debug)]
#[command(
    bin_name = "sdk uninstall",
    about = "sdk subcommand to remove a specific candidate version"
)]
struct Args {
    #[arg(short = 'f', long = "force")]
    force: bool,

    #[arg(required(true))]
    candidate: String,

    #[arg(required(true))]
    version: String,
}

fn main() {
    let args = Args::parse();
    let candidate = args.candidate;
    let version = args.version;
    let force = args.force;
    let sdkman_dir = infer_sdkman_dir();

    validate_candidate(&known_candidates(&sdkman_dir), &candidate);

    let candidate_path = sdkman_dir.join(CANDIDATES_DIR).join(&candidate);
    let version_path = validate_version_path(&sdkman_dir, &candidate, &version);
    let current_link_path = candidate_path.join(CURRENT_DIR);
    let current_version = fs::canonicalize(&current_link_path)
        .ok()
        .zip(fs::canonicalize(&version_path).ok())
        .is_some_and(|(current, version)| current == version)
        || fs::read_to_string(current_link_path.join(CURRENT_VERSION_FILE))
            .is_ok_and(|current| current.trim() == version);

    if current_version {
        if !force {
            eprintln!(
                "\n{} {} is the {} version and should not be removed.",
                candidate.bold(),
                version.bold(),
                "current".italic(),
            );
            println!(
                "\n\nOverride with {}, but leaves the candidate unusable!",
                "--force".italic()
            );
            process::exit(1);
        }

        remove_symlink_dir(&current_link_path).unwrap_or_else(|_| {
            remove_dir_all(&current_link_path)
                .unwrap_or_else(|_| panic!("cannot remove current directory for {}.", candidate))
        });
    }

    remove_dir_all(version_path)
        .map(|_| {
            println!("removed {} {}.", candidate.bold(), version.bold());
        })
        .expect("panic! could not delete directory.");
}
