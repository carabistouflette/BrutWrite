//! Input validation utilities for Tauri commands.
//!
//! Provides centralized validation for user inputs to prevent security issues
//! like path traversal attacks and resource exhaustion.

use std::path::Path;

/// Maximum length for names (project names, author names, etc.)
const MAX_NAME_LENGTH: usize = 256;

/// Maximum content size (10MB)
const MAX_CONTENT_LENGTH: usize = 10 * 1024 * 1024;

/// Characters that are not allowed in names
const INVALID_NAME_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*', '\0'];

/// Validate that a path doesn't contain traversal attempts.
///
/// # Arguments
/// * `path` - The path string to validate
///
/// # Returns
/// * `Ok(())` if the path is safe
/// * `Err(Validation)` if path traversal is detected
pub fn validate_path(path: &str) -> crate::errors::Result<()> {
    // Check for obvious path traversal patterns
    if path.contains("..") {
        return Err(crate::errors::Error::Validation(
            "Path traversal not allowed: '..' sequences are forbidden".into(),
        ));
    }

    // Normalize and check the path
    let path_obj = Path::new(path);

    // Check for absolute path attempts on Unix that might escape intended directories
    // This is a basic check; for production, consider canonicalization
    for component in path_obj.components() {
        if let std::path::Component::ParentDir = component {
            return Err(crate::errors::Error::Validation(
                "Path traversal not allowed".into(),
            ));
        }
    }

    Ok(())
}

/// Validate a name field (project name, author name, chapter title, etc.)
///
/// # Arguments
/// * `name` - The name to validate
///
/// # Returns
/// * `Ok(())` if the name is valid
/// * `Err(Validation)` if the name is invalid
pub fn validate_name(name: &str) -> crate::errors::Result<()> {
    // Check length
    if name.is_empty() {
        return Err(crate::errors::Error::Validation(
            "Name cannot be empty".into(),
        ));
    }

    if name.len() > MAX_NAME_LENGTH {
        return Err(crate::errors::Error::Validation(format!(
            "Name exceeds maximum length of {} characters",
            MAX_NAME_LENGTH
        )));
    }

    // Check for control characters
    if name.chars().any(|c| c.is_control()) {
        return Err(crate::errors::Error::Validation(
            "Name contains invalid control characters".into(),
        ));
    }

    // Check for invalid filesystem characters
    if name.chars().any(|c| INVALID_NAME_CHARS.contains(&c)) {
        return Err(crate::errors::Error::Validation(format!(
            "Name contains invalid characters: {:?}",
            INVALID_NAME_CHARS
        )));
    }

    // Check that name doesn't start or end with whitespace
    if name != name.trim() {
        return Err(crate::errors::Error::Validation(
            "Name cannot start or end with whitespace".into(),
        ));
    }

    Ok(())
}

/// Validate content size to prevent resource exhaustion.
///
/// # Arguments
/// * `content` - The content to validate
///
/// # Returns
/// * `Ok(())` if the content size is acceptable
/// * `Err(Validation)` if the content is too large
pub fn validate_content_size(content: &str) -> crate::errors::Result<()> {
    if content.len() > MAX_CONTENT_LENGTH {
        return Err(crate::errors::Error::Validation(format!(
            "Content exceeds maximum size of {} bytes ({} MB)",
            MAX_CONTENT_LENGTH,
            MAX_CONTENT_LENGTH / (1024 * 1024)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_normal() {
        assert!(validate_path("/home/user/project").is_ok());
        assert!(validate_path("relative/path").is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("../etc/passwd").is_err());
        assert!(validate_path("/home/../etc").is_err());
        assert!(validate_path("path/to/../../secret").is_err());
    }

    #[test]
    fn test_validate_name_normal() {
        assert!(validate_name("My Project").is_ok());
        assert!(validate_name("John Doe").is_ok());
        assert!(validate_name("Chapter-1_draft").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(validate_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_name_control_chars() {
        assert!(validate_name("name\x00with\x01control").is_err());
        assert!(validate_name("name\nwith\nnewlines").is_err());
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(validate_name("name<with>brackets").is_err());
        assert!(validate_name("name:with:colons").is_err());
        assert!(validate_name("name|with|pipes").is_err());
    }

    #[test]
    fn test_validate_name_whitespace() {
        assert!(validate_name("  leading").is_err());
        assert!(validate_name("trailing  ").is_err());
        assert!(validate_name("  both  ").is_err());
        assert!(validate_name("internal spaces ok").is_ok());
    }

    #[test]
    fn test_validate_content_size_normal() {
        assert!(validate_content_size("normal content").is_ok());
        assert!(validate_content_size(&"a".repeat(1000)).is_ok());
    }

    #[test]
    fn test_validate_content_size_large() {
        let large_content = "a".repeat(MAX_CONTENT_LENGTH + 1);
        assert!(validate_content_size(&large_content).is_err());
    }
}
