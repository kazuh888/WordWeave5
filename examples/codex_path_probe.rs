//! Read-only discovery check: no Codex launch, authentication or user-data access.
fn main() {
    let configured = std::env::args().nth(1).unwrap_or_else(|| "codex".into());
    match wordweave5::codex::resolve_executable(&configured) {
        Ok(path) => println!("{}", path.display()),
        Err(error) => { eprintln!("{error}"); std::process::exit(1); }
    }
}
