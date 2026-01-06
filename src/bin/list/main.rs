use colored::Colorize;
use reqwest::blocking::Client;
use sdkman_cli_native::{constants::CANDIDATES_DIR, helpers::infer_sdkman_dir};
use std::{
    env,
    error::Error,
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::Path,
    process::exit,
    time::Duration,
};
use urlencoding::encode;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e.to_string().red());
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let sdkman_dir = infer_sdkman_dir();
    let args: Vec<String> = env::args().collect();
    let candidate = args.get(1).map(String::as_str);

    let available = env::var("SDKMAN_AVAILABLE").ok().as_deref() != Some("false");

    match candidate {
        None => list_candidates(available),
        Some(c) => list_versions(&sdkman_dir, available, c),
    }
}

/// Replacement for `__sdkman_list_candidates`
fn list_candidates(available: bool) -> Result<(), Box<dyn Error>> {
    if !available {
        println!("{}", "This command is not available while offline.".red());
        return Ok(());
    }

    let api = env::var("SDKMAN_CANDIDATES_API")?;
    let url = format!("{}/candidates/list", api.trim_end_matches('/'));
    print_paged(&secure_get(&url)?)?;
    Ok(())
}

/// Replacement for `__sdkman_list_versions`
fn list_versions(
    sdkman_dir: &Path,
    available: bool,
    candidate: &str,
) -> Result<(), Box<dyn Error>> {
    let candidates_dir = sdkman_dir.join(CANDIDATES_DIR);
    let versions_csv = build_version_csv(&candidates_dir, candidate);
    let current = determine_current_version(&candidates_dir, candidate).unwrap_or_default();

    if !available {
        offline_list(candidate, &versions_csv, &current)?;
        return Ok(());
    }

    let api = env::var("SDKMAN_CANDIDATES_API")?;
    let platform = env::var("SDKMAN_PLATFORM")?;

    let url = format!(
        "{}/candidates/{}/{}/versions/list?current={}&installed={}",
        api.trim_end_matches('/'),
        candidate,
        platform,
        encode(&current),
        encode(&versions_csv),
    );

    print_paged(&secure_get(&url)?)?;
    Ok(())
}

/// Replacement for `__sdkman_offline_list`
fn offline_list(candidate: &str, versions_csv: &str, current: &str) -> io::Result<()> {
    println!("{}", "-".repeat(80));
    println!(
        "{}",
        format!("Offline: only showing installed {candidate} versions").yellow()
    );
    println!("{}", "-".repeat(80));

    let versions: Vec<&str> = versions_csv
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if versions.is_empty() {
        println!("{}", "   None installed!".yellow());
    } else {
        for v in versions.iter().rev() {
            if *v == current {
                println!(" > {}", v);
            } else {
                println!(" * {}", v);
            }
        }
    }

    println!("{}", "-".repeat(80));
    println!("* - installed");
    println!("> - currently in use");
    println!("{}", "-".repeat(80));
    Ok(())
}

/// Replacement for `__sdkman_build_version_csv`
/// - scans `${SDKMAN_CANDIDATES_DIR}/${candidate}`
/// - includes dirs OR symlinks
/// - excludes "current"
/// - sorts ascending
/// - joins with commas
fn build_version_csv(candidate_dir: &Path, candidate: &str) -> String {
    let base = candidate_dir.join(candidate);
    if !base.is_dir() {
        return String::new();
    }

    let mut versions: Vec<String> = Vec::new();
    let rd = match fs::read_dir(&base) {
        Ok(rd) => rd,
        Err(_) => return String::new(),
    };

    for entry in rd.flatten() {
        let name = entry.file_name();
        if name == OsStr::new("current") {
            continue;
        }

        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if !(ft.is_dir() || ft.is_symlink()) {
            continue;
        }

        if let Some(s) = name.to_str() {
            versions.push(s.to_string());
        }
    }
    versions.sort();
    versions.join(",")
}

/// Replacement of `__sdkman_determine_current_version`
fn determine_current_version(candidate_dir: &Path, candidate: &str) -> Option<String> {
    let current_path = candidate_dir.join(candidate).join("current");
    let target = fs::read_link(&current_path).ok()?;
    target.file_name()?.to_str().map(|s| s.to_string())
}

fn print_paged(s: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(s.as_bytes())?;
    if !s.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
}

/// Replacement for `__sdkman_secure_curl`
fn secure_get(url: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let res = client.get(url).send()?;
    let status = res.status();
    let text = res.text()?;

    if !status.is_success() {
        return Err(format!("Request failed ({status})").into());
    }
    Ok(text)
}
