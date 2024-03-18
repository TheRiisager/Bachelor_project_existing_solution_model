use postgres_types::{ToSql, FromSql};
use tokio_postgres::{NoTls, Error, Client};
use serde::Serialize;

pub async fn setup_db_connection() -> Result<Client, Error> {
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=data", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("db connection error: {}", e);
        }
    });
    
    Ok(client)
}

pub async fn get_entries(client: &Client) -> Result<Vec<DBEntry>, Error> {
    let query = client.query("SELECT * FROM data", &[]).await?;
    let mut entries: Vec<DBEntry> = Vec::new();

    for row in query {
        let entry= DBEntry {
            id: row.try_get("id")?,
            col1: row.try_get("col2")?,
            col2: row.try_get("col3")?
        };
        entries.push(entry);
    }

    Ok(entries)
}

#[derive(Debug, ToSql, FromSql, Serialize)]
pub struct DBEntry {
    pub id: i32,
    pub col1: String,
    pub col2: String
}