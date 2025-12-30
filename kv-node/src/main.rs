use dashmap::DashMap;
use kv_node::{
    config::Config,
    network::{ReplicationServer},
};
use std::{path::Path, sync::{Arc, RwLock}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "../config.toml".to_string();
    let file_path = Path::new(&path);
    let config = match Config::load_config(file_path.to_path_buf()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return Ok(());
        }
    };

    let map = Arc::new(DashMap::new());
    let peers = Arc::new(RwLock::new(config.peers.clone()));
    let node_id = config.node_id.clone();
    // let node_addr = config.listen_address.clone();
    let server = ReplicationServer {
        store: map.clone(),
        node_id: node_id,
        peers: peers,
    };

    server.start_listener(config).await?;
    Ok(())
}
