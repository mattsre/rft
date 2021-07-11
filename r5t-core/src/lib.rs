use core::fmt;
use std::collections::HashMap;
// Params structure:
// {
//     source_file: "test_passive.py",
//     params: {
//         "start_date": "1980",
//         "end_date": "2020",
//     }
// }

#[derive(Clone)]
pub struct Job {
    pub source_file: String,
    pub params: HashMap<String, String>,
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source_file: {}\n", &self.source_file).unwrap_or(());
        write!(f, "params: \n").unwrap_or(());
        for (key, val) in &self.params.clone() {
            write!(f, "  '{}': '{}'\n", key, val).unwrap_or(());
        }

        write!(f, "")
    }
}

impl Job {
    pub fn new(filename: &str, params: HashMap<String, String>) -> Job {
        return Job {
            source_file: filename.to_string(),
            params,
        };
    }
}
