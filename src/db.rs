use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use tiberius::FromSql;
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
    pub date: NaiveDateTime,
    pub content: Option<Vec<u8>>,
}

impl DB {
    pub async fn connect(connection_string: &str) -> Result<DB> {
        let config = Config::from_ado_string(&connection_string)?;

        println!("Connecting to server {}", config.get_addr());

        let tcp = TcpStream::connect_named(&config).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(config.clone(), tcp.compat_write()).await?;

        Ok(DB { client })
    }

    pub async fn insert_file_with_content(
        &mut self,
        file_name: &str,
        file_date: DateTime<Utc>,
        content: &[u8],
    ) -> Result<()> {
        print!("Adding new file...");

        self.client
            .execute(
                "INSERT INTO PolyCalcVersion (FileName, FileDate, FileImage) VALUES (@P1, @P2, @P3)",
                &[&file_name, &file_date, &content],
            )
            .await?;

        println!("OK");

        Ok(())
    }

    pub async fn update_file_content(
        &mut self,
        file_name: &str,
        file_date: DateTime<Utc>,
        content: &[u8],
    ) -> Result<()> {
        print!("Updating file content...");

        self.client
            .execute(
                "UPDATE PolyCalcVersion set FileDate = @P1, FileImage = @P2 WHERE FileName = @P3",
                &[&file_date, &content, &file_name],
            )
            .await?;

        println!("OK");

        Ok(())
    }

    pub async fn get_db_files(&mut self) -> Result<Vec<DBFile>> {
        let rows = self
            .client
            .query("select FileName, FileDate from PolyCalcVersion", &[])
            .await?
            .into_first_result()
            .await?;

        let result = Self::map_db_files(&rows).await?;

        Ok(result)
    }

    pub async fn get_db_files_with_content(&mut self) -> Result<Vec<DBFile>> {
        let rows = self
            .client
            .query(
                "select FileName, FileDate, FileImage from PolyCalcVersion",
                &[],
            )
            .await?
            .into_first_result()
            .await?;

        let result = Self::map_db_files(&rows).await?;

        Ok(result)
    }

    pub async fn get_db_file_content(&mut self, file_name: &str) -> Result<Option<Vec<u8>>> {
        let rows = self
            .client
            .query(
                "select FileImage from PolyCalcVersion where FileName = @P1",
                &[&file_name],
            )
            .await?
            .into_first_result()
            .await?;

        if let Some(content) = rows
            .first()
            .map(|row| Self::try_get_binary(row, "FileImage"))
        {
            content
        } else {
            bail!("File not found: {}", file_name)
        }
    }

    async fn map_db_files(rows: &[Row]) -> Result<Vec<DBFile>> {
        rows.iter().map(Self::try_map_db_file).collect()
    }

    fn try_map_db_file(row: &Row) -> Result<DBFile> {
        Ok(DBFile {
            name: Self::try_get_string(&row, "FileName").unwrap_or_default(),
            date: Self::try_get_not_nullable(row, "FileDate")?,
            content: Self::try_get_binary(&row, "FileImage").unwrap_or_default(), // this field is not always in result set
        })
    }

    fn try_get_string(row: &Row, col: &str) -> Option<String> {
        // Error values are converted to empty strings
        row.try_get::<&str, _>(col)
            .map(|value| value.map(|s| s.to_string()))
            .ok()
            .flatten()
        //.unwrap_or_default()
        //.map_or_else(|_| Some("".to_string()), |value| value.map(|s| s.to_string()))
    }

    // When field is not nullable, this should not fail in that case, only fail on conversion error
    fn try_get_not_nullable<'a, R: FromSql<'a>>(row: &'a Row, col: &str) -> Result<R> {
        Ok(row.try_get(col)?.unwrap_or_else(|| unreachable!()))
    }

    fn try_get_binary(row: &Row, col: &str) -> Result<Option<Vec<u8>>> {
        let data = row.try_get::<&[u8], _>(col)?;
        Ok(data.map(|d| d.into()))
    }
}
