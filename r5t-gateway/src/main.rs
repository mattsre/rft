#[macro_use]
extern crate rocket;

use r5t_core::Batch;
use redis::{Commands, Connection, ConnectionAddr, ConnectionInfo, RedisResult};
use rocket::serde::json::{serde_json::json, Json, Value};

#[post("/batch", format = "json", data = "<batch>")]
fn create_batch(batch: Json<Batch>) -> Value {
    println!(
        "Recieved batch with ID: {} from author: {} with {} jobs to process using file: {}",
        &batch.batch_id,
        &batch.author,
        &batch.jobs.len(),
        &batch.source_file
    );

    match push_batch_to_redis(batch.into_inner()) {
        Ok(_) => {
            return json!({
                "status": "ok",
            })
        }
        Err(err) => {
            if err.is_connection_refusal() {
                println!("Error connecting to Redis");
            } else if err.is_io_error() {
                println!("IO Error");
            } else {
                eprintln!("{}", err);
            }

            return json!({
                "status": "failed"
            });
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![create_batch])
}

fn push_batch_to_redis(batch: Batch) -> redis::RedisResult<()> {
    let connection_details = ConnectionInfo {
        addr: Box::new(ConnectionAddr::Tcp("127.0.0.1".to_string(), 6379)),
        db: 0,
        username: None,
        passwd: Some("Mxu168c6OL".to_string()),
    };
    let client = redis::Client::open(connection_details)?;
    let mut conn = client.get_connection()?;

    match serde_json::to_string(&batch) {
        Ok(batch_json) => {
            if let Err(err) = conn.rpush::<&str, String, i32>("queued_batches", batch_json) {
                return RedisResult::Err(err);
            }
        }
        Err(_) => todo!(),
    }

    Ok(())
}
