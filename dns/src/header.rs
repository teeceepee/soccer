use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::io::Cursor;

// 12 bytes
#[derive(Debug)]
pub struct Header {
    id: u16, // 随机数字
    unparsed: u16,
    qdcount: u16, // question 数量，一般为 1
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl Header {
    pub fn new() -> Self {
        Self {
            id: 209 * 256 + 183,
            unparsed: 256, // rd = 1, rd 设置为 1 启用服务端的递归查询，只需一次请求即可返回地址
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    pub fn to_bytes(&self, bytes: &mut Vec<u8>) -> std::io::Result<()> {
        bytes.write_u16::<BE>(self.id)?;
        bytes.write_u16::<BE>(self.unparsed)?;
        bytes.write_u16::<BE>(self.qdcount)?;
        bytes.write_u16::<BE>(self.ancount)?;
        bytes.write_u16::<BE>(self.nscount)?;
        bytes.write_u16::<BE>(self.arcount)?;

        Ok(())
    }

    pub fn parse_from_reader(rdr: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let id = rdr.read_u16::<BE>()?;
        let unparsed = rdr.read_u16::<BE>()?;
        let qdcount = rdr.read_u16::<BE>()?;
        let ancount = rdr.read_u16::<BE>()?;
        let nscount = rdr.read_u16::<BE>()?;
        let arcount = rdr.read_u16::<BE>()?;

        let h = Self {
            id,
            unparsed,
            qdcount,
            ancount,
            nscount,
            arcount,
        };

        Ok(h)
    }

    pub fn qd_count(&self) -> u16 {
        self.qdcount
    }

    pub fn answer_count(&self) -> u16 {
        self.ancount
    }
}
