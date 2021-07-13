use std::process::exit;

use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client, ResourceExt};
use r5t_core::{Batch, Job};
use redis::{Commands, ConnectionAddr, ConnectionInfo};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let connection_details = ConnectionInfo {
        addr: Box::new(ConnectionAddr::Tcp("127.0.0.1".to_string(), 6379)),
        db: 0,
        username: None,
        passwd: Some("CFLnpBvBb6".to_string()),
    };
    if let Ok(client) = redis::Client::open(connection_details) {
        match client.get_connection() {
            Ok(mut conn) => loop {
                let mut queued_batches: Vec<String> = conn.lrange("queued_batches", 0, -1).unwrap();

                if queued_batches.len() == 0 {
                    println!("No batches in queue");
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                    continue;
                }

                while queued_batches.len() != 0 {
                    let current_batch: String = conn.lpop("queued_batches").unwrap();
                    let parsed_batch: Batch =
                        serde_json::from_str(&current_batch.as_str()).unwrap();

                    println!("Processing batch: \n{}", parsed_batch);

                    queued_batches = conn.lrange("queued_batches", 0, -1).unwrap();
                }

                println!("Processed all batches!");
                tokio::time::sleep(Duration::from_millis(5000)).await;
            },
            Err(err) => {
                eprintln!("FATAL - Error getting connection to Redis queue: {}!", err);
                exit(1)
            }
        }
    }

    Ok(())
}
