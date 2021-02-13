use chrono::{DateTime, Local, Utc};
use glob::{glob_with, MatchOptions, Pattern};
use std::{error::Error, fs, process, time::SystemTime};

use crate::{command::Command, config::Config, db::DB};

pub struct Updater<'a> {
    config: &'a Config,
    command: Command,
    update_mode_name: String,
}

impl Updater<'_> {
    pub fn new<'a>(config: &'a Config, command: Command, update_mode_name: &str) -> Updater<'a> {
        Updater {
            config: config,
            command: command,
            update_mode_name: update_mode_name.to_owned(),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        match self.command {
            Command::Upload => self.upload().await?,
            Command::Download => self.download().await?,
        }
        Ok(())
    }

    async fn upload(&self) -> Result<(), Box<dyn Error>> {
        match self.get_file_mask() {
            Some(pattern) => self.upload_files(&pattern).await?,
            None => {
                // TODO: we do not handle default file masks at the moment
                process::exit(1)
            }
        }
        Ok(())
    }

    async fn download(&self) -> Result<(), Box<dyn Error>> {
        match self.get_file_mask() {
            Some(pattern) => self.download_files(&pattern).await?,
            None => {
                // TODO: we do not handle default file masks at the moment
                process::exit(1)
            }
        }
        Ok(())
    }

    async fn download_files(&self, pattern_str: &str) -> Result<(), Box<dyn Error>> {
        let mut client = DB::connect(self.config.connection_string.as_str()).await?;

        println!("Downloading files..."); // TODO do not download all files

        let db_files = client.get_db_files_with_content().await?;

        for db_file in db_files {
            let pattern = Pattern::new(pattern_str)?;
            if pattern.matches(db_file.name.as_str()) {
                print!("{}...", db_file.name);
                match db_file.content {
                    Some(content) => {
                        // TODO use write_all
                        fs::write(db_file.name, content)?;
                        println!("OK");
                    }
                    None => println!("Zero length, skipped"),
                }
            }
        }

        Ok(())
    }

    async fn upload_files(&self, pattern_str: &str) -> Result<(), Box<dyn Error>> {
        let mut client = self.connect().await?;

        let db_files = client.get_db_files().await?;

        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };
        for entry in glob_with(pattern_str, options)? {
            let path = entry?;

            let metadata = fs::metadata(&path)?;
            let last_modified = metadata.modified()?;

            let file_name = path.file_name().map(|f| f.to_string_lossy()).ok_or("-")?;

            if metadata.is_file() {
                println!(
                    "{}: Last modified {}, size {} bytes",
                    file_name,
                    Self::format_time(last_modified),
                    metadata.len(),
                );
            }

            let content = fs::read(&path)?;
            let file_name: &String = &file_name.into();
            let file_date: DateTime<Utc> = last_modified.into();
            if !db_files.contains(file_name) {
                client.insert_file_name(file_name, file_date).await?;
            }
            client
                .upload_file_content(file_name, file_date, content)
                .await?;
        }

        Ok(())
    }

    fn get_file_mask(&self) -> Option<String> {
        let update_mode = self.update_mode_name.to_lowercase();
        // TODO  Default mode is not implemented
        let update_mode = self
            .config
            .update_mode
            .iter()
            .find(|mode| mode.name.to_lowercase() == update_mode);

        // TODO Use map
        match update_mode {
            Some(mode) => Some(mode.file_mask.to_owned()),
            None => {
                println!(
                    "'{}' update mode not found in configuration file",
                    self.update_mode_name
                );
                None
            }
        }
    }

    async fn connect(&self) -> Result<DB, Box<dyn Error>> {
        Ok(DB::connect(self.config.connection_string.as_str()).await?)
    }

    fn format_time(system_time: SystemTime) -> String {
        let datetime: DateTime<Local> = system_time.into();
        datetime.format("%d/%m/%Y %T").to_string()
    }
}
