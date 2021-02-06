use chrono::{DateTime, Utc};
use std::error::Error;
use tiberius::{Client, Config};
use tiberius::{Row, SqlBrowser};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub type SqlConnection = Client<Compat<TcpStream>>;

pub struct DB {
    client: SqlConnection,
}

pub struct DBFile {
    pub name: String,
    pub content: Option<Vec<u8>>,
}

impl DB {
    pub async fn connect(connection_string: &str) -> Result<DB, Box<dyn Error>> {
        let config = Config::from_ado_string(&connection_string)?;

        let tcp = TcpStream::connect_named(&config).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(config.clone(), tcp.compat_write()).await?;

        Ok(DB { client })
    }

    pub async fn insert_file_name(
        &mut self,
        file_name: &str,
        file_date: DateTime<Utc>,
    ) -> Result<(), Box<dyn Error>> {
        print!("Adding new file...");
        self.client
            .execute(
                "INSERT INTO PolyCalcVersion (FileName, FileDate) VALUES (@P1, @P2)",
                &[&file_name, &file_date],
            )
            .await?;
        println!("OK");
        Ok(())
    }

    pub async fn upload_file_content(
        &mut self,
        file_name: &str,
        file_date: DateTime<Utc>,
        content: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        print!("Uploading content...");
        self.client
            .execute(
                "UPDATE PolyCalcVersion set FileDate = @P1, FileImage = @P2 WHERE FileName = @P3",
                &[&file_date, &content, &file_name],
            )
            .await?;
        println!("OK");
        Ok(())
    }

    pub async fn get_db_files(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let stream = self
            .client
            .query("select FileName from PolyCalcVersion", &[])
            .await?;

        let result: Vec<String> = stream
            .into_first_result()
            .await?
            .iter()
            .map(|row| Self::try_get_string(&row, "FileName"))
            .filter_map(|file_name| file_name)
            .collect();

        Ok(result)
    }

    pub async fn get_db_files_with_content(&mut self) -> Result<Vec<DBFile>, Box<dyn Error>> {
        let stream = self
            .client
            .query("select FileName, FileImage from PolyCalcVersion", &[])
            .await?;

        let result: Vec<DBFile> = stream
            .into_first_result()
            .await?
            .iter()
            .map(|row| DBFile {
                name: Self::try_get_string(&row, "FileName").unwrap_or_default(),
                content: {
                    let data: Result<Option<&[u8]>, _> = row.try_get("FileImage");
                    //println!("{:?}", data);
                    let data = data.unwrap_or_default();
                    //data.take() TODO
                    data.map(|d| d.into())
                },
            })
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
