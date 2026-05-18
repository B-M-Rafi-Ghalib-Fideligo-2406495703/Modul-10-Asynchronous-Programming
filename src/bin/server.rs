use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebsocketStream};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    username: String,
    content: String,
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebsocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("New connection from Fide's Komputer {}", addr);
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            let text = msg.as_text().unwrap();
                            println!("From client {}: {}", addr, text);
                            
                            // Coba parse sebagai JSON (dari Yew client)
                            let formatted_msg = if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                                // Jika valid JSON, tambahkan IP/Port ke username
                                chat_msg.username = format!("{} ({})", chat_msg.username, addr);
                                serde_json::to_string(&chat_msg).unwrap()
                            } else {
                                // Jika teks biasa (dari terminal client), biarkan teks biasa dengan prepend IP
                                format!("{}: {}", addr, text)
                            };
                            
                            bcast_tx.send(formatted_msg)?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            msg = bcast_rx.recv() => {
                let msg = msg?;
                ws_stream.send(Message::text(msg)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                println!("Error processing connection: {}", e);
            }
            Ok::<_, Box<dyn Error + Send + Sync>>(())
        });
    }
}
