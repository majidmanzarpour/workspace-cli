use crate::client::ApiClient;
use crate::error::Result;
use super::batch_types::*;

/// Add a shape to a slide, optionally with text, fill color, and text styling
pub async fn add_shape(
    client: &ApiClient,
    presentation_id: &str,
    object_id: &str,
    slide_id: &str,
    shape_type: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    text: Option<&str>,
    fill: Option<&str>,
    font_size: Option<f64>,
    bold: bool,
) -> Result<SlidesBatchUpdateResponse> {
    let mut requests = vec![
        create_shape_request(object_id, slide_id, shape_type, x, y, width, height),
    ];

    if let Some(txt) = text {
        requests.push(insert_text_request(object_id, txt));

        // Apply text style if font_size or bold specified
        if font_size.is_some() || bold {
            requests.push(update_text_style_request(object_id, font_size, bold, txt.len()));
        }
    }

    if let Some(color) = fill {
        let (r, g, b) = parse_hex_color(color)
            .map_err(|e| crate::error::WorkspaceError::Config(e))?;
        requests.push(update_shape_fill_request(object_id, r, g, b));

        // Auto-set text to white if fill is dark
        if text.is_some() && (r + g + b) < 1.5 {
            requests.push(update_text_color_request(object_id, 1.0, 1.0, 1.0));
        }
    }

    let request = SlidesBatchUpdateRequest { requests };
    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}

/// Add a table to a slide, optionally populated with data and header color
pub async fn add_table(
    client: &ApiClient,
    presentation_id: &str,
    object_id: &str,
    slide_id: &str,
    rows: u32,
    cols: u32,
    data: Option<&Vec<Vec<String>>>,
    header_color: Option<&str>,
) -> Result<SlidesBatchUpdateResponse> {
    let mut requests = vec![
        create_table_request(object_id, slide_id, rows, cols),
    ];

    // Populate cells with data
    if let Some(data) = data {
        for (r, row) in data.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if !cell.is_empty() {
                    requests.push(insert_table_text_request(
                        object_id, r as u32, c as u32, cell,
                    ));
                }
            }
        }
    }

    // Apply header row color
    if let Some(color) = header_color {
        let (r, g, b) = parse_hex_color(color)
            .map_err(|e| crate::error::WorkspaceError::Config(e))?;
        requests.push(update_table_cell_fill_request(object_id, 0, 0, cols, r, g, b));
    }

    let request = SlidesBatchUpdateRequest { requests };
    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}

/// Embed a Google Sheets chart on a slide
pub async fn add_chart(
    client: &ApiClient,
    presentation_id: &str,
    object_id: &str,
    slide_id: &str,
    spreadsheet_id: &str,
    chart_id: u64,
    linked: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<SlidesBatchUpdateResponse> {
    let request = SlidesBatchUpdateRequest {
        requests: vec![create_sheets_chart_request(
            object_id,
            slide_id,
            spreadsheet_id,
            chart_id,
            linked,
            x, y, width, height,
        )],
    };
    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}

/// Delete a slide or page element
pub async fn delete_object(
    client: &ApiClient,
    presentation_id: &str,
    object_id: &str,
) -> Result<SlidesBatchUpdateResponse> {
    let request = SlidesBatchUpdateRequest {
        requests: vec![delete_object_request(object_id)],
    };
    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}

/// Raw batchUpdate passthrough
pub async fn batch_update(
    client: &ApiClient,
    presentation_id: &str,
    requests: Vec<serde_json::Value>,
) -> Result<SlidesBatchUpdateResponse> {
    let request = SlidesBatchUpdateRequest { requests };
    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}
