use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub document_id: String,
    pub title: String,
    pub body: Option<Body>,
    pub revision_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub content: Vec<StructuralElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralElement {
    pub start_index: Option<i64>,
    pub end_index: Option<i64>,
    pub paragraph: Option<Paragraph>,
    pub section_break: Option<SectionBreak>,
    pub table: Option<Table>,
    pub table_of_contents: Option<TableOfContents>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    pub content: Vec<StructuralElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub elements: Vec<ParagraphElement>,
    #[serde(rename = "paragraphStyle")]
    pub paragraph_style: Option<ParagraphStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphStyle {
    pub named_style_type: Option<String>,
    pub heading_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphElement {
    pub start_index: Option<i64>,
    pub end_index: Option<i64>,
    pub text_run: Option<TextRun>,
    pub inline_object_element: Option<InlineObjectElement>,
    pub horizontal_rule: Option<HorizontalRule>,
    pub page_break: Option<PageBreak>,
    pub column_break: Option<ColumnBreak>,
    pub equation: Option<Equation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineObjectElement {
    pub inline_object_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalRule {
    // Marker for horizontal rule
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBreak {
    // Marker for page break
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnBreak {
    // Marker for column break
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equation {
    // Marker for equation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub content: Option<String>,
    #[serde(rename = "textStyle")]
    pub text_style: Option<TextStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub small_caps: Option<bool>,
    pub font_size: Option<FontSize>,
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
    pub link: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSize {
    pub magnitude: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub rgb_color: Option<RgbColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbColor {
    pub red: Option<f64>,
    pub green: Option<f64>,
    pub blue: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionBreak {
    // Simplified - just a marker
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub rows: i64,
    pub columns: i64,
    pub table_rows: Vec<TableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    pub table_cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: Vec<StructuralElement>,
}

// Batch update types
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateRequest {
    pub requests: Vec<Request>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<InsertTextRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertTextRequest {
    pub text: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub index: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub document_id: String,
    pub replies: Vec<serde_json::Value>,
}
