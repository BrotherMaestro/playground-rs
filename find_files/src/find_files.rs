// 2023 Hayden Sip

use std::{path::{PathBuf}, ffi::OsStr};
use walkdir::{WalkDir};

fn os_str_contains_name(os_file_name : &OsStr, file_name : &str) -> bool {
    os_file_name
        .to_str()
        .unwrap_or_default()
        .contains(file_name)
}

// Search for files containing file_name, starting from parent directory described by root_directory
pub fn find_files_containing_name(root_directory: &str, file_name : &str) -> Vec<PathBuf> {
    WalkDir::new(root_directory)
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| os_str_contains_name(x.file_name(), file_name))
        .map(|x| x.into_path())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_os_str_with_name() {
        // Construct the os_str to test (as a file name)
        let os_file_name = OsStr::new("sample.txt");

        // Check rejection of standard fail case (names are different)
        assert!(!os_str_contains_name(os_file_name, "reject.txt"));

        // Check acceptance of standard succeeding case (names exactly match)
        assert!(os_str_contains_name(os_file_name, "sample.txt"));

        // Check a partial match is successful
        assert!(os_str_contains_name(os_file_name, "ample"));

        // Expect failure when matching against a path
        assert!(!os_str_contains_name(os_file_name, "tests/assets/sample.txt"));
    }
}