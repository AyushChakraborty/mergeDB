use dashmap::DashMap;
use kv_types::Merge;
use kv_types::{aw_set::AWSet, pn_counter::PNCounter};
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::{net::SocketAddr, sync::{Arc, RwLock}};
use tonic::{transport::{Server}, Request, Response};

use crate::communication::{GossipChangesRequest, GossipChangesResponse, PnCounterMessage, replication_service_client::ReplicationServiceClient};
use crate::{
    communication::{
        replication_service_server::ReplicationService,
        replication_service_server::ReplicationServiceServer, PropagateDataRequest,
        PropagateDataResponse,
    },
    config::Config,
};

const K: usize = 3;

#[derive(Debug)]
pub enum CRDTValue {
    Counter(PNCounter), //others later
    ASet(AWSet<String>),
}

#[derive(Debug, Clone)]
pub struct ReplicationServer {
    pub store: Arc<DashMap<String, CRDTValue>>,
    pub node_id: String,
    pub peers: Arc<RwLock<Vec<String>>>,
}

// convert domain -> proto for sending
impl From<PNCounter> for PnCounterMessage {
    fn from(domain: PNCounter) -> Self {
        Self {
            p: domain.p,
            n: domain.n,
        }
    }
}

// convert proto -> domain for receiving
impl From<PnCounterMessage> for PNCounter {
    fn from(wire: PnCounterMessage) -> Self {
        Self {
            p: wire.p,
            n: wire.n,
        }
    }
}

#[tonic::async_trait]
impl ReplicationService for ReplicationServer {
    async fn propagate_data(
        &self,
        request: tonic::Request<PropagateDataRequest>,
    ) -> Result<tonic::Response<PropagateDataResponse>, tonic::Status> {
        let req_inner = request.into_inner();

        let value_type = req_inner.valuetype;
        let key = req_inner.key;
        let raw_value_bytes = req_inner.value;
        let map = Arc::clone(&self.store);

        if value_type == "CSET" {
            //value shld be a u64
            let bytes: [u8; 8] = raw_value_bytes.try_into().map_err(|_| {
                tonic::Status::invalid_argument("invalid byte length for u64, expected 8 bytes")
            })?;

            let numeric_val: u64 = u64::from_be_bytes(bytes);

            println!("received valid CSET: {}", numeric_val);

            let new_pn: CRDTValue = CRDTValue::Counter(PNCounter {
                p: HashMap::from([(self.node_id.clone(), numeric_val)]),
                n: HashMap::from([(self.node_id.clone(), 0)]),
            });
            map.insert(key, new_pn);
        } else {
            println!("other types soon!");
        }

        Ok(Response::new(PropagateDataResponse { success: true }))
    }

    async fn gossip_changes(
        &self,
        changes: tonic::Request<GossipChangesRequest>,
    ) -> Result<tonic::Response<GossipChangesResponse>, tonic::Status> {
        let changes_inner = changes.into_inner();
        let key = changes_inner.key;
        let counter = changes_inner.counter.unwrap();
        let remote_counter = PNCounter::from(counter); //the actual PNCounter type

        //call merge now with the value corresponding to the same key in this node
        self.store
            .entry(key)
            .and_modify(|current_value| {
                match current_value {
                    CRDTValue::Counter(local_counter) => {
                        local_counter.merge(&mut remote_counter.clone());
                        println!("merged from remote node");
                    } //other types later
                    _ => println!("type mismatch: key exisits, but value is not of type PNCounter"),
                }
            })
            .or_insert_with(|| CRDTValue::Counter(remote_counter));

        Ok(Response::new(GossipChangesResponse { success: true }))
    }
}

impl ReplicationServer {
    pub async fn start_listener(&self, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = config.listen_address.as_str().parse()?;
        Server::builder()
            .add_service(ReplicationServiceServer::new(self.clone()))
            .serve(addr)
            .await?;

        Ok(())
    }

    pub async fn push(&self, key: String, value: CRDTValue) -> Result<(), Box<dyn std::error::Error>> {
        //send updates to k randomly chosen peers
        //first make sure to preconnect to 3 randomly chosen peer nodes
        //lots of things to think of, like what if a node goes down, how will this node reconnect to
        //some other node etc, will tackle these later

        let mut rng = rand::rng();

        //a connection pool of rpc connections so as to not cause redundant ::connect's again if
        //a node has already been connected to in an earlier iteration
        // let mut connection_pool: HashMap<String, ReplicationServiceClient<Channel>> = HashMap::new();

        let chosen_peers: Vec<String> = {
            let peer_guard = self.peers.read().expect("RwLock poisoned");
            peer_guard
            .choose_multiple(&mut rng, K)
            .cloned()
            .collect()
        };

        for peer_addr in chosen_peers.iter() {
            match ReplicationServiceClient::connect((*peer_addr).clone()).await {
                Ok(mut peer_client) => {
                    match &value {
                        CRDTValue::Counter(inner) => {
                            let state = Request::new(GossipChangesRequest {
                                key: key.clone(),
                                counter: Some(PnCounterMessage::from(inner.clone())),
                            });
                            
                            println!("connected to the peer with id: {}", peer_addr);
                            match peer_client.gossip_changes(state).await {
                                Ok(response) => println!("Response from peer: {:?}", response.into_inner()),
                                Err(e) => println!("failed to send update to {}: {}", peer_addr, e),
                            }
                        }
                        _ => print!("other types soon!"),
                    }
                }
                Err(e) => {
                    println!("failed to connect to {}: {}", peer_addr, e);
                    continue;
                }
            }
        }

        Ok(())
    }
}
