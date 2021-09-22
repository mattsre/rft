use clap::{crate_version, App, Arg, Values};
use git2::{Config, ErrorCode, Repository};
use isahc::{prelude::*, Body, Error, HttpClient, Request, Response};
use rft_core::batch::Batch;
use rft_core::job::Job;
use std::{collections::HashMap, process::exit};

enum ParamError {
    InvalidParam,
}

#[derive(PartialEq, Eq)]
enum ParamFormat {
    Pairs,
    Matrix,
}

fn main() {
    let app = App::new("rft-client")
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
                        "pairs" => match generate_map_for_params(params, ParamFormat::Pairs) {
                            Ok((param_map, num_of_pairs)) => {
                                let repo = match Repository::discover(".") {
                                    Ok(repo) => repo,
                                    Err(e) => {
                                        eprintln!("Failed to open a Git repository: {}", e);
                                        std::process::exit(1);
                                    }
                                };

                                let author = get_current_author();
                                let full_path = get_full_source_path(&repo, filename);
                                let origin_url = get_repository_url(&repo);
                                let current_branch = get_current_branch(&repo);
                                let mut batch =
                                    Batch::new(&author, &full_path, &origin_url, &current_branch);
                                for i in 0..num_of_pairs {
                                    let mut single_job_params: HashMap<String, String> =
                                        HashMap::new();
                                    for (key, val) in &param_map {
                                        single_job_params
                                            .insert(key.clone(), val.get(i).unwrap().clone());
                                    }

                                    let job = Job::new(single_job_params);
                                    batch.jobs.push(job.clone());
                                }

                                println!("Batch has {} jobs", batch.jobs.len());
                                let json = serde_json::to_string(&batch)
                                    .unwrap_or_else(|_| "".to_string());
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
                        "matrix" => match generate_map_for_params(params, ParamFormat::Matrix) {
                            Ok((param_map, _)) => {
                                let repo = match Repository::discover(".") {
                                    Ok(repo) => repo,
                                    Err(e) => {
                                        eprintln!("Failed to open a Git repository: {}", e);
                                        std::process::exit(1);
                                    }
                                };

                                let combos = generate_param_combos(param_map);

                                let author = get_current_author();
                                let full_path = get_full_source_path(&repo, filename);
                                let origin_url = get_repository_url(&repo);
                                let current_branch = get_current_branch(&repo);
                                let mut batch =
                                    Batch::new(&author, &full_path, &origin_url, &current_branch);
                                for combo in combos {
                                    let job = Job::new(combo);
                                    batch.jobs.push(job.clone());
                                }

                                println!("Batch has {} jobs", batch.jobs.len());

                                let json = serde_json::to_string(&batch)
                                    .unwrap_or_else(|_| "".to_string());
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

fn get_current_author() -> String {
    let gitconfig = Config::open(
        &Config::find_global()
            .expect("Unable to find global gitconfig. Does one exist in ${HOME}/.gitconfig?"),
    )
    .expect("Unable to open gitconfig to find author name");

    match gitconfig.get_string("user.name") {
        Ok(name) => name,
        Err(e) => {
            eprintln!(
                "No username found in .gitconfig. Please set a username with git config --global user.name. \nError: {}", &e
            );
            std::process::exit(1);
        }
    }
}

fn get_full_source_path(repo: &Repository, filename: &str) -> String {
    let path_prefix = match repo.workdir() {
        Some(p) => p,
        None => {
            eprintln!(
                "The current repository is bare. rft does not support working with bare repositories"
            );
            std::process::exit(1);
        }
    };

    let current_dir = std::env::current_dir().expect("Unable to get current directory");
    let source_path = current_dir.join(filename);

    source_path
        .strip_prefix(path_prefix)
        .unwrap_or_else(|e| {
            eprintln!("Unable to strip path prefix: {}", &e);
            std::process::exit(1);
        })
        .to_str()
        .expect("Couldn't convert path to a string")
        .to_owned()
}

fn get_repository_url(repo: &Repository) -> String {
    let origin = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(e) => {
            eprintln!("Failed to find a remote named origin. rft-client only supports executing from git repositories using origin as their remote name: {}", e);
            std::process::exit(1);
        }
    };

    let url = match origin.url() {
        Some(url) => url.to_string(),
        None => {
            eprintln!(
                "No URL configured for the git remote 'origin'. Please configure an origin URL"
            );
            std::process::exit(1);
        }
    };

    println!("Found origin URL: {}", &url);

    url
}

fn get_current_branch(repo: &Repository) -> String {
    let git_head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(_) => todo!(),
    };

    let branch = match git_head.as_ref().and_then(|h| h.shorthand()) {
        Some(branch) => branch,
        None => {
            eprintln!("Not currently on any Git branch. Please checkout to a working branch");
            std::process::exit(1);
        }
    };

    println!("Currently on branch: {}", branch);

    branch.to_string()
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
        let mut new_combos = Vec::<HashMap<String, String>>::new();
        for val in values {
            if combos.is_empty() {
                let mut temp = HashMap::new();
                temp.insert(key.clone(), val);
                new_combos.push(temp);
            } else {
                for mut combo in combos.clone() {
                    combo.insert(key.clone(), val.clone());
                    new_combos.push(combo);
                }
            }
        }
        combos = new_combos;
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
        if values.is_empty() {
            return Err(ParamError::InvalidParam);
        }

        // Ensure all parameters are of the same length
        if format == ParamFormat::Pairs {
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
