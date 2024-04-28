use crate::header::Header;
use crate::question::Question;
use crate::resource_record::ResourceRecord;
use std::io::Cursor;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ResponseMessage {
    header: Header,
    question: Option<Question>,
    answer_records: Vec<ResourceRecord>,
}

impl ResponseMessage {
    // 解析 DNS 的响应
    pub fn parse_response(response: &[u8]) -> std::io::Result<Self> {
        // println!("resp size: {}, response: {:?}", response.len(), response);
        let mut reader = Cursor::new(response);

        let header = Header::parse_from_reader(&mut reader)?;
        // println!("header: {:?}", header);

        let question = if header.qd_count() == 1 {
            Some(Question::parse_from_reader(&mut reader)?)
        } else {
            None
        };

        let count = header.answer_count() as usize;
        let mut answer_records: Vec<ResourceRecord> = Vec::with_capacity(count);
        for _ in 0..header.answer_count() {
            let record = ResourceRecord::parse_from_reader(&mut reader)?;
            answer_records.push(record);
        }

        let msg = Self {
            header,
            question,
            answer_records,
        };

        Ok(msg)
    }

    pub fn first_address(&self) -> Option<[u8; 4]> {
        let r = self.answer_records.first()?;

        let addr = [
            *r.rdata.get(0)?,
            *r.rdata.get(1)?,
            *r.rdata.get(2)?,
            *r.rdata.get(3)?,
        ];

        Some(addr)
    }

    pub fn last_address(&self) -> Option<[u8; 4]> {
        let r = self.answer_records.last()?;

        let addr = [
            *r.rdata.get(0)?,
            *r.rdata.get(1)?,
            *r.rdata.get(2)?,
            *r.rdata.get(3)?,
        ];

        Some(addr)
    }


    pub fn addresses(&self) -> Vec<Option<[u8; 4]>> {
        self.answer_records.iter().map(|r| {
            let addr = [
                *r.rdata.get(0)?,
                *r.rdata.get(1)?,
                *r.rdata.get(2)?,
                *r.rdata.get(3)?,
            ];

            Some(addr)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_answers() {
        // response of "z.cnn"
        let resp = &[
            209, 183, 129, 131, 0, 1, 0, 0, 0, 1, 0, 0, // header
            1, 122, 3, 99, 110, 110, 0, 0, 1, 0, 1, 0, 0, 6, 0, 1, 0, 0, 2, 58, 0, 64, 1, 97, 12, 114, 111, 111, 116, 45, 115, 101, 114, 118, 101, 114, 115, 3, 110, 101, 116, 0, 5, 110, 115, 116, 108, 100, 12, 118, 101, 114, 105, 115, 105, 103, 110, 45, 103, 114, 115, 3, 99, 111, 109, 0, 120, 164, 113, 48, 0, 0, 7, 8, 0, 0, 3, 132, 0, 9, 58, 128, 0, 1, 81, 128];

        let resp = ResponseMessage::parse_response(resp).unwrap();
        assert_eq!(None, resp.first_address());
    }
}
