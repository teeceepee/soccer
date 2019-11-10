// https://www.ietf.org/rfc/rfc1928.txt

use futures::future::try_join;

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;
use std::net::SocketAddr;

use soccer::Destination;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = std::env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>()?;

    let mut listener = TcpListener::bind(&addr)
        .await
        .expect("监听失败");
    println!("Listening on: {}, pid: {}", addr, std::process::id());

    loop {
        let (mut client_socket, client_addr) = listener.accept().await?;
        println!("\nAccept a connection from {}", client_addr);

        tokio::spawn(async move {
            recv_method_selection_message(&mut client_socket).await.unwrap();
            send_method_selection_message(&mut client_socket).await.unwrap();

            let remote_dst = recv_request(&mut client_socket).await;
            println!("remote destination: {}", remote_dst.to_str());


            send_reply(&mut client_socket).await.unwrap();


            // baidu.com 220.181.38.148
            // qq.com 59.37.96.63
            // cn.bing.com 202.89.233.100
            let outbound_addr: SocketAddr = "220.181.38.148:80".parse().unwrap();
//            let outbound_addr: SocketAddr = "59.37.96.63:80".parse().unwrap();
//            let outbound_addr: SocketAddr = "202.89.233.100:80".parse().unwrap();
            let remote_stream = TcpStream::connect(&outbound_addr)
                .await
                .expect("failed to connect to remote");

            proxy(client_socket, remote_stream).await;
            println!("Proxy Finished");
        });
    }
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

async fn proxy(mut client: TcpStream, mut remote: TcpStream) {
    let (mut client_read_half, mut client_write_half) = client.split();
    let (mut remote_read_half, mut remote_write_half) = remote.split();
    let client_to_remote = client_read_half.copy(&mut remote_write_half);
    let remote_to_client = remote_read_half.copy(&mut client_write_half);

    try_join(client_to_remote, remote_to_client)
        .await
        .expect("try_join 出错");
    //futures::future::try_select(client_to_remote, remote_to_client).await;
}

async fn _proxy_manually(mut client: TcpStream, mut remote: TcpStream) {
    let mut buf = [0; 1024];
    let n = client
        .read(&mut buf)
        .await
        .expect("failed to read data from remote");

    let request_bytes = &buf[0..n];
    let req = String::from_utf8_lossy(request_bytes);

    let req_s = req.replace("\r\n", "<CR><LF>\r\n");
    println!("\n{}", req_s);

    remote
        .write_all(request_bytes)
        .await
        .expect("failed to write data to remote");

    loop {
        let read_len = remote
            .read(&mut buf)
            .await
            .expect("失败：failed to read data from remote");

        println!("长度：{}", read_len);

        if read_len == 0 {
            break;
        }

        let response_bytes = &buf[0..read_len];
        let resp = String::from_utf8_lossy(response_bytes);
        let resp_s = resp.replace("\r\n", "<CR><LF>\r\n");
        println!("接收到的：\n{}", resp_s);
        println!("====");

        client
            .write_all(response_bytes)
            .await
            .expect("failed to write data to remote");
    }
}
