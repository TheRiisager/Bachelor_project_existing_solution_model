use serde::Deserialize;
use tokio_postgres::{Client, Error, NoTls, Statement};

#[derive(Deserialize, Debug)]
pub struct AuditEntry {
    pub hash1: String,
    pub hash2: String
}

pub struct AuditDBService {
    pub client: Client,
    pub statement: Statement
}

impl AuditDBService {
    pub async fn new() -> Self {
        let (client, connection) = match tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=data port=5430", NoTls).await {
            Ok((client, connection)) => (client, connection),
            Err(e) => panic!("{:?}", e),
        };
        println!("client and connection established");
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("db connection error: {}", e);
            }
        });

        let _ = match client.query(
            "CREATE TABLE IF NOT EXISTS data(id Integer PRIMARY KEY, hash1 TEXT, hash2 TEXT);",&[]).await {
                Ok(res) => res,
                Err(e) => panic!("{:?}", e),
            };

        let statement = match client.prepare("INSERT INTO data (hash1, hash2) VALUES ($1, $2);").await {
            Ok(statement) => statement,
            Err(e) => panic!("{:?}", e),
        };
    
        Self {
            client,
            statement
        }
    }

    pub async fn save(&self, entry: AuditEntry) -> Result<(), Error> {
        let _query = self.client.query(&self.statement, 
        &[
            &entry.hash1, 
            &entry.hash2, 
        ])
        .await?;

        Ok(())
    }
}