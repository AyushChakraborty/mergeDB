use communication::replication_service_client::ReplicationServiceClient;
use communication::SetvalueMessage;
use tonic::Request;

pub mod communication {
    tonic::include_proto!("communication");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ReplicationServiceClient::connect("http://127.0.0.1:8080").await?;

    println!("Connected to the server!");

    let key = "test_key".to_string();
    let value = serde_json::json!({
        "name": "mergeDB",
        "status": "Running",
    });

    let value_bytes = serde_json::to_vec(&value)?;

    let request = Request::new(SetvalueMessage {
        key: key.clone(),
        value: value_bytes,
    });

    let response = client.propagate_data(request).await?;

    println!("Response: {:?}", response.into_inner());

    Ok(())
}
