mod db;

use db::DBEntry;
use futures::future::join_all;
use lapin::{Connection, ConnectionProperties, options::{BasicConsumeOptions, BasicAckOptions}, types::FieldTable, ConsumerState};
use tokio::{time::{Instant, timeout}, task::{JoinHandle, JoinError}};
use tokio_stream::StreamExt;
use std::{error::Error, sync::OnceLock, fs::{File, self}, time::Duration};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::db::DBService;

static DB_SERVICE:OnceLock<DBService> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let address = "amqp://connector:coolpassword@127.0.0.1:5672";
    let connection = Connection::connect(&address, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    let mut consumer = channel
        .basic_consume(
            "api_data",
            "consumer_app",
            BasicConsumeOptions { 
                no_local: false, 
                no_ack: true, 
                exclusive: false, 
                nowait: false },
            FieldTable::default(),
        )
        .await?;
    println!("rabbit init complete");

    let serv = DBService::new().await;
    let _ = DB_SERVICE.set(serv);
    println!("db init complete");

    let mut tasks: Vec<JoinHandle<u128>> = Vec::new();
    let tasks_limit = 4;
    let semaphore = Arc::new(Semaphore::new(tasks_limit));
    while let Ok(Some(Ok(delivery))) = timeout(Duration::from_millis(100), consumer.next()).await {
        let start = Instant::now();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let task: JoinHandle<u128> = tokio::spawn(async move {
            let mut entry: DBEntry = serde_json::from_slice(&delivery.data).unwrap();
            let p = permit;
            let mut hasher = Sha256::new();
            hasher.update(&entry.col1);
            let hash1 = hasher.finalize();
            let res1 = base16ct::lower::encode_string(&hash1);

            let mut hasher2 = Sha256::new();
            hasher2.update(&entry.col2);
            let hash2 = hasher2.finalize();
            let res2 = base16ct::lower::encode_string(&hash2);

            entry.hash1 = Some(res1);
            entry.hash2 = Some(res2);

            let _ = DB_SERVICE
                .get()
                .expect("No DB service")
                .save(entry).await;
            drop(p);
            let duration = start.elapsed();
            println!("message processed in {:?}", duration);
            return duration.as_millis();
        });
        tasks.push(task);
    }
    let _ = semaphore.acquire_many(tasks_limit as u32).await.unwrap();
    show_duration(join_all(tasks).await);
    Ok(())
}

fn show_duration(results: Vec<Result<u128, JoinError>>) {
    let count: u128 = results.len() as u128;
    let mut sum = 0;
    for result in results {
        sum += result.unwrap();
    }
    let average = sum / count;
    println!("average task completion time: {:?} ms", average)
}