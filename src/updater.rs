use std::{error::Error, fs, time::SystemTime};
use chrono::{DateTime, Local, Utc};
use glob::glob;

use crate::{config::Config, db::DB};

pub struct Updater<'a> {
    config: &'a Config,
    update_mode_name: String,
}

impl Updater<'_> {
    pub fn new<'a>(config: &'a Config, update_mode_name: &str) -> Updater<'a> {
        Updater {
            config: config, 
            update_mode_name: update_mode_name.to_owned(),
        }
    }

    pub async fn upload(&self) -> Result<(), Box<dyn Error>> {
        //let current_dir = env::current_dir()?;
        let update_mode = self.update_mode_name.to_lowercase();
        // TODO  Default mode is not implemented
        let update_mode = self.config.update_mode.iter().find(|mode| mode.name.to_lowercase() == update_mode);
        match update_mode {
            Some(mode) => self.upload_files(&mode.file_mask).await?,
            None => println!("'{}' update mode not found in configuration file", self.update_mode_name)
        }
        Ok(())
    }

    async fn upload_files(&self, file_mask: &str) -> Result<(), Box<dyn Error>> {
        let mut client = DB::connect(&self.config.connection_string).await?;

        let db_files = client.get_db_files().await?;

        for entry in glob(file_mask)? {
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
            client.upload_file_content(file_name, file_date, content).await?;
        }

        Ok(())
    }

    fn format_time(system_time: SystemTime) -> String {
        let datetime: DateTime<Local> = system_time.into();
        datetime.format("%d/%m/%Y %T").to_string()
    }
}