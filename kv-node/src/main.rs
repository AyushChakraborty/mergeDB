use std::{path::Path};
use kv_node::config::Config;

fn main() {
    let path = "config.toml".to_string();
    let file_path = Path::new(&path);
    match Config::load_config(file_path.to_path_buf()) {
        Ok(c) => {
        println!("{:?}", c);
        },
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
