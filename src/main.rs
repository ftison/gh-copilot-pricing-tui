use std::process;

use tracing::{debug, error, info};

use gh_copilot_pricing_tui::app::run_tui;
use gh_copilot_pricing_tui::data::fetch::fetch_documentation;
use gh_copilot_pricing_tui::data::parse::parse_documentation;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Fetching GitHub Copilot pricing documentation");
    let markdown = match fetch_documentation().await {
        Ok(md) => {
            debug!(bytes = md.len(), "Documentation fetched successfully");
            md
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch documentation");
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    let pricing_data = match parse_documentation(&markdown) {
        Ok(data) => {
            info!(entries = data.len(), "Pricing data parsed successfully");
            data
        }
        Err(e) => {
            error!(error = %e, "Failed to parse documentation");
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = run_tui(pricing_data.entries).await {
        error!(error = %e, "TUI failed");
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
