use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

use crate::{ID_ALPHA, ID_LENGTH};

// Job structure:
// {
//     "job_id": "EKKFKWaBJZ",
//     "params": {
//         "start_date": "1980",
//         "end_date": "2020"
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
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
