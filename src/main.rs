use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use futures_util::{StreamExt, SinkExt}; 
use serde_json::Value; 

#[tokio::main]
async fn main() {
    let url = Url::parse("ws://localhost:8080/v1/graphql").expect("Failed to parse URL");

    let (mut socket, response) =
        connect_async(url).await.expect("Failed to connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());

    let init_message = r#"{"type":"connection_init"}"#;
    socket.send(Message::Text(init_message.into())).await.expect("Failed to send init message");

    if let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => println!("Received after init: {}", text),
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error during receive after init: {:?}", e);
                return;
            }
        }
    }

    let graphql_query = r#"{"id":"1","type":"start","payload":{"query":"query MyQuery { Balance { id } }"}}"#;
    socket.send(Message::Text(graphql_query.into())).await.expect("Failed to send query");

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received message: {}", text);
                let data: Value = serde_json::from_str(&text).unwrap_or_default();
                if let Some(msg_type) = data["type"].as_str() {
                    match msg_type {
                        "ka" => println!("Received keep-alive signal."),
                        "data" => println!("Received data: {:?}", data["payload"]),
                        "error" => println!("Error received: {}", data),
                        "complete" => {
                            println!("Query complete.");
                            break;
                        },
                        _ => println!("Received other type of message: {}", msg_type),
                    }
                }
            },
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error during receive: {:?}", e);
                break;
            }
        }
    }
}
