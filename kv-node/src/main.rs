use std::{path::Path, sync::Arc};
use dashmap::DashMap;
use kv_node::{config::Config, network::ReplicationServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "config.toml".to_string();
    let file_path = Path::new(&path);
    let config = match Config::load_config(file_path.to_path_buf()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return Ok(());
        }
    };

    let map = Arc::new(DashMap::new());
    let server = ReplicationServer {
        change: map,
    };

    server.start_listener(config).await?;

    Ok(())
}
