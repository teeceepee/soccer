extern crate tokio;

use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let soccer_address = "127.0.0.1:8080".to_string();
    let addr = soccer_address.parse::<SocketAddr>()?;
    // 1. connects to the server
    let mut soccer_stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to soccer");
    // 2. sends a version identifier/method selection message
    let method_selection_message: [u8; 3] = [0x05, 0x01, 0x00];
    soccer_stream.write_all(&method_selection_message).await?;

    // 3. receives a METHOD selection message
    let mut method_buf = [0; 2];
    soccer_stream.read_exact(&mut method_buf).await?;
    println!("method_buf: {:?}", method_buf);

    // 4. sends the request details
    let request = [
        0x05, 0x01, 0x00, 0x03,
        0x4, 122, 46, 99, 110, // "z.cn"
        0, 80,
    ];
    soccer_stream.write_all(&request).await?;

    // 5. receives request's reply
    let mut reply_buf: [u8; 10] = [0; 10];
    soccer_stream.read_exact(&mut reply_buf).await?;
    println!("reply_buf:  {:?}", reply_buf);

    let http_req = [
        "GET / HTTP/1.1\r\n",
        "Host: z.cn\r\n",
        "User-Agent: curl/7.88.1\r\n",
        "Accept: */*\r\n",
        "\r\n",
    ];
    let r = http_req.concat();
    soccer_stream.write_all(r.as_bytes()).await?;

    // TODO receive response

    Ok(())
}
