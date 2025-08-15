use std::fs::OpenOptions;
use std::io::Write;

use serde::{Deserialize, Serialize};

mod action;

use action::RefactoringAction;

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    action: String,
    result: String,
}

fn main() {
    let mut log_file = OpenOptions::new().append(true).open("./log.txt").unwrap();

    let entry: Entry = serde_json::from_slice(std::env::args().nth(1).unwrap().as_bytes()).unwrap();

    let old = std::fs::read_to_string("./job.c").unwrap();

    let action: RefactoringAction = entry.action.parse().unwrap();

    // Validate the action before applying it
    match action.validate(&old, &entry.result) {
        Ok(()) => {
            println!("Action validated successfully.");
        }
        Err(e) => {
            eprintln!("Error during validation: {}", e);
            std::process::exit(1);
        }
    }

    println!("Old:\n{old}\nAction: {action:?}\nResult:\n{}", entry.result);

    write!(log_file, "{action:?}").unwrap();

    std::fs::write("job.c", entry.result).unwrap();

    println!("Result has been written into the file.");
}
