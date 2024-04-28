use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use bytes::BytesMut;
use tokio::net::UdpSocket;

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.2.1
//
// Messages carried by UDP are restricted to 512 bytes (not counting the IP
// or UDP headers).
const MAX_RESPONSE_SIZE: usize = 512;

pub async fn resolve_domain(server_addr: SocketAddr, domain: &str) -> std::io::Result<Option<IpAddr>> {
    tracing::debug!("resolving domain: {}", domain);

    let request_bytes = dns::encode_request(domain).unwrap();

    let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    let sock = UdpSocket::bind(local_addr).await?;

    let _send_size = sock.send_to(&request_bytes, server_addr).await?;

    let mut resp_buf = BytesMut::with_capacity(MAX_RESPONSE_SIZE);
    let response_size = sock.recv_buf(&mut resp_buf).await?;
    let response_bytes = &resp_buf[0..response_size];
    tracing::debug!("received udp response, length: {}, {:?}", response_size, response_bytes);

    let resp = dns::decode_response(response_bytes)?;

    match resp.last_address() {
        None => {
            tracing::debug!("received udp response has no answers");
            Ok(None)
        }
        Some(addr) => {
            tracing::debug!("received udp response has {} answers", resp.addresses().len());
            let ip_addr = std::net::IpAddr::from(std::net::Ipv4Addr::from(addr));
            Ok(Some(ip_addr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve() {
        let server_addr = "114.114.114.114:53".parse().unwrap();
        let ret1 = resolve_domain(server_addr,"z.cn").await;
        assert!(ret1.unwrap().is_some());

        let ret2 = resolve_domain(server_addr,"qwertyuiop.cn").await;
        assert!(ret2.unwrap().is_none());
    }
}
