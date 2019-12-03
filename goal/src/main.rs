use bytes::{BigEndian, ByteOrder, BytesMut};
use tokio;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;
use std::net::SocketAddr;

mod resolve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = std::env::args().nth(1).unwrap_or("0.0.0.0:18030".to_string());
    let address: SocketAddr = addr.parse::<SocketAddr>()?;

    let mut listener = TcpListener::bind(&address)
        .await
        .expect("监听端口失败");

    println!("Listening on: {}, pid: {}", address, std::process::id());

    loop {
        let (mut soccer_socket, soccer_addr) = listener.accept().await?;
        println!("\nAccept a connection from {}", soccer_addr);

        tokio::spawn(async move {
            // 读协议头
            let mut port_buf = [0u8; 2];
            soccer_socket.read_exact(&mut port_buf).await.unwrap();

            let port = BigEndian::read_u16(&port_buf);
            println!("port: {}", port);


            let mut reserved_buf = [0u8; 2];
            soccer_socket.read_exact(&mut reserved_buf).await.unwrap();
            let reserved = BigEndian::read_u16(&reserved_buf);
            println!("reserved: {}", reserved);


            let mut len_buf = [0u8; 2];
            soccer_socket.read_exact(&mut len_buf).await.unwrap();
            let len = BigEndian::read_u16(&len_buf);
            println!("len: {}", len);


            let mut buf = BytesMut::with_capacity(len as usize);
            buf.resize(len as usize, 0);
            soccer_socket.read_exact(&mut buf).await.unwrap();

            let domain = String::from_utf8_lossy(&buf).to_string();
            println!("domain: {}", domain);

            // 解析目标域名和目标端口
            let dest_domain: String = domain;
            let dest_port = port;

            let dest_ip_addr = resolve::resolve_domain(&dest_domain).await.unwrap();

            let dest_addr = SocketAddr::new(dest_ip_addr, dest_port);
            println!("{}", dest_addr);

            let dest_stream = TcpStream::connect(dest_addr)
                .await
                .expect("Failed to connect to destination");

            transfer::transfer(soccer_socket, dest_stream).await;
            println!("Transfer finished");
        });
    }
}
