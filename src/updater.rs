use std::{error::Error, fs, time::SystemTime};
use chrono::{DateTime, Local, Utc};
use glob::glob;
use tiberius::{Row};

use crate::{config::Config, db::{SqlConnection, connect}};

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
        let mut client = self.get_connection().await?;

        let db_files = self.get_db_files(&mut client).await?;

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
                self.insert_file_name(&mut client, file_name, file_date).await?;
            }
            self.upload_file_content(&mut client, file_name, file_date, content).await?;
        }

        Ok(())
    }

    async fn insert_file_name(&self, client: &mut SqlConnection, file_name: &str, file_date: DateTime<Utc>) -> Result<(), Box<dyn Error>> {
        print!("Adding new file...");
        client.execute("INSERT INTO PolyCalcVersion (FileName, FileDate) VALUES (@P1, @P2)", &[&file_name, &file_date]).await?;
        println!("OK");
        Ok(())
    }

    async fn upload_file_content(&self, client: &mut SqlConnection, file_name: &str, file_date: DateTime<Utc>, content: Vec<u8>) -> Result<(), Box<dyn Error>> {
        print!("Uploading content...");
        client.execute("UPDATE PolyCalcVersion set FileDate = @P1, FileImage = @P2 WHERE FileName = @P3", &[&file_date, &content, &file_name]).await?;
        println!("OK");
        Ok(())
    }

    fn format_time(system_time: SystemTime) -> String {
        let datetime: DateTime<Local> = system_time.into();
        datetime.format("%d/%m/%Y %T").to_string()
    }

    async fn get_connection(&self) -> Result<SqlConnection, Box<dyn Error>> {
        let connection = connect(&self.config.connection_string).await?;
        Ok(connection)
    }

    async fn get_db_files(&self, client: &mut SqlConnection) -> Result<Vec<String>, Box<dyn Error>> {
        let stream = client.query("select * from PolyCalcVersion", &[]).await?;

        let result: Vec<String> = stream
            .into_first_result()
            .await?
            .iter()
            .map(|row| Self::try_get_string(&row, "FileName"))
            .filter_map(|file_name| file_name)
            .collect();

        Ok(result)
    }

    fn try_get_string(row: &Row, col: &str) -> Option<String> {
        // Error values are converted to empty strings
        row.try_get::<&str, &str>(col)
        .map(|value| value.map(|s| s.to_string()))
        .unwrap_or_default()
        //.map_or_else(|_| Some("".to_string()), |value| value.map(|s| s.to_string()))
    }
}