use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Spreadsheet, ValueRange, BatchGetValuesResponse};

pub async fn get_spreadsheet(client: &ApiClient, spreadsheet_id: &str) -> Result<Spreadsheet> {
    let path = format!("/spreadsheets/{}", spreadsheet_id);
    client.get(&path).await
}

pub async fn get_values(
    client: &ApiClient,
    spreadsheet_id: &str,
    range: &str,
) -> Result<ValueRange> {
    let path = format!(
        "/spreadsheets/{}/values/{}",
        spreadsheet_id,
        urlencoding::encode(range)
    );
    client.get(&path).await
}

pub async fn get_multiple_ranges(
    client: &ApiClient,
    spreadsheet_id: &str,
    ranges: &[&str],
) -> Result<BatchGetValuesResponse> {
    let ranges_param = ranges.iter()
        .map(|r| ("ranges", r.to_string()))
        .collect::<Vec<_>>();

    let path = format!("/spreadsheets/{}/values:batchGet", spreadsheet_id);
    client.get_with_query(&path, &ranges_param).await
}

/// Convert ValueRange to CSV string
pub fn values_to_csv(values: &ValueRange) -> String {
    let mut csv = String::new();

    for row in &values.values {
        let line: Vec<String> = row.iter()
            .map(|cell| {
                let s = match cell {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    _ => cell.to_string(),
                };
                // Escape for CSV (RFC 4180)
                if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s
                }
            })
            .collect();

        csv.push_str(&line.join(","));
        csv.push('\n');
    }

    csv
}

/// Parse a range string to extract sheet name and cell range
/// Handles A1 notation including quoted sheet names (e.g., 'Sheet Name'!A1:B2)
/// and escaped quotes within sheet names (e.g., 'John''s Data'!A1)
pub fn parse_range(range: &str) -> (Option<String>, String) {
    // Find the separator '!' that's not inside quotes
    let separator_pos = if range.starts_with('\'') {
        // Sheet name is quoted, find the closing quote (accounting for escaped quotes '')
        let mut i = 1;
        let chars: Vec<char> = range.chars().collect();
        while i < chars.len() {
            if chars[i] == '\'' {
                // Check if this is an escaped quote ('')
                if i + 1 < chars.len() && chars[i + 1] == '\'' {
                    // Escaped quote, skip both
                    i += 2;
                } else {
                    // This is the closing quote
                    // The separator should be right after
                    if i + 1 < chars.len() && chars[i + 1] == '!' {
                        break;
                    } else {
                        // Closing quote found but no separator - return None
                        return (None, range.to_string());
                    }
                }
            } else {
                i += 1;
            }
        }
        // i + 1 is the position of the '!' separator
        if i < chars.len() && i + 1 < chars.len() && chars[i + 1] == '!' {
            Some(i + 1)
        } else {
            None
        }
    } else {
        // Sheet name is not quoted, find the first '!'
        range.find('!')
    };

    if let Some(pos) = separator_pos {
        let sheet_part = &range[..pos];
        // Remove enclosing single quotes if present and unescape internal quotes
        let sheet = if sheet_part.starts_with('\'') && sheet_part.ends_with('\'') && sheet_part.len() >= 2 {
            // Remove outer quotes and unescape internal quotes ('' -> ')
            sheet_part[1..sheet_part.len() - 1].replace("''", "'")
        } else {
            sheet_part.to_string()
        };
        let cells = range[pos + 1..].to_string();
        (Some(sheet), cells)
    } else {
        (None, range.to_string())
    }
}
