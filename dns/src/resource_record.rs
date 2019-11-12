use crate::utils::qname_to_domain;
use byteorder::{ReadBytesExt, BE};
use std::io::{BufRead, Cursor};

#[derive(Debug)]
pub struct ResourceRecord {
    domain: String,
    rtype: u16,
    rclass: u16,
    ttl: u32,
    rdlength: u16,
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    pub fn parse_from_reader(rdr: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        // [192, 12]
        let first_byte = rdr.read_u8()?;
        let second_byte = rdr.read_u8()?;

        // let mark = first_byte >> 6;
        let offset: u64 = u64::from(first_byte & 63) + u64::from(second_byte);

        let current_pos = rdr.position();

        rdr.set_position(offset);
        let mut name: Vec<u8> = Vec::new();
        read_domain_name(&mut name, rdr)?;
        rdr.set_position(current_pos);

        let domain = qname_to_domain(&name);
        let rtype = rdr.read_u16::<BE>()?;
        let rclass = rdr.read_u16::<BE>()?;
        let ttl = rdr.read_u32::<BE>()?;
        let rdlength = rdr.read_u16::<BE>()?;

        let mut rdata: Vec<u8> = Vec::with_capacity(rdlength as usize);
        for _ in 0..rdlength {
            rdata.push(rdr.read_u8()?);
        }

        let record = Self {
            domain,
            rtype,
            rclass,
            ttl,
            rdlength,
            rdata,
        };

        Ok(record)
    }
}

fn read_domain_name(domain_name: &mut Vec<u8>, rdr: &mut Cursor<&[u8]>) -> std::io::Result<()> {
    rdr.read_until(0, domain_name)?;

    Ok(())
}
