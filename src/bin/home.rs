use clap::Parser;

use sdkman_cli_native::helpers::{
    infer_sdkman_dir, known_candidates, validate_candidate, validate_version_path,
};

#[derive(Parser, Debug)]
#[command(
    bin_name = "sdk home",
    about = "sdk subcommand to output the path of a specific candidate version"
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

    validate_candidate(&known_candidates(&sdkman_dir), &candidate);

    let candidate_path = validate_version_path(&sdkman_dir, &candidate, &version);
    print!("{}", candidate_path.display());
}
