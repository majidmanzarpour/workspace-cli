use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub presentation_id: String,
    pub title: String,
    #[serde(default)]
    pub slides: Vec<Page>,
    pub page_size: Option<Size>,
    pub locale: Option<String>,
    pub revision_id: Option<String>,
    #[serde(default)]
    pub masters: Vec<Page>,
    #[serde(default)]
    pub layouts: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub object_id: String,
    pub page_type: Option<String>,
    #[serde(default)]
    pub page_elements: Vec<PageElement>,
    pub slide_properties: Option<SlideProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlideProperties {
    pub layout_object_id: Option<String>,
    pub master_object_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageElement {
    pub object_id: String,
    pub size: Option<Size>,
    pub transform: Option<Transform>,
    pub shape: Option<Shape>,
    pub table: Option<Table>,
    pub image: Option<Image>,
    pub video: Option<Video>,
    pub line: Option<Line>,
    pub word_art: Option<WordArt>,
    pub speaker_spotlight: Option<SpeakerSpotlight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub magnitude: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub translate_x: Option<f64>,
    pub translate_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub shape_type: Option<String>,
    pub text: Option<TextContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    #[serde(default)]
    pub text_elements: Vec<TextElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    pub start_index: Option<i64>,
    pub end_index: Option<i64>,
    pub paragraph_marker: Option<ParagraphMarker>,
    pub text_run: Option<TextRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphMarker {
    pub style: Option<ParagraphStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    // Simplified
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub content: Option<String>,
    pub style: Option<TextStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub font_size: Option<Dimension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub rows: i64,
    pub columns: i64,
    #[serde(default)]
    pub table_rows: Vec<TableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    pub row_height: Option<Dimension>,
    #[serde(default)]
    pub table_cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub text: Option<TextContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub content_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub source: Option<String>,
    pub id: Option<String>,
    pub video_properties: Option<VideoProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProperties {
    pub outline: Option<Outline>,
    pub auto_play: Option<bool>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub mute: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub line_properties: Option<LineProperties>,
    pub line_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineProperties {
    pub line_fill: Option<LineFill>,
    pub weight: Option<Dimension>,
    pub dash_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineFill {
    pub solid_fill: Option<SolidFill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolidFill {
    pub color: Option<Color>,
    pub alpha: Option<f64>,
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
#[serde(rename_all = "camelCase")]
pub struct Outline {
    pub outline_fill: Option<OutlineFill>,
    pub weight: Option<Dimension>,
    pub dash_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineFill {
    pub solid_fill: Option<SolidFill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordArt {
    pub rendered_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerSpotlight {
    pub speaker_spotlight_properties: Option<SpeakerSpotlightProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerSpotlightProperties {
    pub outline: Option<Outline>,
    pub shadow: Option<Shadow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    pub r#type: Option<String>,
    pub transform: Option<Transform>,
    pub alignment: Option<String>,
}
