extern crate tokio;
extern crate tokio_tungstenite;

use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use bytes::{ByteOrder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;

use std::error::Error;
use std::io::{Cursor, Read};
use std::net::SocketAddr;
use futures::{SinkExt, StreamExt};

mod resolve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = std::env::args().nth(1).unwrap_or("0.0.0.0:18030".to_string());
    let address: SocketAddr = addr.parse::<SocketAddr>()?;

    let listener = TcpListener::bind(&address)
        .await
        .expect("监听端口失败");

    println!("Listening on: {}, pid: {}", address, std::process::id());

    loop {
        let (soccer_socket, soccer_addr) = listener.accept().await?;
        println!("\nAccept a connection from {}", soccer_addr);

        let ws = tokio_tungstenite::accept_async(soccer_socket)
            .await
            .expect("Error during the websocket handshake occurred");

        tokio::spawn(process(ws));
    }
}

async fn process(mut ws: WebSocketStream<TcpStream>) {
    // 读协议头
    let request_header_msg = ws.next().await.unwrap().unwrap();
    println!("request_header_msg: {:?}", request_header_msg);
    if !request_header_msg.is_binary() {
        return;
    }
    let request_header = request_header_msg.into_data();
    // 解析目标域名和目标端口
    let (dest_domain, dest_port) = decode_request_header(&request_header).unwrap();

    let dest_ip_addr = resolve::resolve_domain(&dest_domain).await.unwrap();

    let dest_addr = SocketAddr::new(dest_ip_addr, dest_port);
    println!("Resolved dest_addr: {}", dest_addr);

    let dest_stream = TcpStream::connect(dest_addr)
        .await
        .expect("Failed to connect to destination");

    let (dest_read, dest_write) = dest_stream.into_split();
    let (soccer_write, soccer_read) = ws.split();

    // dest ==> soccer
    tokio::spawn(async move {
        transfer::tcp_to_ws(dest_read, soccer_write).await;
    });

    // soccer ===> dest
    transfer::ws_to_tcp(soccer_read, dest_write).await;
}

fn decode_request_header(request_header: &[u8]) -> std::io::Result<(String, u16)> {
    let mut rdr = Cursor::new(request_header);

    let port = ReadBytesExt::read_u16::<BE>(&mut rdr)?;
    println!("port: {}", port);

    let _reserved = ReadBytesExt::read_u16::<BE>(&mut rdr)?;
    println!("reserved: {}", _reserved);

    let len = ReadBytesExt::read_u16::<BE>(&mut rdr)?;
    println!("len: {}", len);

    let mut buf = Vec::with_capacity(len as usize);
    buf.resize(len as usize, 0);
    std::io::Read::read_exact(&mut rdr, &mut buf)?;

    let domain = String::from_utf8_lossy(&buf).to_string();
    println!("domain: {}", domain);

    return Ok((domain, port));
}
