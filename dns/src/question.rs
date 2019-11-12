use crate::utils::{domain_to_qname, qname_to_domain};
use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::io::{BufRead, Cursor};

#[derive(Debug)]
pub struct Question {
    domain: String,
    qtype: u16,
    qclass: u16,
}

impl Question {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            qtype: 1,
            qclass: 1,
        }
    }

    pub fn to_bytes(&self, bytes: &mut Vec<u8>) -> std::io::Result<()> {
        let qname = domain_to_qname(&self.domain);
        for b in qname {
            bytes.push(b);
        }

        bytes.write_u16::<BE>(self.qtype)?;
        bytes.write_u16::<BE>(self.qclass)?;

        Ok(())
    }

    pub fn parse_from_reader(rdr: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let mut qname: Vec<u8> = Vec::new();
        rdr.read_until(0, &mut qname)?;
        let domain = qname_to_domain(&qname);

        let qtype = rdr.read_u16::<BE>()?;
        let qclass = rdr.read_u16::<BE>()?;

        let q = Self {
            domain,
            qtype,
            qclass,
        };

        Ok(q)
    }
}
