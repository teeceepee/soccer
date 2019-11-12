use crate::header::Header;
use crate::question::Question;

pub struct RequestMessage {
    header: Header,
    question: Question,
}

impl RequestMessage {
    pub fn new(domain: &str) -> Self {
        let header = Header::new();
        let question = Question::new(domain);

        Self { header, question }
    }

    pub fn to_bytes(&self, bytes: &mut Vec<u8>) -> std::io::Result<()> {
        self.header.to_bytes(bytes)?;
        self.question.to_bytes(bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_bytes_test() {
        let mut bytes: Vec<u8> = vec![];

        let msg = RequestMessage::new("baidu.com");
        msg.to_bytes(&mut bytes).unwrap();

        let expected = [
            209, 183, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, // header
            5, 98, 97, 105, 100, 117, 3, 99, 111, 109, 0, 0, 1, 0, 1, // question
        ];

        for (i, b) in bytes.iter().enumerate() {
            assert_eq!(*b, expected[i]);
        }
    }
}
