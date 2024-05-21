mod command;
mod config;
mod db;
mod updater;

use anyhow::Result;
use std::{env, process, str::FromStr};

use command::Command;
use config::get;
use updater::Updater;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}

async fn run() -> Result<()> {
    println!("PolyMix Updater v0.2  (c) 2021 PolyMix Development Group.\nUse to work (update, download, etc.) with file images stored in database.\n");
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        process::exit(1);
    } else {
        // try parsing command
        let command = Command::from_str(&args[1]);
        if let Ok(command) = command {
            // try reading configuration
            let config = get("settings.json");
            match config {
                Ok(config) => Updater::new(&config, command, &args[2]).run().await?,
                Err(error) => {
                    println!("{error}");
                    process::exit(2);
                }
            }
        } else {
            print_usage();
            process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        "USAGE:
    \tpolymix-update [COMMAND] [mode] [FLAGS]\n"
    );
    println!(
        "COMMANDS:
    \tupload\t\tWrite files from current directory to database
    \tdownload\tRead files from database and store in current directory
    \tlist\t\tlist files stored in database\n"
    );
}
