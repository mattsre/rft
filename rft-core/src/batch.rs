use crate::{job::Job, ID_ALPHA, ID_LENGTH};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::fmt;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not deserialize batch. Source: {}", source))]
    DeserializeFailed {
        batch: String,
        source: serde_json::Error,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

/// Batch structure:
/// {  
///     "batch_id": "fkIopp4D_K",  
///     "author": "Matt",
///     "source_file": "examples/basic/main.py",
///     "repository_url": "git@github.com/retwolf/rft",
///     "branch": "master",
///     "jobs": [
///         {...} - See job structure below for this format
///     ]
/// }
#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn from_json(json: &str) -> Result<Batch> {
        serde_json::from_str(json).context(DeserializeFailed {
            batch: json.to_string(),
        })
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

#[cfg(test)]
mod tests {
    use crate::batch::Batch;

    #[test]
    fn deserialize_batch() {
        let valid_batch_json = r#"
        {
            "batch_id": "fkIopp4D_K",  
            "author": "Matt",
            "source_file": "examples/basic/main.py",
            "repository_url": "git@github.com/retwolf/rft",
            "branch": "master",
            "jobs": [
                {
                    "job_id": "EKKFKWaBJZ",
                    "params": {
                        "start_date": "1980",
                        "end_date": "2020"
                    }
                }
            ]
        }"#;

        let test_batch =
            Batch::from_json(&valid_batch_json).expect("Should successfully deserialize JSON");

        assert!(test_batch.batch_id == "fkIopp4D_K");

        let invalid_batch_json = r#"
        {
            "batch_id": 1
        }
        "#;

        Batch::from_json(&invalid_batch_json).expect_err("Should produce a deserialization error.");
    }
}
