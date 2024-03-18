use serde::Deserialize;
use tokio_postgres::{Client, Error, NoTls, Statement};

#[derive(Deserialize, Debug)]
pub struct DBEntry {
    pub id: i32,
    pub col1: String,
    pub col2: String,
}

pub struct DBService {
    pub client: Client,
    pub statement: Statement
}

impl DBService {
    pub async fn new() -> Self {
        let (client, connection) = match tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=data port=5431", NoTls).await {
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
            "CREATE TABLE IF NOT EXISTS data(id Integer PRIMARY KEY, col2 TEXT, col3 TEXT);",&[]).await {
                Ok(res) => res,
                Err(e) => panic!("{:?}", e),
            };

        let statement = match client.prepare("INSERT INTO data (id, col2, col3) VALUES ($1, $2, $3);").await {
            Ok(statement) => statement,
            Err(e) => panic!("{:?}", e),
        };
    
        Self {
            client,
            statement
        }
    }

    pub async fn save(&self, entry: DBEntry) -> Result<(), Error> {
        let _query = self.client.query(&self.statement, 
        &[
            &entry.id, 
            &entry.col1, 
            &entry.col2
        ])
        .await?;

        Ok(())
    }
}