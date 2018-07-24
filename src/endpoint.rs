use addrport::AddrPort;
use ipnet::Ipv4Net;
use std::net::Ipv4Addr;
use std::io::Write;
use std::process::{Command, Stdio};


fn gen_keys() -> (String, String) {
    let output = Command::new("wg")
        .args(&["genkey"])
        .output()
        .expect("Failed to execute wg genkey");

    let privkey =
        String::from_utf8(output.stdout).unwrap()
            .trim()
            .trim_left()
            .to_string();

    let mut command = Command::new("wg")
        .args(&["pubkey"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn wg pubkey");

    command.stdin
        .as_mut()
        .expect("Failed to get stdin for wg pubkey")
        .write_all(privkey.as_bytes())
        .expect("Failed to write privkey to wg pubkey");

    let output =
        command.wait_with_output()
            .expect("Failed to get output for wg pubkey");

    let pubkey =
        String::from_utf8(output.stdout).unwrap()
            .trim()
            .trim_left()
            .to_string();

    (privkey, pubkey)
}



#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndPoint {
    name: String,
    private_key: String,
    public_key: String,
    external_address: Option<AddrPort>,
    internal_address: Ipv4Addr,
    allowed_ips: Vec<Ipv4Net>,
    persistent_keepalive: Option<usize>
}


impl EndPoint {
    pub fn new<S: Into<String>>(name: S, internal_address: Ipv4Addr)
        -> EndPoint {

        let (private_key, public_key) = gen_keys();
        EndPoint {
            name: name.into(),
            private_key: private_key,
            public_key: public_key,
            external_address: None,
            internal_address: internal_address,
            allowed_ips: Vec::new(),
            persistent_keepalive: None
        }
    }

    pub fn builder_external_address(
        mut self,
        external_address: Option<AddrPort>
    ) -> EndPoint {
        self.external_address = external_address;
        self
    }

    pub fn builder_push_allowed_ips(
        mut self,
        allowed_ip: Ipv4Net
    ) -> EndPoint {
        self.allowed_ips.push(allowed_ip);
        self
    }

    pub fn builder_persistent_keepalive(mut self, keepalive: Option<usize>)
        -> EndPoint {
        self.persistent_keepalive = keepalive;
        self
    }

    pub fn set_external_address(&mut self, external_address: Option<AddrPort>) {

        self.external_address = external_address;
    }

    pub fn set_internal_address(&mut self, internal_address: Ipv4Addr) {

        self.internal_address = internal_address;
    }

    pub fn push_allowed_ip(&mut self, allowed_ip: Ipv4Net) {
        self.allowed_ips.push(allowed_ip);
    }

    pub fn set_persistent_keepalive(&mut self, keepalive: Option<usize>) {
        self.persistent_keepalive = keepalive;
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn private_key(&self) -> &str { &self.private_key }
    pub fn public_key(&self) -> &str { &self.public_key }
    pub fn external_address(&self) -> Option<&AddrPort> {
        self.external_address.as_ref()
    }
    pub fn internal_address(&self) -> &Ipv4Addr { &self.internal_address }
    pub fn allowed_ips(&self) -> Vec<Ipv4Net> {
        if !self.allowed_ips.is_empty() {
            self.allowed_ips.clone()
        }
        else {
            vec![Ipv4Net::new(self.internal_address().clone(), 32)
                .expect("Failed to make Ipv4Net for allowed_ips()")]
        }
    }
    pub fn persistent_keepalive(&self) -> Option<usize> {
        self.persistent_keepalive.clone()
    }

    pub fn interface(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("[Interface]".to_string());
        lines.push(format!("# name: {}", self.name()));
        lines.push(format!("PrivateKey = {}", self.private_key()));
        if let Some(external_address) = self.external_address() {
            lines.push(format!("ListenPort = {}", external_address.port()));
        }
        lines.join("\n")
    }

    pub fn peer(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("[Peer]".to_string());
        lines.push(format!("# {}", self.name()));
        lines.push(format!("PublicKey = {}", self.public_key()));
        if let Some(external_address) = self.external_address() {
            lines.push(format!("Endpoint = {}", external_address));
        }
        
        lines.push(format!("AllowedIPs = {}",
            self.allowed_ips()
                .iter()
                .map(|ip| format!("{}", ip))
                .collect::<Vec<String>>()
                .join(", ")));
    
        lines.join("\n")
    }
}
