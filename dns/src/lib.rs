// https://www.reddit.com/r/rust/comments/381y9g/why_udpsocket_in_rust_must_be_created_using_the/

mod header;
mod question;
mod request_message;
mod resource_record;
mod response_message;
mod utils;

pub use response_message::ResponseMessage;

pub fn encode_request(domain: &str) -> std::io::Result<Vec<u8>> {
    let request_msg = request_message::RequestMessage::new(domain);
    let mut request_bytes: Vec<u8> = vec![];
    request_msg.to_bytes(&mut request_bytes)?;

    Ok(request_bytes)
}

pub fn decode_response(response_bytes: &[u8]) -> std::io::Result<ResponseMessage> {
    ResponseMessage::parse_response(response_bytes)
}
