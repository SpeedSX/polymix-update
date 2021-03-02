use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use glob::{glob_with, MatchOptions, Pattern};
use std::{fs, path::PathBuf, process, time::SystemTime};

use crate::{command::Command, config::Config, db::DB};

pub struct Updater<'a> {
    config: &'a Config,
    command: Command,
    update_mode_name: String,
}

impl Updater<'_> {
    pub fn new<'a>(config: &'a Config, command: Command, update_mode_name: &str) -> Updater<'a> {
        Updater {
            config,
            command,
            update_mode_name: update_mode_name.to_owned(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        match self.command {
            Command::Upload => self.upload().await?,
            Command::Download => self.download().await?,
            Command::List => self.list().await?,
        }
        Ok(())
    }

    async fn process_files<'a, F, Fut>(&'a self, process_fn: F) -> Result<()>
    where
        F: FnOnce(&'a Self, String) -> Fut,
        Fut: std::future::Future<Output = Result<()>> + 'a,
    {
        match self.get_file_mask() {
            Some(pattern) => process_fn(self, pattern).await?,
            None => {
                // TODO: we do not handle default file masks at the moment
                process::exit(1)
            }
        }
        Ok(())
    }

    async fn upload(&self) -> Result<()> {
        self.process_files(Self::upload_files).await
    }

    async fn download(&self) -> Result<()> {
        self.process_files(Self::download_files).await
    }

    async fn list(&self) -> Result<()> {
        self.process_files(Self::list_files).await
    }

    async fn download_files(&self, pattern_str: String) -> Result<()> {
        let mut client = self.connect().await?;

        println!("Downloading files..."); // TODO do not download all files

        let db_files = client.get_db_files_with_content().await?;

        for db_file in db_files {
            let pattern = Pattern::new(&pattern_str)?;
            if pattern.matches(db_file.name.as_str()) {
                print!("{}...", db_file.name);
                match db_file.content {
                    Some(content) => {
                        fs::write(db_file.name, content)?;
                        println!("OK");
                    }
                    None => println!("Zero length, skipped"),
                }
            }
        }

        Ok(())
    }

    fn get_local_files(pattern_str: &str) -> Result<Vec<PathBuf>> {
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };
        // let paths: Result<Vec<_>> = pattern_str
        //     .split(';')
        //     .map(|pattern| glob_with(pattern, options))
        //     .collect();

        let mut paths: Vec<PathBuf> = vec![];
        for pattern in pattern_str.split(';') {
            for entry in glob_with(pattern, options)? {
                let path = entry?;
                if let Some(_) = path.file_name() {
                    paths.push(path);
                }
            }
        }

        paths.dedup();

        Ok(paths)
    }

    async fn upload_files(&self, pattern_str: String) -> Result<()> {
        let local_files = Self::get_local_files(&pattern_str)?;
        if local_files.is_empty() {
            return Ok(());
        }

        let mut client = self.connect().await?;

        let db_files: Vec<String> = client
            .get_db_files()
            .await?
            .iter()
            .map(|f| f.name.clone())
            .collect();

        for path in local_files {
            let file_name: String = path.to_string_lossy().into();
            let metadata = fs::metadata(&path)?;
            let last_modified = metadata.modified()?;

            if metadata.is_file() {
                println!(
                    "{}: Last modified {}, size {} bytes",
                    file_name,
                    Self::format_date_time(last_modified),
                    metadata.len(),
                );
            }

            let content = fs::read(&path)?;
            let file_date: DateTime<Utc> = last_modified.into();
            if db_files.contains(&file_name) {
                client
                    .update_file_content(&file_name, file_date, &content)
                    .await?;
            } else {
                client
                    .insert_file_with_content(&file_name, file_date, &content)
                    .await?;
            }
        }

        Ok(())
    }

    async fn list_files(&self, pattern_str: String) -> Result<()> {
        let mut client = self.connect().await?;

        let db_files = client.get_db_files().await?;

        println!();

        let mut match_count = 0;
        let pattern = Pattern::new(&pattern_str)?;

        for db_file in db_files {
            if pattern.matches(db_file.name.as_str()) {
                println!(
                    "{}\t{}",
                    db_file.name,
                    Self::format_db_date_time(db_file.date)
                );
                match_count += 1;
            }
        }

        println!("\n{} file(s)", match_count);

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

    async fn connect(&self) -> Result<DB> {
        Ok(DB::connect(self.config.connection_string.as_str()).await?)
    }

    fn format_date_time(system_time: SystemTime) -> String {
        let datetime: DateTime<Local> = system_time.into();
        datetime.format("%d/%m/%Y %T").to_string()
    }

    fn format_db_date_time(dt: NaiveDateTime) -> String {
        dt.format("%d/%m/%Y %T").to_string()
    }
}
