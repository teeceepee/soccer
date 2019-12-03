mod byte_shift;
mod decode;
mod encode;

use futures::future;
use tokio::net::TcpStream;

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
