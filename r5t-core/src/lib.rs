use core::fmt;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static ID_LENGTH: usize = 10;

// Batch structure:
// {
//     "batch_id": "fkIopp4D_K",
//     "author": "Matt",
//     "source_file": "testing.py",
//     "jobs": [
//         {...} - See job structure below for this format
//     ]
// }
#[derive(Clone, Serialize, Deserialize)]
pub struct Batch {
    pub batch_id: String,
    pub author: String,
    pub source_file: String,
    pub jobs: Vec<Job>,
}

impl Batch {
    pub fn new(author: &str, source_file: &str) -> Batch {
        Batch {
            batch_id: nanoid!(ID_LENGTH),
            author: author.to_string(),
            source_file: source_file.to_string(),
            jobs: Vec::<Job>::new(),
        }
    }
}

impl fmt::Display for Batch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "batch_id: {}\n", &self.batch_id).unwrap_or(());
        write!(f, "author: {}\n", &self.author).unwrap_or(());
        write!(f, "source_file: {}\n", &self.source_file).unwrap_or(());
        write!(f, "jobs: \n").unwrap_or(());
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
        return Job {
            job_id: nanoid!(ID_LENGTH),
            params,
        };
    }
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "job_id: {}\n", &self.job_id).unwrap_or(());
        write!(f, "params: \n").unwrap_or(());
        for (key, val) in &self.params.clone() {
            write!(f, "  '{}': '{}'\n", key, val).unwrap_or(());
        }

        write!(f, "")
    }
}
