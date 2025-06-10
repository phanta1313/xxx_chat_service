use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{StreamExt, SinkExt, stream::SplitSink};
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::Message;



type Sender = SplitSink<WebSocketStream<TcpStream>, Message>;
type PeerMap = Arc<Mutex<Vec<(Sender, SocketAddr)>>>;


#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:82".to_string();
    let listener = TcpListener::bind(&addr).await.unwrap();
    let peers: PeerMap = Arc::new(Mutex::new(Vec::new()));

    println!("WebSocket сервер запущен на {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let peers = peers.clone();
        tokio::spawn(handle_connection(stream, addr, peers));
    }
}


async fn handle_connection(stream: TcpStream, addr: SocketAddr, peers: PeerMap) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            return;
        }
    };

    let (write, mut read) = ws_stream.split();

    peers.lock().await.push((write, addr));

    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    if let Err(e) = broadcast(&peers, msg, addr).await {
                    }
                }
            }
            Err(e) => {
                break;
            }
        }
    }

    remove_peer(&peers, addr).await;
}


async fn broadcast(peers: &PeerMap, msg: Message, sender_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut peers = peers.lock().await;
    let mut failed_connections = Vec::new();

    for (peer, addr) in peers.iter_mut() {
        if *addr != sender_addr {
            if let Err(e) = peer.send(msg.clone()).await {
                failed_connections.push(*addr);
            }
        }
    }

    if !failed_connections.is_empty() {
        peers.retain(|(_, addr)| !failed_connections.contains(addr));
    }

    Ok(())
}


async fn remove_peer(peers: &PeerMap, addr: SocketAddr) {
    let mut peers = peers.lock().await;
    peers.retain(|(_, peer_addr)| *peer_addr != addr);
}