use std::fmt;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddrPort {
    address: String,
    port: u16
}

impl AddrPort {
    pub fn new<A: Into<String>>(address: A, port: u16) -> AddrPort {
        AddrPort {
            address: address.into(),
            port: port
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}


impl fmt::Display for AddrPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.address(), self.port())
    }
}