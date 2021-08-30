use core::fmt;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::HashMap;

static ID_LENGTH: usize = 10;
static ID_ALPHA: [char; 16] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Batch structure:
/// {  
///     "batch_id": "fkIopp4D_K",  
///     "author": "Matt",
///     "source_file": "examples/basic/main.py",
///     "repository": "git@github.com/retwolf/rft"
///     "branch": "master"
///     "jobs": [
///         {...} - See job structure below for this format
///     ]
/// }
#[derive(Clone, Serialize, Deserialize)]
pub struct Batch {
    /// a short nanoid representing the batch
    pub batch_id: String,
    /// the author of the batch
    pub author: String,
    /// relative path from the root of a git repository to the file to be ran for this batch
    pub source_file: String,
    /// the repository url to download the source code from
    pub repository_url: String,
    /// the git branch to checkout and execute from
    pub branch: String,
    /// a list of jobs to be executed in this batch
    pub jobs: Vec<Job>,
}

impl Batch {
    pub fn new(author: &str, source_file: &str, repository_url: &str, branch: &str) -> Batch {
        Batch {
            batch_id: nanoid!(ID_LENGTH, &ID_ALPHA),
            author: author.to_string(),
            source_file: source_file.to_string(),
            repository_url: repository_url.to_string(),
            branch: branch.to_string(),
            jobs: Vec::<Job>::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Batch, Error> {
        match serde_json::from_str(json) {
            Ok(b) => Ok(b),
            Err(e) => Err(e),
        }
    }
}

impl fmt::Display for Batch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "batch_id: {}", &self.batch_id).unwrap_or(());
        writeln!(f, "author: {}", &self.author).unwrap_or(());
        writeln!(f, "source_file: {}", &self.source_file).unwrap_or(());
        writeln!(f, "repository_url: {}", &self.repository_url).unwrap_or(());
        writeln!(f, "branch: {}", &self.branch).unwrap_or(());
        writeln!(f, "jobs: ").unwrap_or(());
        for job in &self.jobs.clone() {
            write!(f, "{}", &job).unwrap_or(());
        }

        write!(f, "")
    }
}

// Job structure:
// {
//     "job_id": "EKKFKWaBJZ"
//     "params": {
//         "start_date": "1980",
//         "end_date": "2020",
//     }
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub params: HashMap<String, String>,
}

impl Job {
    pub fn new(params: HashMap<String, String>) -> Job {
        Job {
            job_id: nanoid!(ID_LENGTH, &ID_ALPHA),
            params,
        }
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "job_id: {}", &self.job_id).unwrap_or(());
        writeln!(f, "params: ").unwrap_or(());
        for (key, val) in &self.params.clone() {
            writeln!(f, "  '{}': '{}'", key, val).unwrap_or(());
        }

        write!(f, "")
    }
}
