mod command;
mod config;
mod db;
mod updater;

use anyhow::{Result, anyhow};
use std::{env, process, str::FromStr};

use command::Command;
use config::get;
use updater::Updater;

#[derive(Debug)]
struct ParsedArgs {
    command: Command,
    mode: String,
    sql_username: Option<String>,
    sql_password: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}

async fn run() -> Result<()> {
    println!(
        "PolyMix Updater v0.2  (c) 2021-2026 PolyMix Development Group.\nUse to work (update, download, etc.) with file images stored in database.\n"
    );
    let args: Vec<_> = env::args().collect();
    let parsed_args = match parse_args(&args) {
        Ok(parsed_args) => parsed_args,
        Err(error) => {
            println!("{error}");
            println!();
            print_usage();
            process::exit(1);
        }
    };

    // try reading configuration
    let config = get("settings.json");
    match config {
        Ok(config) => {
            Updater::new(
                &config,
                parsed_args.command,
                &parsed_args.mode,
                parsed_args.sql_username,
                parsed_args.sql_password,
            )
            .run()
            .await?;
        }
        Err(error) => {
            println!("{error}");
            process::exit(2);
        }
    }

    Ok(())
}

fn parse_args(args: &[String]) -> Result<ParsedArgs> {
    if args.len() < 3 {
        return Err(anyhow!("Not enough arguments."));
    }

    let command = Command::from_str(&args[1]).map_err(|()| anyhow!("Unknown command '{}'.", args[1]))?;
    let mode = args[2].clone();

    let mut sql_username: Option<String> = None;
    let mut sql_password: Option<String> = None;

    let mut index = 3;
    while index < args.len() {
        let arg = &args[index];

        if let Some(value) = arg.strip_prefix("--sql-user=") {
            if value.is_empty() {
                return Err(anyhow!("Flag '--sql-user' requires a non-empty value."));
            }
            if sql_username.replace(value.to_string()).is_some() {
                return Err(anyhow!("Flag '--sql-user' was provided more than once."));
            }
            index += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--sql-password=") {
            if value.is_empty() {
                return Err(anyhow!("Flag '--sql-password' requires a non-empty value."));
            }
            if sql_password.replace(value.to_string()).is_some() {
                return Err(anyhow!("Flag '--sql-password' was provided more than once."));
            }
            index += 1;
            continue;
        }

        match arg.as_str() {
            "--sql-user" | "--user" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| anyhow!("Flag '--sql-user' requires a value."))?;
                if sql_username.replace(value.to_string()).is_some() {
                    return Err(anyhow!("Flag '--sql-user' was provided more than once."));
                }
            }
            "--sql-password" | "--password" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| anyhow!("Flag '--sql-password' requires a value."))?;
                if sql_password.replace(value.to_string()).is_some() {
                    return Err(anyhow!("Flag '--sql-password' was provided more than once."));
                }
            }
            _ => return Err(anyhow!("Unknown argument '{arg}'.")),
        }

        index += 1;
    }

    if sql_username.is_some() ^ sql_password.is_some() {
        return Err(anyhow!(
            "Both '--sql-user' and '--sql-password' must be provided together."
        ));
    }

    Ok(ParsedArgs {
        command,
        mode,
        sql_username,
        sql_password,
    })
}

fn print_usage() {
    println!(
        "USAGE:
    	polymix-update [COMMAND] [mode] [FLAGS]\n"
    );
    println!(
        "COMMANDS:
    \tupload\t\tWrite files from current directory to database
    \tdownload\tRead files from database and store in current directory
    \tlist\t\tlist files stored in database\n"
    );
    println!(
        "FLAGS:
    	--sql-user, --user <name>\t\tSQL Server user name (requires --sql-password)
    	--sql-password, --password <value>\tSQL Server password (requires --sql-user)\n"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parse_args_supports_sql_credentials() {
        let parsed = parse_args(&v(&[
            "polymix-update",
            "list",
            "xls",
            "--sql-user",
            "sa",
            "--sql-password",
            "secret",
        ]))
        .unwrap();

        assert_eq!(parsed.command, Command::List);
        assert_eq!(parsed.mode, "xls");
        assert_eq!(parsed.sql_username.as_deref(), Some("sa"));
        assert_eq!(parsed.sql_password.as_deref(), Some("secret"));
    }

    #[test]
    fn parse_args_supports_inline_sql_credentials() {
        let parsed = parse_args(&v(&[
            "polymix-update",
            "download",
            "xls",
            "--sql-user=sa",
            "--sql-password=secret",
        ]))
        .unwrap();

        assert_eq!(parsed.command, Command::Download);
        assert_eq!(parsed.sql_username.as_deref(), Some("sa"));
        assert_eq!(parsed.sql_password.as_deref(), Some("secret"));
    }

    #[test]
    fn parse_args_requires_both_sql_credentials() {
        let error = parse_args(&v(&["polymix-update", "upload", "xls", "--sql-user", "sa"]))
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Both '--sql-user' and '--sql-password' must be provided together.")
        );
    }
}
