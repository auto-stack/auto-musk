//! Integration tests for the ok.txt file artifact.
//!
//! These tests verify that the ok.txt file was created correctly
//! as part of the implementation work product.

use std::fs;
use std::path::Path;

/// Test that ok.txt exists in the project root.
#[test]
fn test_ok_file_exists() {
    let ok_path = Path::new("ok.txt");
    assert!(ok_path.exists(), "ok.txt should exist in the project root");
    assert!(ok_path.is_file(), "ok.txt should be a file, not a directory");
}

/// Test that ok.txt contains the correct content.
#[test]
fn test_ok_file_content() {
    let content = fs::read_to_string("ok.txt")
        .expect("Should be able to read ok.txt");
    
    assert_eq!(content, "ok", "ok.txt should contain exactly 'ok'");
}

/// Test that ok.txt has the expected file size (2 bytes).
#[test]
fn test_ok_file_size() {
    let metadata = fs::metadata("ok.txt")
        .expect("Should be able to read ok.txt metadata");
    
    let file_size = metadata.len();
    assert_eq!(file_size, 2, "ok.txt should be exactly 2 bytes (content 'ok')");
}

/// Test that ok.txt can be read as bytes correctly.
#[test]
fn test_ok_file_bytes() {
    let bytes = fs::read("ok.txt")
        .expect("Should be able to read ok.txt as bytes");
    
    assert_eq!(bytes, b"ok", "ok.txt bytes should equal b\"ok\"");
}
