use dashmap::DashMap;
use kv_types::Merge;
use kv_types::{aw_set::AWSet, pn_counter::PNCounter};
use std::collections::HashMap;
use std::{net::SocketAddr, sync::Arc};
use tonic::{transport::Server, Response};

use crate::communication::{GossipChangesRequest, GossipChangesResponse, PnCounterMessage};
use crate::{
    communication::{
        replication_service_server::ReplicationService,
        replication_service_server::ReplicationServiceServer, PropagateDataRequest,
        PropagateDataResponse,
    },
    config::Config,
};

#[derive(Debug)]
pub enum CRDTValue {
    Counter(PNCounter), //others later
    ASet(AWSet<String>),
}

#[derive(Debug, Clone)]
pub struct ReplicationServer {
    pub store: Arc<DashMap<String, CRDTValue>>,
    pub node_id: String,
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
}
