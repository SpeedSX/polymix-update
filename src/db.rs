use anyhow::Result;
use tiberius::FromSql;
use chrono::{DateTime, NaiveDateTime, Utc};
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

        let tcp = TcpStream::connect_named(&config).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(config.clone(), tcp.compat_write()).await?;

        Ok(DB { client })
    }

    pub async fn insert_file_name(
        &mut self,
        file_name: &str,
        file_date: DateTime<Utc>,
    ) -> Result<()> {
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
    ) -> Result<()> {
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

    async fn map_db_files(rows: &[Row]) -> Result<Vec<DBFile>> {
        rows.iter().map(Self::try_map_db_file).collect()
    }

    fn try_map_db_file(row: &Row) -> Result<DBFile> {
        Ok(DBFile {
            name: Self::try_get_string(&row, "FileName").unwrap_or_default(),
            date: Self::try_get_not_nullable(row, "FileDate")?, //row.try_get("FileDate")?.unwrap_or_else(|| {unreachable!()}), 
            content: {
                let data: Result<Option<&[u8]>, _> = row.try_get("FileImage");
                //println!("{:?}", data);
                let data = data.unwrap_or_default();
                //data.take() TODO
                data.map(|d| d.into())
            },
        })
    }

    fn try_get_string(row: &Row, col: &str) -> Option<String> {
        // Error values are converted to empty strings
        row.try_get::<&str, &str>(col)
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

    // fn try_get_date(row: &Row, col: &str) -> Option<NaiveDateTime> {
    //     row.try_get::<NaiveDateTime, &str>(col).ok().flatten()
    // }
}
