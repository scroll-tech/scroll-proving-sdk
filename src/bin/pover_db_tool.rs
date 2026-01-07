use clap::{Parser, Subcommand};
use eyre::WrapErr;
use scroll_proving_sdk::db::{Db, PROVING_TASK_ID_KEY_PREFIX};
use scroll_proving_sdk::utils::init_color_eyre_hook;
use std::collections::HashMap;
use std::path::PathBuf;

/// Tool to interact with the prover database.
#[derive(Parser)]
struct Cli {
    /// Path to the database
    #[clap(long, default_value = ".work/db")]
    db: PathBuf,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all coordinator and prover task ID pairs
    List,
    /// Get the corresponding task ID
    Get {
        #[command(subcommand)]
        commands: GetCommands,
    },
}

#[derive(Subcommand)]
enum GetCommands {
    /// Get coordinator task ID by prover task ID
    Coordinator { task_id: String },
    /// Get prover task ID by coordinator task ID
    Prover { task_id: String },
}

fn main() -> eyre::Result<()> {
    init_color_eyre_hook();

    let cli = Cli::parse();
    let db = Db::new(&cli.db).context("open database")?;

    let mut c2p = HashMap::new();
    let mut p2c = HashMap::new();

    for result in db.inner().prefix_iterator(PROVING_TASK_ID_KEY_PREFIX) {
        let (proving_task_key_bytes, _) = result.context("iter entry")?;
        let public_key =
            std::str::from_utf8(&proving_task_key_bytes[PROVING_TASK_ID_KEY_PREFIX.len()..])?;

        let Some((coordinator_task, prover_task_id)) = db.get_task(public_key) else {
            continue;
        };

        c2p.insert(coordinator_task.task_id.clone(), prover_task_id.clone());
        p2c.insert(prover_task_id, coordinator_task.task_id);
    }

    match cli.commands {
        Commands::List => {
            for (c_task_id, p_task_id) in c2p.iter() {
                println!("{}\t{}", c_task_id, p_task_id);
            }
        }
        Commands::Get { commands } => match commands {
            GetCommands::Coordinator { task_id } => {
                if let Some(coordinator_task_id) = p2c.get(&task_id) {
                    println!("{coordinator_task_id}");
                }
            }
            GetCommands::Prover { task_id } => {
                if let Some(prover_task_id) = c2p.get(&task_id) {
                    println!("{prover_task_id}");
                }
            }
        },
    }

    Ok(())
}
