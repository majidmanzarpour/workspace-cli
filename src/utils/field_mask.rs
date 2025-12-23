/// Parse a comma-separated field mask string into a vector of fields
pub fn parse_field_mask(mask: &str) -> Vec<String> {
    mask.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Build a Google API fields parameter from a list of fields
pub fn build_fields_param(fields: &[String]) -> String {
    fields.join(",")
}

/// Default fields for different resource types (token-efficient defaults)
pub mod defaults {
    pub fn gmail_message() -> Vec<String> {
        vec![
            "id".to_string(),
            "threadId".to_string(),
            "labelIds".to_string(),
            "snippet".to_string(),
            "internalDate".to_string(),
        ]
    }

    pub fn gmail_message_full() -> Vec<String> {
        vec![
            "id".to_string(),
            "threadId".to_string(),
            "labelIds".to_string(),
            "snippet".to_string(),
            "payload".to_string(),
            "internalDate".to_string(),
        ]
    }

    pub fn drive_file() -> Vec<String> {
        vec![
            "id".to_string(),
            "name".to_string(),
            "mimeType".to_string(),
            "webViewLink".to_string(),
            "modifiedTime".to_string(),
        ]
    }

    pub fn drive_file_full() -> Vec<String> {
        vec![
            "id".to_string(),
            "name".to_string(),
            "mimeType".to_string(),
            "webViewLink".to_string(),
            "webContentLink".to_string(),
            "size".to_string(),
            "createdTime".to_string(),
            "modifiedTime".to_string(),
            "parents".to_string(),
        ]
    }

    pub fn calendar_event() -> Vec<String> {
        vec![
            "id".to_string(),
            "summary".to_string(),
            "start".to_string(),
            "end".to_string(),
            "status".to_string(),
        ]
    }

    pub fn calendar_event_full() -> Vec<String> {
        vec![
            "id".to_string(),
            "summary".to_string(),
            "description".to_string(),
            "start".to_string(),
            "end".to_string(),
            "status".to_string(),
            "location".to_string(),
            "attendees".to_string(),
            "organizer".to_string(),
        ]
    }
}

/// Validate that field names don't contain invalid characters
/// Supports Google API field mask syntax including parentheses for sub-selections
pub fn validate_field_mask(fields: &[String]) -> Result<(), FieldMaskError> {
    for field in fields {
        if field.is_empty() {
            return Err(FieldMaskError::EmptyField);
        }
        // Field names can contain: alphanumeric, dots, slashes, underscores, parentheses, and commas
        // Examples: "id", "items/id", "items(id,name)", "user.emailAddress"
        if !field.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '/' || c == '_' || c == '(' || c == ')' || c == ',') {
            return Err(FieldMaskError::InvalidCharacter(field.clone()));
        }

        // Basic validation: parentheses should be balanced
        let open_count = field.chars().filter(|&c| c == '(').count();
        let close_count = field.chars().filter(|&c| c == ')').count();
        if open_count != close_count {
            return Err(FieldMaskError::InvalidCharacter(format!("{} (unbalanced parentheses)", field)));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum FieldMaskError {
    EmptyField,
    InvalidCharacter(String),
}

impl std::fmt::Display for FieldMaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField => write!(f, "Field mask contains empty field"),
            Self::InvalidCharacter(field) => write!(f, "Invalid character in field: {}", field),
        }
    }
}

impl std::error::Error for FieldMaskError {}
