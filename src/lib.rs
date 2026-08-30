//! Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from tl_parse"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("tl_parse"));
    }
}
