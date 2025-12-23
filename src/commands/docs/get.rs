use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Document, StructuralElement, Paragraph};

pub async fn get_document(client: &ApiClient, document_id: &str) -> Result<Document> {
    let path = format!("/documents/{}", document_id);
    client.get(&path).await
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
