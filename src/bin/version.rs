use std::process;

use clap::Parser;
use colored::Colorize;

use sdkman_cli_native::{
    constants::VAR_DIR,
    helpers::{infer_sdkman_dir, read_file_content},
};

const CLI_VERSION_FILE: &str = "version";
const NATIVE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    bin_name = "sdk version",
    about = "sdk subcommand to display the installed SDKMAN version"
)]
struct Args;

fn main() {
    Args::parse();

    let sdkman_dir = infer_sdkman_dir();
    let cli_version_file = sdkman_dir.join(VAR_DIR).join(CLI_VERSION_FILE);
    let cli_version = read_file_content(&cli_version_file).unwrap_or_else(|| {
        eprintln!(
            "Unable to read SDKMAN! version file: {}",
            cli_version_file.display()
        );
        process::exit(1);
    });

    println!(
        "\n{}\nscript: {}\nnative: {} ({} {})\n",
        "SDKMAN!".bold().yellow(),
        cli_version,
        NATIVE_VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH
    );
}
