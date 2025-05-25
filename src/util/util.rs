pub fn get_version() -> String {
    String::from("v2")
}

pub fn format_duration(millis: u64) -> String {
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60000 {
        format!("{:.2}s", millis as f64 / 1000.0)
    } else {
        let seconds = millis / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}m {}s", minutes, remaining_seconds)
    }
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    if name.len() > 255 {
        return Err("Project name is too long (max 255 characters)".to_string());
    }

    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err("Project name contains invalid characters".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        assert_eq!(get_version(), "v2");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test<file>"), "test_file_");
        assert_eq!(sanitize_filename("normal_file"), "normal_file");
        assert_eq!(
            sanitize_filename("file:with|special*chars"),
            "file_with_special_chars"
        );
    }

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("Valid_Project").is_ok());
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("Project<With>Invalid:Chars").is_err());

        let long_name = "a".repeat(256);
        assert!(validate_project_name(&long_name).is_err());
    }
}
