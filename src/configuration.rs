use endpoint::{EndPoint, Router};
use serde_yaml;
use std::fs::File;
use std::io::{Read, Write};
use ipnet::Ipv4Net;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    master_subnet: Option<Ipv4Net>,
    router: Router,
    clients: Vec<EndPoint>
}


impl Configuration {
    pub fn open(path: &Path) -> Configuration {
        let mut file = File::open(path)
            .expect(&format!("Failed to open {:?}", path));

        let mut buffer: String = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to read configuration file");

        match serde_yaml::from_str(&buffer) {
            Ok(configuration) => configuration,
            Err(e) => {
                eprintln!("Failed to parse configuration");
                panic!("{}", e);
            }
        }
    }

    pub fn save(&self, path: &Path) {
        let mut file = File::create(path)
            .expect(&format!("Failed to open {:?}", path));

        let bytes = serde_yaml::to_string(&self)
            .expect("Failed to serialize configuration");

        file.write_all(bytes.as_bytes())
            .expect("Failed to write configuration file");
    }

    pub fn new(router: Router) -> Configuration {
        Configuration {
            master_subnet: None,
            router: router,
            clients: Vec::new()
        }
    }

    pub fn push_client(&mut self, client: EndPoint) {
        self.clients.push(client);
    }

    pub fn set_master_subnet(&mut self, master_subnet: Option<Ipv4Net>) {
        self.master_subnet = master_subnet;
    }

    pub fn remove_client_by_name(&mut self, name: &str) -> bool {
        for i in 0..self.clients.len() {
            if self.clients[i].name() == name {
                self.clients.remove(i);
                return true;
            }
        }
        false
    }

    pub fn master_subnet(&self) -> Option<&Ipv4Net> {
        self.master_subnet.as_ref() 
    }
    pub fn router(&self) -> &Router { &self.router }
    pub fn clients(&self) -> &[EndPoint] { &self.clients }

    pub fn client_by_name(&self, name: &str) -> Option<&EndPoint> {
        self.clients
            .iter()
            .find(|client| client.name() == name)
    }

    pub fn all_allowed_ips(&self) -> Vec<Ipv4Net> {
        match self.master_subnet() {
            Some(master_subnet) => vec![master_subnet.clone()],
            None =>
                self.clients()
                    .iter()
                    .flat_map(|client| client.allowed_ips())
                    .collect::<Vec<Ipv4Net>>()
        }
    }

    pub fn client_config(&self, name: &str) -> Option<String> {
        let client = self.client_by_name(name)?;

        Some(format!("{}\n\n{}",
            client.interface(),
            self.router.peer(client, &self.all_allowed_ips())))
    }
}