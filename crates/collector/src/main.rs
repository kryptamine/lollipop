use std::path::PathBuf;

use clap::Parser;
use lollipop_collector::{collect, Config, SeedTier, SystemClock};

/// Collects ranked solo matches into a database, until a failure it cannot retry.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The owner's tier: its seed players and its neighbours' are walked.
    #[arg(long, value_enum)]
    tier: SeedTier,
    #[arg(long, default_value = "/var/lib/lollipop/collector.sqlite3")]
    database: PathBuf,
    #[arg(long, default_value = "/etc/lollipop/riot-key")]
    key_file: PathBuf,
    #[arg(long, default_value = "https://euw1.api.riotgames.com")]
    platform_url: String,
    #[arg(long, default_value = "https://europe.api.riotgames.com")]
    regional_url: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    let key = std::fs::read_to_string(&args.key_file)
        .map_err(|error| format!("cannot read {}: {error}", args.key_file.display()))?;
    let config = Config {
        tier: args.tier,
        key: key.trim().to_string(),
        database: args.database,
        platform_url: args.platform_url,
        regional_url: args.regional_url,
    };
    Err(collect(&config, &SystemClock).await.to_string())
}
