mod command;
mod config;
mod db;
mod updater;

use std::{env, error::Error, process, str::FromStr};

use command::Command;
use config::get_config;
use updater::Updater;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await?;
    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    println!("PolyMix Updater v0.1  (c) 2021 PolyMix Development Group.\nUse to work (update, download, etc.) with file images stored in database.\n");
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        process::exit(1);
    } else {
        // try parsing command
        let command = Command::from_str(&args[1]);
        match command {
            Ok(command) => {
                // try reading configuration
                let config = get_config("settings.json");
                match config {
                    Ok(config) => Updater::new(&config, command, &args[2]).run().await?,
                    Err(error) => {
                        println!("{}", error);
                        process::exit(2);
                    }
                }
            }
            Err(_) => {
                print_usage();
                process::exit(1);
            }
        }
        println!("\nDone");
    }

    Ok(())
}

fn print_usage() {
    println!("USAGE:
    \tpolymix-update [COMMAND] [mode] [FLAGS]\n");
    println!("COMMANDS:
    \tupload\t\tWrite files from current directory to database
    \tdownload\tRead files from database and store in current directory
    \tlist\t\tlist files stored in database\n");
}
