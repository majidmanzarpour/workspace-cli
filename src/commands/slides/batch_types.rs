use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct SlidesBatchUpdateRequest {
    pub requests: Vec<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlidesBatchUpdateResponse {
    pub presentation_id: Option<String>,
    #[serde(default)]
    pub replies: Vec<Value>,
}

/// Parse "#RRGGBB" hex color to (r, g, b) floats in 0.0-1.0 range
pub fn parse_hex_color(hex: &str) -> std::result::Result<(f64, f64, f64), String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!("Invalid hex color '{}': expected 6 hex digits", hex));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())? as f64 / 255.0;
    Ok((r, g, b))
}

pub fn create_slide_request(object_id: &str, index: Option<u32>, layout: &str) -> Value {
    let mut req = json!({
        "createSlide": {
            "objectId": object_id,
            "slideLayoutReference": {
                "predefinedLayout": layout
            }
        }
    });
    if let Some(idx) = index {
        req["createSlide"]["insertionIndex"] = json!(idx);
    }
    req
}

pub fn create_shape_request(
    object_id: &str,
    slide_id: &str,
    shape_type: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Value {
    json!({
        "createShape": {
            "objectId": object_id,
            "shapeType": shape_type,
            "elementProperties": {
                "pageObjectId": slide_id,
                "size": {
                    "height": {"magnitude": height, "unit": "PT"},
                    "width": {"magnitude": width, "unit": "PT"}
                },
                "transform": {
                    "scaleX": 1.0,
                    "scaleY": 1.0,
                    "translateX": x,
                    "translateY": y,
                    "unit": "PT"
                }
            }
        }
    })
}

pub fn insert_text_request(object_id: &str, text: &str) -> Value {
    json!({
        "insertText": {
            "objectId": object_id,
            "insertionIndex": 0,
            "text": text
        }
    })
}

pub fn update_shape_fill_request(object_id: &str, r: f64, g: f64, b: f64) -> Value {
    json!({
        "updateShapeProperties": {
            "objectId": object_id,
            "shapeProperties": {
                "shapeBackgroundFill": {
                    "solidFill": {
                        "color": {
                            "rgbColor": {"red": r, "green": g, "blue": b}
                        }
                    }
                }
            },
            "fields": "shapeBackgroundFill"
        }
    })
}

pub fn update_text_style_request(
    object_id: &str,
    font_size: Option<f64>,
    bold: bool,
    _text_len: usize,
) -> Value {
    let mut style = json!({});
    let mut fields = Vec::new();

    if let Some(size) = font_size {
        style["fontSize"] = json!({"magnitude": size, "unit": "PT"});
        fields.push("fontSize");
    }
    if bold {
        style["bold"] = json!(true);
        fields.push("bold");
    }
    // White text if fill is set (caller should decide, but we'll provide a helper)
    json!({
        "updateTextStyle": {
            "objectId": object_id,
            "style": style,
            "textRange": {
                "type": "ALL"
            },
            "fields": fields.join(",")
        }
    })
}

pub fn update_text_color_request(object_id: &str, r: f64, g: f64, b: f64) -> Value {
    json!({
        "updateTextStyle": {
            "objectId": object_id,
            "style": {
                "foregroundColor": {
                    "opaqueColor": {
                        "rgbColor": {"red": r, "green": g, "blue": b}
                    }
                }
            },
            "textRange": {
                "type": "ALL"
            },
            "fields": "foregroundColor"
        }
    })
}

pub fn create_table_request(
    object_id: &str,
    slide_id: &str,
    rows: u32,
    cols: u32,
) -> Value {
    json!({
        "createTable": {
            "objectId": object_id,
            "elementProperties": {
                "pageObjectId": slide_id
            },
            "rows": rows,
            "columns": cols
        }
    })
}

pub fn insert_table_text_request(
    table_id: &str,
    row: u32,
    col: u32,
    text: &str,
) -> Value {
    json!({
        "insertText": {
            "objectId": table_id,
            "cellLocation": {
                "rowIndex": row,
                "columnIndex": col
            },
            "text": text,
            "insertionIndex": 0
        }
    })
}

pub fn update_table_cell_fill_request(
    table_id: &str,
    row: u32,
    col: u32,
    col_span: u32,
    r: f64,
    g: f64,
    b: f64,
) -> Value {
    json!({
        "updateTableCellProperties": {
            "objectId": table_id,
            "tableRange": {
                "location": {"rowIndex": row, "columnIndex": col},
                "rowSpan": 1,
                "columnSpan": col_span
            },
            "tableCellProperties": {
                "tableCellBackgroundFill": {
                    "solidFill": {
                        "color": {
                            "rgbColor": {"red": r, "green": g, "blue": b}
                        }
                    }
                }
            },
            "fields": "tableCellBackgroundFill"
        }
    })
}

pub fn create_sheets_chart_request(
    object_id: &str,
    slide_id: &str,
    spreadsheet_id: &str,
    chart_id: u64,
    linked: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Value {
    let linking_mode = if linked { "LINKED" } else { "NOT_LINKED_IMAGE" };
    json!({
        "createSheetsChart": {
            "objectId": object_id,
            "spreadsheetId": spreadsheet_id,
            "chartId": chart_id,
            "linkingMode": linking_mode,
            "elementProperties": {
                "pageObjectId": slide_id,
                "size": {
                    "height": {"magnitude": height, "unit": "PT"},
                    "width": {"magnitude": width, "unit": "PT"}
                },
                "transform": {
                    "scaleX": 1.0,
                    "scaleY": 1.0,
                    "translateX": x,
                    "translateY": y,
                    "unit": "PT"
                }
            }
        }
    })
}

pub fn delete_object_request(object_id: &str) -> Value {
    json!({
        "deleteObject": {
            "objectId": object_id
        }
    })
}
