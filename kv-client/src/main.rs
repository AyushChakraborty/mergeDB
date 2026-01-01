use communication::replication_service_client::ReplicationServiceClient;
use communication::PropagateDataRequest;
use tonic::Request;

pub mod communication {
    tonic::include_proto!("communication");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ReplicationServiceClient::connect("http://127.0.0.1:8000").await?;

    println!("Connected to the server!");

    //assuming the case: CSET test_key 1   is sent to the node
    let value_type = String::from("CSET");
    let key = "test_key".to_string();
    let value: u64 = 1;

    let request = Request::new(PropagateDataRequest {
        valuetype: value_type.clone(),
        key: key.clone(),
        value: value.to_be_bytes().to_vec(),
    });

    let response = client.propagate_data(request).await?;

    println!("Response: {:?}", response.into_inner());

    Ok(())
}
