mod db;

use db::{get_entries, setup_db_connection, DBEntry};
use lapin::{Connection, ConnectionProperties, Channel, options::{QueueDeclareOptions, BasicPublishOptions}, types::FieldTable, BasicProperties};
use tokio::join;

#[tokio::main]
async fn main() {
    let (channel, channel_2) = match setup_channel().await {
        Ok(channel) => channel,
        Err(e) => {
            println!("error: {:?}", e);
            return;
        }
    };

    let db_client = match setup_db_connection().await {
        Ok(client) => client,
        Err(e) => {
            println!("error! {:?}", e);
            return;
        }
    };

    let mut entries = match get_entries(&db_client).await {
        Ok(entries) => entries,
        Err(e) => {
            println!("Error! {:?}", e);
            return;
        },        
    };

    println!("entries: {:?}", entries.len());

    let entries_2 = entries.split_off((entries.len() / 2) - 1);

    let first_half = tokio::spawn(async move {
        for entry in entries {
            match publish_message(entry_to_json(&entry), &channel).await {
                Ok(_) => println!("message sent for entry {:?} from thread 1", entry.id),
                Err(e) => println!("Error! {:?}", e),
            }
        }
    });

    let second_half = tokio::spawn(async move {
        for entry in entries_2 {
            match publish_message(entry_to_json(&entry), &channel_2).await {
                Ok(_) => println!("message sent for entry {:?} from thread 2", entry.id),
                Err(e) => println!("Error! {:?}", e),
            }
        }
    });

    join!(second_half, first_half);
}


async fn setup_channel() -> Result<(Channel, Channel), lapin::Error> {
    let address = "amqp://connector:coolpassword@127.0.0.1:5672";
    let connection = Connection::connect(&address, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    let channel_2 = connection.create_channel().await?;

    let _queue = channel.queue_declare(
        "api_data", 
        QueueDeclareOptions::default(), 
        FieldTable::default()
    ).await?;

    return Ok((channel, channel_2));
}

async fn publish_message(msg: String, channel: &Channel) -> Result<(), lapin::Error> {
    let _confirm = channel.basic_publish(
        "", 
        "api_data", 
        BasicPublishOptions::default(), 
        msg.as_bytes(), 
        BasicProperties::default()
    ).await?.await?;

    Ok(())
}

fn entry_to_json(entry: &DBEntry) -> String {
    match serde_json::to_string(entry) {
        Ok(json) => json, 
        Err(_) => "".into(),
    }
}

