use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Document, StructuralElement, Paragraph};

pub async fn get_document(client: &ApiClient, document_id: &str) -> Result<Document> {
    let path = format!("/documents/{}", document_id);
    client.get(&path).await
}

pub async fn get_document_metadata(client: &ApiClient, document_id: &str) -> Result<Document> {
    let path = format!("/documents/{}", document_id);
    let query = [("fields", "documentId,title,revisionId,body.content(endIndex,paragraph(paragraphStyle(namedStyleType),elements(textRun(content))))")];
    client.get_with_query(&path, &query).await
}

/// Extract info summary from a lightweight document fetch
pub fn document_info(doc: &Document) -> serde_json::Value {
    let mut info = serde_json::Map::new();
    info.insert("documentId".into(), serde_json::json!(doc.document_id));
    info.insert("title".into(), serde_json::json!(doc.title));
    if let Some(ref rev) = doc.revision_id {
        info.insert("revisionId".into(), serde_json::json!(rev));
    }

    let elements = doc.body.as_ref().map(|b| &b.content[..]).unwrap_or(&[]);

    // Character count from max endIndex
    let chars: i64 = elements.iter()
        .filter_map(|e| e.end_index)
        .max()
        .unwrap_or(0);
    info.insert("characters".into(), serde_json::json!(chars));
    info.insert("estimatedTokens".into(), serde_json::json!(chars / 4));

    // Count paragraphs and extract headings
    let mut headings = Vec::new();
    let mut paragraph_count: i64 = 0;
    let mut table_count: i64 = 0;

    for element in elements {
        if let Some(ref para) = element.paragraph {
            paragraph_count += 1;
            if let Some(ref style) = para.paragraph_style {
                if let Some(ref named) = style.named_style_type {
                    if named.starts_with("HEADING") {
                        let text: String = para.elements.iter()
                            .filter_map(|e| e.text_run.as_ref()?.content.as_ref())
                            .map(|s| s.trim().to_string())
                            .collect::<Vec<_>>()
                            .join("");
                        let level = named.strip_prefix("HEADING_")
                            .and_then(|s| s.parse::<i64>().ok())
                            .unwrap_or(0);
                        let mut h = serde_json::Map::new();
                        h.insert("level".into(), serde_json::json!(level));
                        h.insert("text".into(), serde_json::json!(text));
                        headings.push(serde_json::Value::Object(h));
                    }
                }
            }
        }
        if element.table.is_some() {
            table_count += 1;
        }
    }

    info.insert("paragraphs".into(), serde_json::json!(paragraph_count));
    info.insert("tables".into(), serde_json::json!(table_count));
    info.insert("headings".into(), serde_json::Value::Array(headings));

    serde_json::Value::Object(info)
}

/// Convert a Google Doc to Markdown format for token efficiency
pub fn document_to_markdown(doc: &Document) -> String {
    let mut markdown = String::new();

    // Title as H1
    markdown.push_str(&format!("# {}\n\n", doc.title));

    // Process body content
    if let Some(ref body) = doc.body {
        for element in &body.content {
            if let Some(text) = element_to_markdown(element) {
                markdown.push_str(&text);
            }
        }
    }

    markdown
}

fn element_to_markdown(element: &StructuralElement) -> Option<String> {
    if let Some(ref para) = element.paragraph {
        return Some(paragraph_to_markdown(para));
    }

    if let Some(ref table) = element.table {
        return Some(table_to_markdown(table));
    }

    if let Some(ref toc) = element.table_of_contents {
        return Some(toc_to_markdown(toc));
    }

    None
}

fn toc_to_markdown(toc: &super::types::TableOfContents) -> String {
    let mut markdown = String::from("## Table of Contents\n\n");
    for element in &toc.content {
        if let Some(text) = element_to_markdown(element) {
            markdown.push_str(&text);
        }
    }
    markdown.push('\n');
    markdown
}

fn paragraph_to_markdown(para: &Paragraph) -> String {
    let mut text = String::new();

    for elem in &para.elements {
        if let Some(ref text_run) = elem.text_run {
            if let Some(ref content) = text_run.content {
                let mut formatted = content.clone();

                // Apply text styles
                if let Some(ref style) = text_run.text_style {
                    let is_bold = style.bold == Some(true);
                    let is_italic = style.italic == Some(true);
                    let is_strikethrough = style.strikethrough == Some(true);

                    // Handle combined formatting correctly
                    if is_bold && is_italic && is_strikethrough {
                        formatted = format!("***~~{}~~***", formatted.trim());
                    } else if is_bold && is_italic {
                        formatted = format!("***{}***", formatted.trim());
                    } else if is_bold && is_strikethrough {
                        formatted = format!("**~~{}~~**", formatted.trim());
                    } else if is_italic && is_strikethrough {
                        formatted = format!("*~~{}~~*", formatted.trim());
                    } else if is_bold {
                        formatted = format!("**{}**", formatted.trim());
                    } else if is_italic {
                        formatted = format!("*{}*", formatted.trim());
                    } else if is_strikethrough {
                        formatted = format!("~~{}~~", formatted.trim());
                    }
                }

                text.push_str(&formatted);
            }
        } else if elem.horizontal_rule.is_some() {
            text.push_str("\n---\n");
        } else if elem.page_break.is_some() {
            text.push_str("\n<!-- Page Break -->\n");
        } else if elem.column_break.is_some() {
            text.push_str("\n<!-- Column Break -->\n");
        } else if elem.inline_object_element.is_some() {
            text.push_str("[Inline Object]");
        } else if elem.equation.is_some() {
            text.push_str("[Equation]");
        }
    }

    // Apply paragraph styles (headings)
    if let Some(ref style) = para.paragraph_style {
        if let Some(ref named_style) = style.named_style_type {
            let prefix = match named_style.as_str() {
                "HEADING_1" => "# ",
                "HEADING_2" => "## ",
                "HEADING_3" => "### ",
                "HEADING_4" => "#### ",
                "HEADING_5" => "##### ",
                "HEADING_6" => "###### ",
                _ => "",
            };
            if !prefix.is_empty() {
                text = format!("{}{}", prefix, text.trim());
            }
        }
    }

    // Ensure proper line ending
    if !text.ends_with('\n') {
        text.push('\n');
    }

    text
}

fn table_to_markdown(table: &super::types::Table) -> String {
    let mut markdown = String::new();

    for (i, row) in table.table_rows.iter().enumerate() {
        markdown.push('|');
        for cell in &row.table_cells {
            let cell_text = cell.content.iter()
                .filter_map(|e| element_to_markdown(e))
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .replace('\n', " ");
            markdown.push_str(&format!(" {} |", cell_text));
        }
        markdown.push('\n');

        // Add header separator after first row
        if i == 0 {
            markdown.push('|');
            for _ in &row.table_cells {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');
        }
    }

    markdown.push('\n');
    markdown
}

/// Extract plain text from document (even more token efficient)
pub fn document_to_text(doc: &Document) -> String {
    let mut text = String::new();

    if let Some(ref body) = doc.body {
        for element in &body.content {
            if let Some(ref para) = element.paragraph {
                for elem in &para.elements {
                    if let Some(ref text_run) = elem.text_run {
                        if let Some(ref content) = text_run.content {
                            text.push_str(content);
                        }
                    } else if elem.horizontal_rule.is_some() {
                        text.push_str("---");
                    }
                }
            } else if let Some(ref table) = element.table {
                text.push_str(&extract_table_text(table));
            }
        }
    }

    text
}

fn extract_table_text(table: &super::types::Table) -> String {
    let mut text = String::new();
    for row in &table.table_rows {
        for cell in &row.table_cells {
            for element in &cell.content {
                if let Some(ref para) = element.paragraph {
                    for elem in &para.elements {
                        if let Some(ref text_run) = elem.text_run {
                            if let Some(ref content) = text_run.content {
                                text.push_str(content);
                            }
                        }
                    }
                }
            }
            text.push('\t');
        }
        text.push('\n');
    }
    text
}
