use std::process::exit;

use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client, ResourceExt};
use r5t_core::Job;
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
                let mut queued_jobs: Vec<String> = conn.lrange("queued_jobs", 0, -1).unwrap();

                if queued_jobs.len() == 0 {
                    println!("No jobs in queue");
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                    continue;
                }

                while queued_jobs.len() != 0 {
                    let current_job: String = conn.lpop("queued_jobs").unwrap();
                    let current_job_parsed: Job =
                        serde_json::from_str(current_job.as_str()).unwrap();

                    println!("Processing job: \n{}", current_job_parsed);

                    queued_jobs = conn.lrange("queued_jobs", 0, -1).unwrap();
                }

                println!("Processed all jobs!");
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
