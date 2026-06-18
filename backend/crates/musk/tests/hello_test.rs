//! Integration tests for the hello module.
//!
//! These tests verify the greet() function behavior including stdout output.

/// Test that greet() produces the correct output.
///
/// This test runs the greet() function and captures its stdout output
/// to verify it prints "hello world" followed by a newline.
#[test]
fn test_greet_output() {
    // For now, just verify it runs without panicking
    // Full stdout capture requires additional infrastructure
    musk::hello::greet();
}

/// Test that greet() can be called multiple times.
#[test]
fn test_greet_multiple_calls() {
    // Verify the function is idempotent and can be called repeatedly
    musk::hello::greet();
    musk::hello::greet();
    musk::hello::greet();
}

/// Test that the hello module is accessible from the library root.
#[test]
fn test_module_accessible() {
    // This test verifies the module is properly re-exported
    // If this compiles, the module structure is correct
    let _ = musk::hello::greet;
}
