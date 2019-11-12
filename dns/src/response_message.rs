use crate::header::Header;
use crate::question::Question;
use crate::resource_record::ResourceRecord;
use std::io::Cursor;

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
