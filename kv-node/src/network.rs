use dashmap::DashMap;
use tonic::{Response, transport::Server};
use std::{net::SocketAddr, sync::Arc};

use crate::{communication::{replication_service_server::ReplicationService, SetvalueMessage, replication_service_server::ReplicationServiceServer, ResponseMessage}, config::Config};

#[derive(Debug, Clone)]
pub struct ReplicationServer {
    pub change: Arc<DashMap<String, String>>,
}

#[tonic::async_trait]
impl ReplicationService for ReplicationServer {
    async fn propagate_data(
        &self,
        request: tonic::Request<SetvalueMessage>,
    ) -> Result<tonic::Response<ResponseMessage>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let value = String::from_utf8(req.value)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid value: {}", e)))?;

        let map = Arc::clone(&self.change);
        map.insert(key, value);

        Ok(Response::new(ResponseMessage { success: true }))
    }
}

impl ReplicationServer {
    pub async fn start_listener(&self, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = config.listen_address.as_str().parse()?;
        Server::builder().add_service(ReplicationServiceServer::new(self.clone())).serve(addr).await?;

        Ok(())
    }
}