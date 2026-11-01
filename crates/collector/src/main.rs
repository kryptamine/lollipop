use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use lollipop_collector::{
    build_table, check_schedule, collect, publish, rollback, Config, PublishConfig, RollbackConfig,
    SeedTier, SystemClock, TableConfig,
};
use lollipop_core::game_inputs::Patch;

const REPO: &str = "kryptamine/lollipop";
const WORK_DIR: &str = "/var/lib/lollipop/publish";

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Collects ranked solo matches into a database, until a failure it cannot retry.
    Collect(CollectArgs),
    /// Aggregates the stored matches into a Benchmark table file and prints the drift report
    /// and the schedule check.
    BuildTable(BuildTableArgs),
    /// Builds a table and publishes it behind the publish gate. A failure, or a kill before its
    /// scheduled spawn, opens an issue.
    Publish(PublishArgs),
    /// Points the manifest at a kept table.
    Rollback(RollbackArgs),
}

#[derive(Args)]
struct CollectArgs {
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

#[derive(Args)]
struct BuildTableArgs {
    #[arg(long, default_value = "/var/lib/lollipop/collector.sqlite3")]
    database: PathBuf,
    /// The repository's `crates/collector/reworks.json`.
    #[arg(long)]
    reworks: PathBuf,
    #[arg(long)]
    output: PathBuf,
    /// A game patch above this, such as 16.24, makes the table stale.
    #[arg(long, value_parser = parse_patch)]
    max_patch: Option<Patch>,
}

#[derive(Args)]
struct PublishArgs {
    #[arg(long, default_value = "/var/lib/lollipop/collector.sqlite3")]
    database: PathBuf,
    /// The repository's `crates/collector/reworks.json`.
    #[arg(long)]
    reworks: PathBuf,
    /// Publishes this built file in place of a new build, behind the same gate.
    #[arg(long, conflicts_with = "max_patch")]
    table: Option<PathBuf>,
    /// A game patch above this, such as 16.24, makes the table stale.
    #[arg(long, value_parser = parse_patch)]
    max_patch: Option<Patch>,
    #[arg(long, default_value = REPO)]
    repo: String,
    #[arg(long, default_value = WORK_DIR)]
    work_dir: PathBuf,
}

#[derive(Args)]
struct RollbackArgs {
    /// The kept table the manifest should name.
    #[arg(long)]
    version: u32,
    #[arg(long, default_value = REPO)]
    repo: String,
    #[arg(long, default_value = WORK_DIR)]
    work_dir: PathBuf,
}

fn parse_patch(value: &str) -> Result<Patch, String> {
    Patch::parse(value).ok_or_else(|| format!("{value} is not a patch such as 16.24"))
}

#[allow(
    clippy::future_not_send,
    reason = "runs on a current-thread runtime, and a rusqlite Connection is not Sync"
)]
async fn run_collect(args: CollectArgs) -> Result<(), String> {
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

#[allow(
    clippy::print_stdout,
    reason = "the drift report and the schedule check are this command's output"
)]
fn run_build_table(args: BuildTableArgs) -> Result<(), String> {
    let config = TableConfig {
        database: args.database,
        reworks: args.reworks,
        output: args.output,
        max_patch: args.max_patch,
    };
    let report = build_table(&config).map_err(|error| error.to_string())?;
    println!("{report}");
    let check = check_schedule(&config.database).map_err(|error| error.to_string())?;
    println!("{check}");
    Ok(())
}

#[allow(clippy::print_stdout, reason = "the outcome is this command's output")]
fn run_publish(args: PublishArgs) -> Result<(), String> {
    let config = PublishConfig {
        database: args.database,
        reworks: args.reworks,
        table: args.table,
        max_patch: args.max_patch,
        repo: args.repo,
        work_dir: args.work_dir,
    };
    let published = publish(&config).map_err(|error| error.to_string())?;
    println!("{published}");
    Ok(())
}

fn run_rollback(args: RollbackArgs) -> Result<(), String> {
    let config = RollbackConfig {
        version: args.version,
        repo: args.repo,
        work_dir: args.work_dir,
    };
    rollback(&config).map_err(|error| error.to_string())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    match Cli::parse().command {
        Command::Collect(args) => run_collect(args).await,
        Command::BuildTable(args) => run_build_table(args),
        Command::Publish(args) => run_publish(args),
        Command::Rollback(args) => run_rollback(args),
    }
}
