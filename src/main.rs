mod config;
mod db;

use std::{error::Error};

use config::get_config;
use db::connect;
use tiberius::Row;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("PolyMix Updater v1.0");
    let config = get_config("settings.json");
    match config {
        Ok(config) => start_update(config).await?,
        Err(error) => println!("{}", error)
    }
    println!("Done");

    Ok(())
}

async fn start_update(config: config::Config) -> Result<(), Box<dyn Error>> {

    let mut client = connect(&config.connection_string).await?;
    let stream = client.query("select * from PolyCalcVersion", &[]).await?;

    let result: Vec<Option<String>> = stream
        .into_first_result()
        .await?
        .iter()
        .map(|row| try_get_string(&row, "FileName"))
        .collect();

    println!("{} file(s) to check", result.len());
    for file_name in result {
        println!("Checking {}: up-to-date", file_name.unwrap_or_default());
    }

    Ok(())
}

fn try_get_string(row: &Row, col: &str) -> Option<String> {
    // Error values are converted to empty strings
    row.try_get::<&str, &str>(col)
       .map(|value| value.map(|s| s.to_string()))
       .unwrap_or_default()
       //.map_or_else(|_| Some("".to_string()), |value| value.map(|s| s.to_string()))
}