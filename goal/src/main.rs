extern crate tokio;
extern crate tokio_tungstenite;

use bytes::{Buf, BytesMut};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;

use log::{error, info};
use std::error::Error;
use std::net::{SocketAddr};
 use futures::{StreamExt};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;
use configuration::{GoalConfiguration};
use domain_name_query_types::NameQuery;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::builder()
        .parse_lossy("trace,tokio=debug,tokio_tungstenite=debug,tungstenite=debug");
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        // .with_max_level(tracing::level_filters::LevelFilter::DEBUG)
        .finish();
    subscriber.init();

    let config_path = "./goal_config.toml";
    let goal_config = configuration::get_config::<GoalConfiguration>(config_path).unwrap();
    tracing::debug!("goal configuration: {:?}", goal_config);

    let bind_address: SocketAddr = goal_config.server.bind_address();

    // 可能的错误提示：
    // 监听端口失败: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
    let listener = TcpListener::bind(&bind_address)
        .await
        .expect("监听失败 Failed to bind");
    tracing::info!("Listening on: {}, pid: {}", bind_address, std::process::id());

    let dns_configuration = goal_config.dns;
    let domain_name_handle = domain_name_actor::actor::ActorHandle::new(dns_configuration.server_address);

    loop {
        let (soccer_socket, soccer_addr) = match transfer::tcp_accept::tcp_accept(&listener).await {
            Some(conn) => conn,
            None => continue,
        };

        info!("Accept a connection from {}", soccer_addr);
        tokio::spawn(process(soccer_socket, domain_name_handle.clone()));
    }
}

async fn process(soccer_socket: TcpStream, domain_name_handle: domain_name_actor::actor::ActorHandle) {
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

    let dest_ip_addr_ret = domain_name_handle.query(NameQuery::a_record(dest_domain.as_str())).await;

    if dest_ip_addr_ret.is_none() {
        // TODO DNS 没有查找到不属于协议错误，应该用其他方式告知客户端
        // "Could not resolve host"
        return;
    }
    let dest_ip_addr = dest_ip_addr_ret.unwrap();

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
    let mut buf = BytesMut::from(request_header);

    let port = Buf::get_u16(&mut buf);
    println!("port: {}", port);

    let _reserved = Buf::get_u16(&mut buf);
    println!("reserved: {}", _reserved);

    let len = Buf::get_u16(&mut buf) as usize;
    println!("len: {}", len);

    // 如果使用 `Vec<u8>` 作为缓冲区，则需要分配并初始化内存，
    // 而使用 `BytesMut` 可以避免
    let domain_bytes = BytesMut::split_to(&mut buf, len);
    let domain = String::from_utf8_lossy(&domain_bytes).to_string();
    println!("domain: {}", domain);

    // if buf.has_remaining() {
    //     // buf should be empty
    // }

    Ok((domain, port))
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
            // https://datatracker.ietf.org/doc/html/rfc6455#section-1.3
            // https://datatracker.ietf.org/doc/html/rfc6455#section-4.2.2
            //
            // Any status code other than 101 indicates that the WebSocket handshake
            // has not completed and that the semantics of HTTP still apply.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_request_header_test() {
        let request_header_data = vec![
            0, 80, // port 80
            0, 0,
            0x0, 0x4, 122, 46, 99, 110, // "z.cn"
        ];

        let (domain, port) = decode_request_header(&request_header_data).unwrap();
        assert_eq!(domain, "z.cn");
        assert_eq!(port, 80);
    }
}
