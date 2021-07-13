use clap::{crate_version, App, Arg, Values};
use isahc::{prelude::*, Body, Error, HttpClient, Request, Response};
use r5t_core::{Batch, Job};
use std::{collections::HashMap, process::exit};

enum ParamError {
    InvalidParam,
}

#[derive(PartialEq, Eq)]
enum ParamFormat {
    PAIRS,
    MATRIX,
}

fn main() {
    let app = App::new("r5t-client")
        .version(crate_version!())
        .about("rust data framework kubernetes operator queue thing")
        .subcommand(App::new("run")
            .about("Creates compute jobs using a source file and set of parameters")
            .arg(
                Arg::new("file")
                    .about(
                        "File to run with provided parameters. Accepts a path relative to the current directory",
                    )
                    .short('f')
                    .long("file")
                    .required(true)
                    .value_name("filename")
                    .takes_value(true)
            )
            .arg(
                Arg::new("params")
                    .about("Parameters to generate job specs from")
                    .multiple(true)
                    .short('p')
                    .long("params")
                    .value_name("key=value1,value2")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::new("format")
                    .about("Format to create job specs from parameters")
                    .long("format")
                    .takes_value(true)
                    .required(true)
                    .possible_values(&["pairs", "matrix"])
                    .default_value("pairs")
            ))
        .get_matches();

    // Handle RUN command logic
    if let Some(run_matches) = app.subcommand_matches("run") {
        if let Some(filename) = run_matches.value_of("file") {
            if let Some(format) = run_matches.value_of("format") {
                if let Some(params) = run_matches.values_of("params") {
                    match format {
                        "pairs" => match generate_map_for_params(params, ParamFormat::PAIRS) {
                            Ok((param_map, num_of_pairs)) => {
                                let mut batch = Batch::new(1, "Matt", filename);
                                for i in 0..num_of_pairs {
                                    let mut single_job_params: HashMap<String, String> =
                                        HashMap::new();
                                    for (key, val) in &param_map {
                                        single_job_params
                                            .insert(key.clone(), val.get(i).unwrap().clone());
                                    }

                                    let job = Job::new(1, single_job_params);
                                    batch.jobs.push(job.clone());
                                }

                                println!("Batch has {} jobs", batch.jobs.len());

                                let json = serde_json::to_string(&batch).unwrap_or("".to_string());
                                match post_batch("http://127.0.0.1:8000/batch", json) {
                                    Ok(mut body) => {
                                        if let Ok(response) = body.text() {
                                            println!("Successfully posted job batch to gateway with response: {}", response);
                                        }
                                    }
                                    Err(err) => println!(
                                        "Error! - Failed to post job batch to gateway: {}",
                                        err
                                    ),
                                }
                            }
                            Err(err) => match err {
                                ParamError::InvalidParam => {
                                    println!("Error! - Number of values across parameters must be equal when in 'pairs' format");
                                    exit(1);
                                }
                            },
                        },
                        "matrix" => match generate_map_for_params(params, ParamFormat::MATRIX) {
                            Ok((param_map, _)) => {
                                let combos = generate_param_combos(param_map);

                                let mut batch = Batch::new(1, "Matt", filename);
                                for combo in combos {
                                    let job = Job::new(1, combo);
                                    batch.jobs.push(job.clone());
                                }

                                println!("Batch has {} jobs", batch.jobs.len());

                                let json = serde_json::to_string(&batch).unwrap_or("".to_string());
                                match post_batch("http://127.0.0.1:8000/batch", json) {
                                    Ok(mut body) => {
                                        if let Ok(response) = body.text() {
                                            println!("Successfully posted job batch to gateway with response: {}", response);
                                        }
                                    }
                                    Err(err) => println!(
                                        "Error! - Failed to post job batch to gateway: {}",
                                        err
                                    ),
                                }
                            }
                            Err(err) => match err {
                                ParamError::InvalidParam => {
                                    println!("Error! - Invalid parameter for 'matrix' format");
                                    exit(1);
                                }
                            },
                        },
                        _ => {
                            println!("Error! - Unknown Error")
                        }
                    }
                }
            }
        }
    }
}

fn post_batch(uri: &str, batch_json: String) -> Result<Response<Body>, Error> {
    let client = HttpClient::new()?;

    let request = Request::post(uri)
        .header("Content-Type", "application/json")
        .body(batch_json)?;

    client.send(request)
}

fn generate_param_combos(params: HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    let mut combos = Vec::<HashMap<String, String>>::new();
    for (key, values) in params {
        if combos.len() == 0 {
            let mut new_combos = Vec::<HashMap<String, String>>::new();
            for val in values {
                let mut temp = HashMap::new();
                temp.insert(key.clone(), val);
                new_combos.push(temp);
            }
            combos = new_combos;
        } else {
            let mut new_combos = Vec::<HashMap<String, String>>::new();
            for val in values {
                for mut combo in combos.clone() {
                    combo.insert(key.clone(), val.clone());
                    new_combos.push(combo);
                }
            }
            combos = new_combos;
        }
    }

    combos
}

fn generate_map_for_params(
    params: Values,
    format: ParamFormat,
) -> Result<(HashMap<String, Vec<String>>, usize), ParamError> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut param_length = 0;
    for param in params {
        let (key, val) = param.split_once("=").unwrap();
        let values: Vec<String> = val.split(',').map(|x| x.to_string()).collect();

        // Ensure parameter has at least one value
        if values.len() == 0 {
            return Err(ParamError::InvalidParam);
        }

        // Ensure all parameters are of the same length
        if format == ParamFormat::PAIRS {
            if param_length != 0 {
                if values.len() != param_length {
                    return Err(ParamError::InvalidParam);
                }
            } else {
                param_length = values.len();
            }
        }

        map.insert(key.to_string(), values);
    }

    Ok((map, param_length))
}
