use futures::future;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpStream};

pub async fn transfer(mut left: TcpStream, mut right: TcpStream) {
    let (mut left_read_half, mut left_write_half) = left.split();
    let (mut right_read_half, mut right_write_half) = right.split();
    let left_to_right = left_read_half.copy(&mut right_write_half);
    let right_to_left = right_read_half.copy(&mut left_write_half);

    future::try_join(left_to_right, right_to_left)
        .await
        .expect("try_join 出错");
    //futures::future::try_select(left_to_right, right_to_left).await;
}

//async fn _proxy_manually(mut client: TcpStream, mut remote: TcpStream) {
//    let mut buf = [0; 1024];
//    let n = client
//        .read(&mut buf)
//        .await
//        .expect("failed to read data from remote");
//
//    let request_bytes = &buf[0..n];
//    let req = String::from_utf8_lossy(request_bytes);
//
//    let req_s = req.replace("\r\n", "<CR><LF>\r\n");
//    println!("\n{}", req_s);
//
//    remote
//        .write_all(request_bytes)
//        .await
//        .expect("failed to write data to remote");
//
//    loop {
//        let read_len = remote
//            .read(&mut buf)
//            .await
//            .expect("失败：failed to read data from remote");
//
//        println!("长度：{}", read_len);
//
//        if read_len == 0 {
//            break;
//        }
//
//        let response_bytes = &buf[0..read_len];
//        let resp = String::from_utf8_lossy(response_bytes);
//        let resp_s = resp.replace("\r\n", "<CR><LF>\r\n");
//        println!("接收到的：\n{}", resp_s);
//        println!("====");
//
//        client
//            .write_all(response_bytes)
//            .await
//            .expect("failed to write data to remote");
//    }
//}
