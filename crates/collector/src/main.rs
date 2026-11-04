use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use lollipop_collector::{
    build_table, check_schedule, collect, manifest_public_key, publish, rollback, Config,
    PublishConfig, RollbackConfig, SeedTier, SystemClock, TableConfig, DATA_DRAGON_URL,
    MIN_CANDIDATE_GAMES,
};
use lollipop_core::game_inputs::Patch;

const REPO: &str = "kryptamine/lollipop";
const WORK_DIR: &str = "/var/lib/lollipop/publish";
const BUCKET: &str = "lollipop-assets";
const SIGNING_KEY: &str = "/etc/lollipop/manifest-key";

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
    /// Prints the public key the app compiles in for a signing key.
    ManifestPublicKey(ManifestPublicKeyArgs),
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
    #[command(flatten)]
    items: ItemArgs,
}

#[derive(Args)]
struct ItemArgs {
    /// Serves `api/versions.json` and each version's `item.json`.
    #[arg(long, default_value = DATA_DRAGON_URL)]
    data_dragon_url: String,
    /// A damage profile bucket with fewer games takes the list pooled over buckets.
    #[arg(long, default_value_t = MIN_CANDIDATE_GAMES)]
    min_candidate_games: usize,
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
    #[arg(long, default_value = BUCKET)]
    bucket: String,
    #[arg(long, default_value = WORK_DIR)]
    work_dir: PathBuf,
    #[arg(long, default_value = SIGNING_KEY)]
    signing_key_file: PathBuf,
    #[command(flatten)]
    items: ItemArgs,
}

#[derive(Args)]
struct RollbackArgs {
    /// The kept table the manifest should name.
    #[arg(long)]
    version: u32,
    #[arg(long, default_value = BUCKET)]
    bucket: String,
    #[arg(long, default_value = WORK_DIR)]
    work_dir: PathBuf,
    #[arg(long, default_value = SIGNING_KEY)]
    signing_key_file: PathBuf,
}

#[derive(Args)]
struct ManifestPublicKeyArgs {
    #[arg(long, default_value = SIGNING_KEY)]
    signing_key_file: PathBuf,
}

fn parse_patch(value: &str) -> Result<Patch, String> {
    Patch::parse(value).ok_or_else(|| format!("{value} is not a patch such as 16.24"))
}

fn run_collect(args: CollectArgs) -> Result<(), Box<dyn Error>> {
    let key = std::fs::read_to_string(&args.key_file)
        .map_err(|error| format!("cannot read {}: {error}", args.key_file.display()))?;
    let config = Config {
        tier: args.tier,
        key: key.trim().to_string(),
        database: args.database,
        platform_url: args.platform_url,
        regional_url: args.regional_url,
    };
    // Only this command runs one: Data Dragon's blocking client panics inside a runtime.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("cannot start the runtime: {error}"))?;
    Err(runtime.block_on(collect(&config, &SystemClock)).into())
}

#[allow(
    clippy::print_stdout,
    reason = "the drift report and the schedule check are this command's output"
)]
fn run_build_table(args: BuildTableArgs) -> Result<(), Box<dyn Error>> {
    let config = TableConfig {
        database: args.database,
        reworks: args.reworks,
        output: args.output,
        max_patch: args.max_patch,
        data_dragon_url: args.items.data_dragon_url,
        min_candidate_games: args.items.min_candidate_games,
    };
    let report = build_table(&config)?;
    println!("{report}");
    let check = check_schedule(&config.database)?;
    println!("{check}");
    Ok(())
}

#[allow(clippy::print_stdout, reason = "the outcome is this command's output")]
fn run_publish(args: PublishArgs) -> Result<(), Box<dyn Error>> {
    let config = PublishConfig {
        database: args.database,
        reworks: args.reworks,
        table: args.table,
        max_patch: args.max_patch,
        repo: args.repo,
        bucket: args.bucket,
        work_dir: args.work_dir,
        signing_key: args.signing_key_file,
        data_dragon_url: args.items.data_dragon_url,
        min_candidate_games: args.items.min_candidate_games,
    };
    let published = publish(&config)?;
    println!("{published}");
    Ok(())
}

fn run_rollback(args: RollbackArgs) -> Result<(), Box<dyn Error>> {
    let config = RollbackConfig {
        version: args.version,
        bucket: args.bucket,
        work_dir: args.work_dir,
        signing_key: args.signing_key_file,
    };
    Ok(rollback(&config)?)
}

#[allow(clippy::print_stdout, reason = "the key is this command's output")]
fn run_manifest_public_key(args: &ManifestPublicKeyArgs) -> Result<(), Box<dyn Error>> {
    let key = manifest_public_key(&args.signing_key_file)?;
    println!("{key}");
    Ok(())
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Command::Collect(args) => run_collect(args),
        Command::BuildTable(args) => run_build_table(args),
        Command::Publish(args) => run_publish(args),
        Command::Rollback(args) => run_rollback(args),
        Command::ManifestPublicKey(args) => run_manifest_public_key(&args),
    }
}

#[allow(
    clippy::print_stderr,
    reason = "a failure is reported to whoever ran the command"
)]
fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
