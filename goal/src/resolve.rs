use tokio::net::UdpSocket;

pub async fn resolve_domain(domain: &str) -> std::io::Result<std::net::IpAddr> {
    let request_bytes = dns::encode_request(domain).unwrap();

    let local_addr = "0.0.0.0:0";
    // let remote_addr = "1.1.1.1:53";
    let remote_addr = "114.114.114.114:53";
    let sock = UdpSocket::bind(local_addr).await?;

    let _send_size = sock.send_to(&request_bytes, remote_addr).await?;

    let mut resp_buf = [0u8; 1000];
    let response_size = sock.recv(&mut resp_buf).await?;

    let response_bytes = &resp_buf[0..response_size];

    let resp = dns::decode_response(response_bytes)?;

    let last_addr = resp.last_address().unwrap();

    let ip_addr = std::net::IpAddr::from(std::net::Ipv4Addr::from(last_addr));

    Ok(ip_addr)
}
