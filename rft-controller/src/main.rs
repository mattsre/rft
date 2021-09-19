use std::env;
use std::process::exit;

use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::batch::v1::Job as K8S_JOB;
use kube::{
    api::{ListParams, PostParams, WatchEvent},
    Api, Client, ResourceExt,
};
use redis::{Commands, ConnectionAddr, ConnectionInfo, RedisConnectionInfo};
use rft_core::Batch;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let redis_password = env::var("REDIS_PASSWORD").unwrap_or_else(|_| "Mxu168c6OL".to_string());
    let connection_details = ConnectionInfo {
        addr: ConnectionAddr::Tcp(redis_host, 6379),
        redis: RedisConnectionInfo {
            db: 0,
            username: None,
            password: Some(redis_password),
        },
    };

    let kube_client = Client::try_default().await?;
    let jobs: Api<K8S_JOB> = Api::namespaced(kube_client, "default");

    // TODO: improve unwrap() error handling here
    if let Ok(redis_client) = redis::Client::open(connection_details) {
        match redis_client.get_connection() {
            Ok(mut conn) => loop {
                let mut queued_batches: Vec<String> = conn.lrange("queued_batches", 0, -1).unwrap();

                if queued_batches.is_empty() {
                    tokio::time::sleep(Duration::from_millis(5000)).await;
                    continue;
                }

                while !queued_batches.is_empty() {
                    let json_batch: String = conn.lpop("queued_batches", None).unwrap();

                    if let Ok(batch) = Batch::from_json(&json_batch) {
                        println!("Processing batch: \n{}", batch);

                        let indexed_job = serde_json::from_value(serde_json::json!({
                            "apiVersion": "batch/v1",
                            "kind": "Job",
                            "metadata": {
                                "name": format!("rft-indexed-job-{}", batch.batch_id)
                            },
                            "spec": {
                                "completions": batch.jobs.len(),
                                "parallelism": 3,
                                "completionMode": "Indexed",
                                "template": {
                                    "spec": {
                                        "restartPolicy": "Never",
                                        "volumes": [
                                            {
                                                "name": "input",
                                                "emptyDir": {}
                                            },
                                        ],
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
                                                        "name": "input",
                                                        "mountPath": "/input"
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
                                                        "name": "input",
                                                        "mountPath": "/input"
                                                    }
                                                ]
                                            }
                                        ],
                                    }
                                }
                            },
                        }))?;

                        jobs.create(&PostParams::default(), &indexed_job).await?;
                        let lp = ListParams::default()
                            .fields(&format!("metadata.name={}", "rft-indexed-job"))
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
                                WatchEvent::Deleted(o) => println!("Deleted Job: {}", o.name()),
                                WatchEvent::Error(e) => println!("Error {}", e),
                                _ => {}
                            }
                        }

                        queued_batches = conn.lrange("queued_batches", 0, -1).unwrap();
                    } else {
                        eprintln!("Unable to deserialize batch, skipping: {}", &json_batch);
                    }
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
