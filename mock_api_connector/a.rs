use lapin::{Connection, ConnectionProperties, Channel, options::{QueueDeclareOptions, BasicPublishOptions}, types::FieldTable, BasicProperties};

#[tokio::main]
async fn main() {
    let channel = match setupChannel().await {
        Ok(channel) => channel,
        Err(e) => {
            println!("error: {:?}", e);
            return;
        }
    };

    match publishMessage(String::from("hello world"), &channel).await {
        Ok(_) => println!("message sent"),
        Err(e) => println!("Error! {:?}", e),
    }
}


async fn setupChannel() -> Result<Channel, lapin::Error> {
    let address = "amqp://connector:coolpassword@127.0.0.1:5672";
    let connection = Connection::connect(&address, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    let queue = channel.queue_declare(
        "api_data", 
        QueueDeclareOptions::default(), 
        FieldTable::default()
    ).await?;

    return Ok(channel);
}

async fn publishMessage(msg: String, channel: &Channel) -> Result<(), lapin::Error> {
    let confirm = channel.basic_publish(
        "", 
        "api_data", 
        BasicPublishOptions::default(), 
        msg.as_bytes(), 
        BasicProperties::default()
    ).await?.await?;

    Ok(())
}
