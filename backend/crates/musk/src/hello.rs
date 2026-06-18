//! Simple hello world module.
//!
//! Provides a basic greeting function for demonstration purposes.

/// Prints "hello world" to stdout.
///
/// This is a simple function that outputs a greeting message.
/// It can be used for testing or demonstration purposes.
///
/// # Example
///
/// ```
/// musk::hello::greet();
/// ```
pub fn greet() {
    println!("hello world");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        // Test that greet() runs without panicking
        greet();
    }
}
