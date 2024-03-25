extern crate tokio;
extern crate tokio_tungstenite;

use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;

use log::{error, info, warn};
use std::error::Error;
use std::io::{Cursor, Read};
use std::net::{SocketAddr};
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

mod resolve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init();

    let addr = std::env::args().nth(1).unwrap_or("0.0.0.0:18030".to_string());
    let address: SocketAddr = addr.parse::<SocketAddr>()?;

    // 可能的错误提示：
    // 监听端口失败: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
    let listener = TcpListener::bind(&address)
        .await
        .expect("监听失败 Failed to bind");
    info!("Listening on: {}, pid: {}", address, std::process::id());

    loop {
        let (soccer_socket, soccer_addr) = match transfer::tcp_accept::tcp_accept(&listener).await {
            Some(conn) => conn,
            None => continue,
        };

        info!("Accept a connection from {}", soccer_addr);
        tokio::spawn(process(soccer_socket));
    }
}

async fn process(soccer_socket: TcpStream) {
    let mut ws = match ws_accept(soccer_socket).await {
        Some(ws_stream) => ws_stream,
        None => return,
    };

    // 读协议头
    // TODO: How to handle `Option` and `Result`
    let request_header_msg_ret = ws.next().await;
    if request_header_msg_ret.is_none() {
        error!("Not received request header message");
        return;
    }
    let msg_ret = request_header_msg_ret.unwrap();
    if msg_ret.is_err() {
        error!("Failed to receive request message");
        return;
    }

    let request_header_msg = msg_ret.unwrap();
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
    let dest_to_soccer = tokio::spawn(async move {
        transfer::tcp_to_ws(dest_read, soccer_write).await;
    });

    // soccer ===> dest
    let soccer_to_dest = tokio::spawn(async move {
        transfer::ws_to_tcp(soccer_read, dest_write).await;
    });

    let _ = tokio::join!(dest_to_soccer, soccer_to_dest);
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

async fn ws_accept(tcp_stream: TcpStream) -> Option<WebSocketStream<TcpStream>> {
    use http::Response as HttpResponse;
    use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

    let callback = |req: &Request, resp: Response| {
        // req 示例:
        // Request { method: GET, uri: /goal, version: HTTP/1.1,
        //     headers: {
        //         "host": "127.0.0.1:18030",
        //         "connection": "Upgrade",
        //         "upgrade": "websocket",
        //         "sec-websocket-version": "13",
        //         "sec-websocket-key": "4qua2jXK9gwaSWDVWZZ8Ow=="
        //     },
        //     body: ()
        // }
        let path = req.uri().path();
        if path == "/goal" {
            Ok(resp)
        } else {
            let err_resp = HttpResponse::builder()
                .status(404)
                .body(None)
                .unwrap();
            Err(err_resp)
        }
    };

    match tokio_tungstenite::accept_hdr_async(tcp_stream, callback).await {
        Ok(ws_stream) => {
            Some(ws_stream)
        }
        Err(e) => {
            error!("Error during the websocket handshake occurred, err: {}", e);
            None
        }
    }
}
