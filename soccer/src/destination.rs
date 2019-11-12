pub struct Destination {
    address: Vec<u8>,
    port: u16,
}

impl Destination {
    pub fn new(address: Vec<u8>, port: u16) -> Self {
        Self {
            address,
            port,
        }
    }

    pub fn domain(&self) -> String {
        String::from_utf8_lossy(&self.address).to_string()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn to_str(&self) -> String {
        let domain = String::from_utf8_lossy(&self.address).to_owned();

        format!("{}:{}", domain, self.port)
    }
}

impl Default for Destination {
    fn default() -> Self {
        Self {
            address: vec![],
            port: 0,
        }
    }
}
