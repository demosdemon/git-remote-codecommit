use std::io::IsTerminal;

use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    let filter = EnvFilter::builder()
        .with_default_directive(Level::ERROR.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_ansi(term_color())
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();
}

fn term_color() -> bool {
    match std::env::var_os("CARGO_TERM_COLOR") {
        Some(val) if val == "always" => true,
        Some(val) if val == "never" => false,
        _ => std::io::stderr().is_terminal(),
    }
}
