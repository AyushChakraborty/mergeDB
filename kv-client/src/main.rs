use communication::replication_service_client::ReplicationServiceClient;
use communication::PropagateDataRequest;
use std::io::Write;
use tonic::Request;

pub mod communication {
    tonic::include_proto!("communication");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut node_addr = String::new();

    //this part will be handled by a load balancer
    print!("enter node's address to connect to(egs: 127.0.0.1:8000): ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut node_addr)?;
    let node_addr = String::from("http://") + node_addr.trim();

    let mut client = ReplicationServiceClient::connect(node_addr.clone()).await?;
    println!("connected to: {}", node_addr);
    println!(
        r#"
                                     ______  ______
                                    |  _  \ | ___ \
 _ __ ___    ___  _ __  __ _   ___  | | | | | |_/ /
| '_ ` _ \  / _ \ | '__|/ _` | / _ \ | | | | | ___ \
| | | | | ||  __/ | |  | (_| ||  __/ | |/ /  | |_/ /
|_| |_| |_| \___| |_|   \__, | \___| |___/   \____/
                         __/ |
                        |___/
    "#
    );

    loop {
        let mut user_query = String::new();

        print!(":: ");
        std::io::stdout().flush().unwrap();

        std::io::stdin()
            .read_line(&mut user_query)
            .expect("failed to read line");
        let parts: Vec<&str> = user_query.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];

        if cmd == "HELP" {
            println!("the following operations are possible as of now: ");
            println!("CSET key value (e.g., CSET mykey 10)");
            println!("CGET key");
            println!("CINC key amt");
            println!("CDEC key amt");
            continue;
        }

        if parts.len() == 3 {
            let value_type = String::from(parts[0]);
            let key = String::from(parts[1]);
            let val_str = parts[2];

            if value_type.starts_with('C') {
                let parsed_value = match val_str.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Error: Value must be an integer");
                        continue;
                    }
                };

                let request = Request::new(PropagateDataRequest {
                    valuetype: value_type.clone(),
                    key: key.clone(),
                    value: parsed_value.to_be_bytes().to_vec(),
                });

                match client.propagate_data(request).await {
                    Ok(response) => println!("response: {:?}", response.into_inner()),
                    Err(e) => println!("RPC Failed: {}", e),
                }
            } else {
                println!("not supported at the moment...");
            }
        } else if parts.len() == 2 {
            let value_type = String::from(parts[0]);
            let key = String::from(parts[1]);

            if value_type == "CGET" {
                let request = Request::new(PropagateDataRequest {
                    valuetype: value_type.clone(),
                    key: key.clone(),
                    value: Vec::new(), //send empty bytes instead
                });

                match client.propagate_data(request).await {
                    Ok(response) => {
                        let resp_inner = response.into_inner().response;

                        let bytes: [u8; 8] = resp_inner.try_into().unwrap_or([0; 8]);
                        let val = i64::from_be_bytes(bytes);

                        println!(":: {}", val);
                    }
                    Err(e) => println!("RPC Failed: {}", e),
                }
            }
        } else {
            println!("incorrect query format");
            println!("Type 'HELP' for instructions");
        }
    }
}
