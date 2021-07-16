use std::process::exit;

use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::{batch::v1::Job as K8S_JOB, core::v1::Pod};
use kube::{
    api::{ListParams, PostParams, WatchEvent},
    Api, Client, ResourceExt,
};
use r5t_core::{Batch, Job};
use redis::{Commands, ConnectionAddr, ConnectionInfo};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let connection_details = ConnectionInfo {
        addr: Box::new(ConnectionAddr::Tcp("127.0.0.1".to_string(), 6379)),
        db: 0,
        username: None,
        passwd: Some("Mxu168c6OL".to_string()),
    };

    let kube_client = Client::try_default().await?;
    let jobs: Api<K8S_JOB> = Api::namespaced(kube_client, "default");

    if let Ok(redis_client) = redis::Client::open(connection_details) {
        match redis_client.get_connection() {
            Ok(mut conn) => loop {
                let mut queued_batches: Vec<String> = conn.lrange("queued_batches", 0, -1).unwrap();

                if queued_batches.len() == 0 {
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                    continue;
                }

                while queued_batches.len() != 0 {
                    let json_batch: String = conn.lpop("queued_batches").unwrap();
                    let parsed_batch: Batch = serde_json::from_str(&json_batch.as_str()).unwrap();

                    println!("Processing batch: \n{}", parsed_batch);

                    let indexed_job = serde_json::from_value(serde_json::json!({
                        "apiVersion": "batch/v1",
                        "kind": "Job",
                        "metadata": {
                            "name": format!("r5t-indexed-job-{}", parsed_batch.batch_id)
                        },
                        "spec": {
                            "completions": parsed_batch.jobs.len(),
                            "parallelism": 3,
                            "completionMode": "Indexed",
                            "template": {
                                "spec": {
                                    "restartPolicy": "Never",
                                    "initContainers": [
                                        {
                                            "name": "input-mapping",
                                            "image": "public.ecr.aws/e1q1z8n5/alpine-jq",
                                            "command": [
                                                "/bin/sh",
                                                "-c",
                                                format!("echo '{}' | jq '.jobs['\"$JOB_COMPLETION_INDEX\"']' > /input/data.json", json_batch)
                                            ],
                                            "volumeMounts": [
                                                {
                                                    "mountPath": "/input",
                                                    "name": "input"
                                                }
                                            ]
                                        }
                                    ],
                                    "containers": [
                                        {
                                            "name": "worker",
                                            "image": "docker.io/library/bash",
                                            "command": [
                                                "bash",
                                                "-c",
                                                "cat /input/data.json"
                                            ],
                                            "volumeMounts": [
                                                {
                                                    "mountPath": "/input",
                                                    "name": "input"
                                                }
                                            ]
                                        }
                                    ],
                                    "volumes": [
                                        {
                                            "name": "input",
                                            "emptyDir": {}
                                        }
                                    ]
                                }
                            }
                        },
                    }))?;

                    let job = jobs.create(&PostParams::default(), &indexed_job).await?;
                    let lp = ListParams::default()
                        .fields(&format!("metadata.name={}", "r5t-indexed-job"))
                        .timeout(10);

                    let mut stream = jobs.watch(&lp, "0").await?.boxed();

                    while let Some(status) = stream.try_next().await? {
                        match status {
                            WatchEvent::Added(o) => println!("Added {}", o.name()),
                            WatchEvent::Modified(o) => {
                                let s = o.status.as_ref().expect("status exists on job");
                                let indices = s.completed_indexes.clone().unwrap_or_default();
                                println!(
                                    "Modified: {} with indices completed: {}",
                                    o.name(),
                                    indices
                                );
                            }
                            WatchEvent::Deleted(o) => println!("Deleted {}", o.name()),
                            WatchEvent::Error(e) => println!("Error {}", e),
                            _ => {}
                        }
                    }

                    queued_batches = conn.lrange("queued_batches", 0, -1).unwrap();
                }

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
