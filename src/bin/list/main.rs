use colored::Colorize;
use reqwest::blocking::Client;
use std::{
    env,
    error::Error,
    io::{self, Write},
    process::exit,
    time::Duration,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e.to_string().red());
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    Ok(())
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
