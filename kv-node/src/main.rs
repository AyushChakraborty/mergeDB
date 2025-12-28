use dashmap::DashMap;
use kv_node::communication::PnCounterMessage;
use kv_node::{
    communication::{replication_service_client::ReplicationServiceClient, GossipChangesRequest},
    config::Config,
    network::{CRDTValue, ReplicationServer},
};
use rand::seq::IndexedRandom;
use std::{collections::HashMap, path::Path, sync::Arc};
use tonic::{transport::Channel, Request};

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
    let peers = config.peers.clone();
    let node_id = config.node_id.clone();
    // let node_addr = config.listen_address.clone();
    let server = ReplicationServer {
        store: map.clone(),
        node_id: node_id,
    };

    server.start_listener(config).await?;

    //once a while, propagate the changes to other nodes
    //first make sure to preconnect to 3 randomly chosen peer nodes
    //lots of things to think of, like what if a node goes down, how will this node reconnect to
    //some other node etc, will tackle these later

    let mut rng = rand::rng();

    //a connection pool of rpc connections so as to not cause redundant ::connect's again if
    //a node has already been connected to in an earlier iteration
    let mut connection_pool: HashMap<String, ReplicationServiceClient<Channel>> = HashMap::new();

    loop {
        let chosen_peers: Vec<&String> = peers.choose_multiple(&mut rng, 3).collect(); //chosen without replacement

        for peer_addr in chosen_peers.iter() {
            if !connection_pool.contains_key(*peer_addr) {
                match ReplicationServiceClient::connect((*peer_addr).clone()).await {
                    Ok(client) => {
                        connection_pool.insert((*peer_addr).clone(), client);
                    }
                    Err(e) => {
                        println!("failed to connect to {}: {}", peer_addr, e);
                        continue;
                    }
                }
            }

            //let mut peer = ReplicationServiceClient::connect((*peer_addr).clone()).await?;

            //for each key in the current node, transfer each of the node states for merge
            if let Some(peer_client) = connection_pool.get_mut(*peer_addr) {
                for key_val in map.iter() {
                    let key = key_val.key();
                    let value = key_val.value();

                    match value {
                        CRDTValue::Counter(inner) => {
                            let state = Request::new(GossipChangesRequest {
                                key: key.clone(),
                                counter: Some(PnCounterMessage::from(inner.clone())),
                            });
                            
                            println!("connected to the peer with id: {}", peer_addr);
                            let response = peer_client.gossip_changes(state).await?;
                            println!("Response from peer: {:?}", response.into_inner());
                        }
                        _ => print!("other types soon!"),
                    }
                }
            }
        }
        //wait for 2s before the next gossip round
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    Ok(())
}
