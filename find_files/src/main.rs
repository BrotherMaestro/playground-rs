//! A command line tool to test finding files!
//!

use find_files::find_files::find_files_containing_name;
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                if line == "q" {
                    return;
                } else {
                    let matching_files = find_files_containing_name(".", line.as_str());
                    for file in matching_files {
                        // Simple implementation, without error handling
                        println!("{}", file.into_os_string().into_string().unwrap());
                    }
                }
            }
            Err(err) => {
                println!("Got error {:?}", err);
                println!("Expectect partial file path or name. Press q to exit");
            }
        }
    }
}
