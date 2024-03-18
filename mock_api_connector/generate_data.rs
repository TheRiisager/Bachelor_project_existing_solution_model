use tokio_postgres::{NoTls, Error};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=data", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client.query(
        "CREATE TABLE IF NOT EXISTS data(col1 SERIAL PRIMARY KEY, col2 TEXT, col3 TEXT);",
        &[]).await?;

    let statement = client.prepare(
        "INSERT INTO data (col2, col3) VALUES ($1, $2)"
        ).await?;
    
    for _ in 0..100000000 {
        let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

        let rand_string2: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

        client.query(&statement, &[&rand_string, &rand_string2]).await?;
    }

    Ok(())
}