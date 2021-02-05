mod config;
mod db;
mod updater;

use std::{env, error::Error};

use config::get_config;
use updater::Updater;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("PolyMix Updater v1.0  (c) 2021 PolyMix Development Group.\n");
    let mut args = env::args();
    if args.len() != 2 {
        println!("Usage: polymix-update <update-mode>");
    } else {
        let config = get_config("settings.json");
        match config {
            Ok(config) => Updater::new(&config, &args.nth(1).unwrap_or_default()).upload().await?,
            Err(error) => println!("{}", error)
        }
        println!("\nDone");
    }

    Ok(())
}
