use html2text::from_read;

/// Convert HTML to Markdown-like plain text
pub fn html_to_markdown(html: &str) -> String {
    // html2text handles the conversion
    let bytes = html.as_bytes();
    from_read(bytes, 80)
}

/// Convert HTML to plain text (strips all formatting)
pub fn html_to_text(html: &str) -> String {
    // Use html2text with a very wide width to avoid line breaks
    let bytes = html.as_bytes();
    let text = from_read(bytes, 10000);

    // Clean up extra whitespace
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Detect if content is HTML
/// Uses heuristics to avoid false positives from code snippets or comparisons
pub fn is_html(content: &str) -> bool {
    let trimmed = content.trim_start();

    // Strong indicators - definitely HTML
    if trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<!doctype")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<HTML") {
        return true;
    }

    // Check for common HTML tags with proper closing or attributes
    // This reduces false positives from code like "array<int>" or "x < 5"
    let has_html_tags = trimmed.contains("<div") || trimmed.contains("</div>")
        || trimmed.contains("<p>") || trimmed.contains("</p>")
        || trimmed.contains("<br") || trimmed.contains("<br/>")
        || trimmed.contains("<table") || trimmed.contains("</table>")
        || trimmed.contains("<span") || trimmed.contains("</span>")
        || trimmed.contains("<body") || trimmed.contains("</body>");

    // Only consider it HTML if we have tags AND the content looks structured
    has_html_tags && (trimmed.matches('<').count() >= 2 || trimmed.contains("/>"))
}

/// Smart convert - detects HTML and converts, otherwise returns as-is
pub fn smart_convert(content: &str) -> String {
    if is_html(content) {
        html_to_markdown(content)
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_detection() {
        assert!(is_html("<html><body>Hello</body></html>"));
        assert!(is_html("<!DOCTYPE html><html>"));
        assert!(is_html("<div>Some content</div>"));
        assert!(!is_html("Plain text without HTML"));
        assert!(!is_html("Just a < symbol"));
    }

    #[test]
    fn test_html_to_markdown() {
        let html = "<h1>Title</h1><p>Paragraph</p>";
        let md = html_to_markdown(html);
        assert!(md.contains("Title"));
        assert!(md.contains("Paragraph"));
    }
}
