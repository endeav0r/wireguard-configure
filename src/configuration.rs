use endpoint::EndPoint;
use serde_yaml;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    router: EndPoint,
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

    pub fn new(router: EndPoint) -> Configuration {
        Configuration {
            router: router,
            clients: Vec::new()
        }
    }

    pub fn push_client(&mut self, client: EndPoint) {
        self.clients.push(client);
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

    pub fn router(&self) -> &EndPoint { &self.router }
    pub fn clients(&self) -> &[EndPoint] { &self.clients }

    pub fn client_by_name(&self, name: &str) -> Option<&EndPoint> {
        self.clients
            .iter()
            .find(|client| client.name() == name)
    }
}