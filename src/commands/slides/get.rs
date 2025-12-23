use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Presentation, Page, PageElement, TextContent};

pub async fn get_presentation(client: &ApiClient, presentation_id: &str) -> Result<Presentation> {
    let path = format!("/presentations/{}", presentation_id);
    client.get(&path).await
}

pub async fn get_page(
    client: &ApiClient,
    presentation_id: &str,
    page_id: &str,
) -> Result<Page> {
    // Google Slides API doesn't support fetching individual pages
    // We need to fetch the entire presentation and extract the page
    let presentation = get_presentation(client, presentation_id).await?;

    presentation.slides.into_iter()
        .find(|page| page.object_id == page_id)
        .ok_or_else(|| crate::error::WorkspaceError::NotFound(
            format!("Page {} not found in presentation {}", page_id, presentation_id)
        ))
}

/// Extract all text from a presentation
pub fn extract_all_text(presentation: &Presentation) -> String {
    let mut text = String::new();

    text.push_str(&format!("# {}\n\n", presentation.title));

    for (i, slide) in presentation.slides.iter().enumerate() {
        text.push_str(&format!("## Slide {}\n\n", i + 1));
        text.push_str(&extract_page_text(slide));
        text.push('\n');
    }

    text
}

/// Extract text from a single page/slide
pub fn extract_page_text(page: &Page) -> String {
    let mut text = String::new();

    for element in &page.page_elements {
        if let Some(element_text) = extract_element_text(element) {
            if !element_text.trim().is_empty() {
                text.push_str(&element_text);
                text.push('\n');
            }
        }
    }

    text
}

fn extract_element_text(element: &PageElement) -> Option<String> {
    // Extract from shape
    if let Some(ref shape) = element.shape {
        if let Some(ref text_content) = shape.text {
            return Some(extract_text_content(text_content));
        }
    }

    // Extract from table
    if let Some(ref table) = element.table {
        let mut table_text = String::new();
        for row in &table.table_rows {
            let cells: Vec<String> = row.table_cells.iter()
                .filter_map(|cell| cell.text.as_ref().map(extract_text_content))
                .collect();
            table_text.push_str(&cells.join(" | "));
            table_text.push('\n');
        }
        return Some(table_text);
    }

    // Extract from WordArt
    if let Some(ref word_art) = element.word_art {
        if let Some(ref text) = word_art.rendered_text {
            return Some(text.clone());
        }
    }

    None
}

fn extract_text_content(content: &TextContent) -> String {
    let mut text = String::new();

    for element in &content.text_elements {
        if let Some(ref text_run) = element.text_run {
            if let Some(ref content) = text_run.content {
                text.push_str(content);
            }
        }
    }

    text
}

/// Get slide text by index (0-based)
pub fn get_slide_text(presentation: &Presentation, index: usize) -> Option<String> {
    presentation.slides.get(index).map(extract_page_text)
}

/// Get presentation summary (titles and slide count)
pub fn get_summary(presentation: &Presentation) -> serde_json::Value {
    let slide_titles: Vec<String> = presentation.slides.iter()
        .enumerate()
        .map(|(i, slide)| {
            // Try to extract title from first text element
            let title = slide.page_elements.iter()
                .filter_map(|e| extract_element_text(e))
                .next()
                .map(|t| t.lines().next().unwrap_or("").to_string())
                .unwrap_or_else(|| format!("Slide {}", i + 1));
            title.trim().to_string()
        })
        .collect();

    serde_json::json!({
        "title": presentation.title,
        "slide_count": presentation.slides.len(),
        "slides": slide_titles
    })
}
