use std::path::PathBuf;

use find_files::find_files::find_files_containing_name;

#[test]
fn found_file_containing_name() {
    // look for sample.txt
    let maybe_paths = find_files_containing_name("tests/assets", "ample1");

    assert!(!maybe_paths.is_empty());

    if let Some(path) = maybe_paths.first() {
        assert_eq!(path.as_os_str(), "tests/assets/sample1.txt");
    }
}

#[test]
fn found_files_containing_name() {
    // look for sample.txt
    let maybe_paths = find_files_containing_name("tests/assets", "sam");

    let sample1_path = PathBuf::from("tests/assets/sample1.txt");
    let sample2_path = PathBuf::from("tests/assets/sample2.txt");

    assert_eq!(vec![sample1_path, sample2_path], maybe_paths);
}

#[test]
fn no_files_containing_name() {
    // look for sample.txt
    let maybe_paths = find_files_containing_name("tests/assets", "false_sample.txt");

    assert!(maybe_paths.is_empty());
}
