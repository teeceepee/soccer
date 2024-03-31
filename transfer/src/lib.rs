mod byte_shift;
mod decode;
mod encode;
pub mod tcp_accept;

use futures::{future, Sink, SinkExt, Stream, StreamExt};
use log::{debug, error};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error, Message};

type WsError = Error;

pub async fn ws_to_tcp<R, W>(mut ws_read: R, mut tcp_write: W)
where
    R: Stream<Item=Result<Message, WsError>> + Unpin,
    W: AsyncWrite + Unpin,
{
    while let Some(msg_ret) = StreamExt::next(&mut ws_read).await {
        match msg_ret {
            Ok(msg) => {
                match msg {
                    Message::Text(cmd) => {
                        if cmd == "half_close" {
                            debug!("Received half_close command from ws");
                            break
                        }
                    }
                    Message::Binary(payload) => {
                        AsyncWriteExt::write_all(&mut tcp_write, &payload).await.unwrap();
                    }
                    Message::Close(_) => {
                        debug!("Closed by CLOSE message");
                        break
                    }
                    unknown_msg => {
                        debug!("Closed by UNKNOWN message, {:?}", unknown_msg);
                        break
                    }
                }
            }
            Err(e) => {
                // Protocol(ResetWithoutClosingHandshake)
                error!("Failed to receive WebSocket message, err: {:?}", e);
                break
            }
        }

        println!("Waiting next ws message...");
    }

    // ws 端已无数据，不会再向 `tcp_write` 写入数据，因此关闭 `tcp_write`，
    // 同时以此通知 TCP 连接的另一端。
    let _ = tcp_write.shutdown().await;
    println!("ws_to_tcp finished");
}

pub async fn tcp_to_ws<R, W>(mut tcp_read: R, mut ws_write: W)
where
    R: AsyncRead + Unpin,
    W: Sink<Message, Error=WsError> + Unpin,
{
    let mut buf = vec![0; 100];
    loop {
        match AsyncReadExt::read(&mut tcp_read, &mut buf).await {
            Ok(0) => {
                // when read() returns Ok(0), this signifies that the stream is closed.
                // Any further calls to read() will complete immediately with Ok(0).
                // With TcpStream instances, this signifies that the read half of the socket is closed.
                debug!("tcp read close: Ok(0)");
                break
            }
            Ok(n) => {
                debug!("tcp read n: {}", n);
                let msg = Message::binary(&buf[0..n]);
                SinkExt::send(&mut ws_write, msg).await.unwrap();
            }
            Err(e) => {
                // 连接到 baidu.com 可能会出现连接重置错误：
                // Connection reset by peer (os error 54)
                debug!("tcp read err: {}", e);
                break
            }
        }
    }

    let half_close_msg = Message::text("half_close");
    match SinkExt::send(&mut ws_write, half_close_msg).await {
        Ok(_) => {},
        Err(e) => {
            // 可能的错误：
            // AlreadyClosed
            // Os { code: 32, kind: BrokenPipe, message: "Broken pipe" }
            error!("Failed to send 'half_close' message, err: {:?}", e);
        }
    }

    println!("tcp_to_ws finished");
}

pub const SUGAR: u8 = 251;

// client <=> goal
pub async fn bridge_soccer_goal(mut left: TcpStream, mut right: TcpStream, sugar: u8) {
    let (mut left_read_half, mut left_write_half) = left.split();
    let (mut right_read_half, mut right_write_half) = right.split();
    let left_to_right = encode::encode(&mut left_read_half, &mut right_write_half, sugar);
    let right_to_left = decode::decode(&mut right_read_half, &mut left_write_half, sugar);

    future::try_join(left_to_right, right_to_left)
        .await
        .expect("try_join 出错");
}

// soccer <=> dest
pub async fn bridge_goal_dest(mut left: TcpStream, mut right: TcpStream, sugar: u8) {
    let (mut left_read_half, mut left_write_half) = left.split();
    let (mut right_read_half, mut right_write_half) = right.split();
    let left_to_right = decode::decode(&mut left_read_half, &mut right_write_half, sugar);
    let right_to_left = encode::encode(&mut right_read_half, &mut left_write_half, sugar);

    future::try_join(left_to_right, right_to_left)
        .await
        .expect("try_join 出错");
}


//pub async fn transfer(mut left: TcpStream, mut right: TcpStream) {
//    let (mut left_read_half, mut left_write_half) = left.split();
//    let (mut right_read_half, mut right_write_half) = right.split();
//    let left_to_right = tokio::io::copy(&mut left_read_half, &mut right_write_half);
//    let right_to_left = tokio::io::copy(&mut right_read_half, &mut left_write_half);
//
//    future::try_join(left_to_right, right_to_left)
//        .await
//        .expect("try_join 出错");
//    //futures::future::try_select(left_to_right, right_to_left).await;
//}
