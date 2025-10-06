use std::{fs::File, io::{self, Read, Write}, path::PathBuf};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    node_id: String,
    listen_address: String,
    peers: Vec<String>,
}

impl Config {
    pub fn load_config(config_path: PathBuf) -> io::Result<Self> {
        let mut file = File::open(&config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let new_config: Self = toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(new_config)
    }

    pub fn store_config(node: &Self, config_path: PathBuf) -> io::Result<()> {
        let mut file = File::create(&config_path)?;

        let contents = toml::to_string(node)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(contents.as_bytes())?;

        Ok(())
    }
}