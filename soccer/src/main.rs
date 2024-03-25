// https://www.ietf.org/rfc/rfc1928.txt

extern crate tokio;
extern crate tokio_tungstenite;

use bytes::{BufMut};
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use std::error::Error;
use std::net::SocketAddr;
use log::{error, info};

use soccer::Destination;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = env_logger::try_init();

    let server_address = std::env::args().nth(1).unwrap_or("127.0.0.1:18030".to_string());
    let addr = "127.0.0.1:8080".to_string();
    let addr = addr.parse::<SocketAddr>()?;

    let listener = TcpListener::bind(&addr)
        .await
        .expect("监听失败");
    info!("Listening on: {}, pid: {}", addr, std::process::id());

    loop {
        let (client_socket, client_addr) = listener.accept().await?;
        info!("Accept a connection from {}", client_addr);

        let goal_addr = server_address.clone();

        tokio::spawn(process(client_socket, goal_addr));
    }
}

async fn process(mut client_socket: TcpStream, goal_addr: String) {
    recv_method_selection_message(&mut client_socket).await.unwrap();
    send_method_selection_message(&mut client_socket).await.unwrap();

    let remote_dst: Destination = recv_request(&mut client_socket).await;
    let dest_str = remote_dst.to_str();
    println!("destination: {}", dest_str);

    send_reply(&mut client_socket).await.unwrap();

    let addr = ["ws://".to_string(), goal_addr, "/goal".to_string()].join("");

    // WebSocket handshake
    let ret = tokio_tungstenite::connect_async(addr).await;
    // 如果请求的路径与 goal 端的要求一致（比如 "/goal"），那么 goal 端会返回 101 Switching Protocols 相应
    // 如果不一致，那么 goal 端会返回 404 相应，此时只能终止处理流程
    if ret.is_err() {
        error!("Failed to connect to soccer, err: {:?}", ret);
        return;
    }

    let (goal_stream, _resp) = ret.unwrap();
    println!("resp: {:?}", _resp);
    let (mut goal_write, goal_read) = goal_stream.split();
    let (client_read, client_write) = client_socket.into_split();

    // Send request header
    let buf: Vec<u8> = encode_request_header(&remote_dst);
    println!("buf: {:?}", buf);
    let request_header_msg = Message::binary(buf);
    goal_write.send(request_header_msg).await.unwrap();

    // goal ==> client
    let goal_to_client = tokio::spawn(async move {
        transfer::ws_to_tcp(goal_read, client_write).await;
    });

    // client ==> goal
    let client_to_goal = tokio::spawn(async move {
        transfer::tcp_to_ws(client_read, goal_write).await;
    });

    let _ = tokio::join!(goal_to_client, client_to_goal);
}

fn encode_request_header(remote_dst: &Destination) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(100);

    // 2 bytes, port
    buf.put_u16_be(remote_dst.port());

    // 2 bytes, reserved
    buf.put_u16_be(0);

    // 2 bytes, domain length
    buf.put_u16_be(remote_dst.address().len() as u16);

    // n bytes, domain
    for &b in remote_dst.address() {
        buf.put_u8(b);
    }

    buf
}

// +----+----------+----------+
// |VER | NMETHODS | METHODS  |
// +----+----------+----------+
// | 1  |    1     | 1 to 255 |
// +----+----------+----------+
async fn recv_method_selection_message(socket: &mut TcpStream) -> std::io::Result<usize> {
    let mut ver_buf = [0; 1];
    socket.read_exact(&mut ver_buf).await?;
    println!("VER: {}", ver_buf[0]);

    let mut nmethods_buf = [0; 1];
    socket.read_exact(&mut nmethods_buf).await?;
    println!("NMETHODS: {}", nmethods_buf[0]);

    let mut methods_buf = vec![0; nmethods_buf[0] as usize];
    socket.read_exact(&mut methods_buf).await
}

// +----+--------+
// |VER | METHOD |
// +----+--------+
// | 1  |   1    |
// +----+--------+
async fn send_method_selection_message(socket: &mut TcpStream) -> std::io::Result<()> {
    // X'00' NO AUTHENTICATION REQUIRED
    // X'01' GSSAPI
    // X'02' USERNAME/PASSWORD
    // X'03' to X'7F' IANA ASSIGNED
    // X'80' to X'FE' RESERVED FOR PRIVATE METHODS
    // X'FF' NO ACCEPTABLE METHODS
    let write_buf = [5, 0x00];
    socket.write_all(&write_buf).await
}

// +----+-----+-------+------+----------+----------+
// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

async fn recv_request(socket: &mut TcpStream) -> Destination {
    let mut buf = [0; 4];
    socket.read_exact(&mut buf).await.unwrap();

    println!("VER: {}, CMD: {}, ATYP: {}", buf[0], buf[1], buf[3]);

    let address_type = buf[3];

    if address_type == 3 {
        let mut len_buf = [0];
        socket.read_exact(&mut len_buf).await.unwrap();

        let address_len = len_buf[0];

        let mut address_buf = vec![0; address_len as usize];
        socket.read_exact(&mut address_buf).await.unwrap();


        let mut port_buf = [0; 2];
        socket.read_exact(&mut port_buf).await.unwrap();

        let port = (port_buf[0] as u16) * 256 + (port_buf[1] as u16);

        let domain = String::from_utf8_lossy(&address_buf);
        // 远程的域名和端口，域名需要解析成 IP 地址
        println!("DST.ADDR: '{}', DST.PORT: {:?}", domain, port);

        Destination::new(address_buf, port)
    } else {
        Destination::default()
    }
}

// +----+-----+-------+------+----------+----------+
// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

// X'00' succeeded
// X'01' general SOCKS server failure
// X'02' connection not allowed by ruleset
// X'03' Network unreachable
// X'04' Host unreachable
// X'05' Connection refused
// X'06' TTL expired
// X'07' Command not supported
// X'08' Address type not supported
// X'09' to X'FF' unassigned
async fn send_reply(socket: &mut TcpStream) -> std::io::Result<()> {
    let reply = [
        5,
        0,
        0,
        1,
        1, 1, 1, 1,
        0, 80,
    ];

    socket.write_all(&reply).await
}
