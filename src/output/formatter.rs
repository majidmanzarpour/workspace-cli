use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Toon,
    Json,
    JsonCompact,
    Jsonl,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "toon" => Some(Self::Toon),
            "json" => Some(Self::Json),
            "json-compact" | "jsoncompact" => Some(Self::JsonCompact),
            "jsonl" | "ndjson" => Some(Self::Jsonl),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

pub struct Formatter {
    format: OutputFormat,
    writer: Box<dyn Write>,
    first_item: bool,
    csv_headers: Option<Vec<String>>,
    fields: Option<Vec<String>>,
    quiet: bool,
}

impl Formatter {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            writer: Box::new(io::stdout()),
            first_item: true,
            csv_headers: None,
            fields: None,
            quiet: false,
        }
    }

    pub fn with_writer<W: Write + 'static>(mut self, writer: W) -> Self {
        self.writer = Box::new(writer);
        self
    }

    /// Set field filtering - only include these fields in output
    pub fn with_fields(mut self, fields: Option<Vec<String>>) -> Self {
        self.fields = fields;
        self
    }

    /// Set quiet mode - suppress all output
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Remove duplicate string values within the same object level (TOON dedup).
    /// If multiple fields have the same string value, only the first is kept.
    /// Applied recursively to nested objects and arrays.
    fn dedup_string_values(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut seen_strings = std::collections::HashSet::new();
                let mut deduped = serde_json::Map::new();
                for (key, val) in map {
                    let val = Self::dedup_string_values(val);
                    if let serde_json::Value::String(ref s) = val {
                        if !s.is_empty() && !seen_strings.insert(s.clone()) {
                            continue; // skip duplicate string value
                        }
                    }
                    deduped.insert(key, val);
                }
                serde_json::Value::Object(deduped)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(Self::dedup_string_values).collect())
            }
            other => other,
        }
    }

    /// Filter a JSON value to only include specified fields
    fn filter_fields(&self, value: serde_json::Value) -> serde_json::Value {
        let fields = match &self.fields {
            Some(f) if !f.is_empty() => f,
            _ => return value,
        };

        match value {
            serde_json::Value::Object(map) => {
                // Check for known list wrapper keys (API responses wrap arrays)
                const WRAPPER_KEYS: &[&str] = &["files", "messages", "items", "labels", "permissions"];

                // Find if this is a wrapper object with an array to filter
                for wrapper_key in WRAPPER_KEYS {
                    if let Some(serde_json::Value::Array(arr)) = map.get(*wrapper_key) {
                        // This is a list wrapper - filter the array items
                        let filtered_items: Vec<serde_json::Value> = arr.iter()
                            .map(|item| self.filter_item_fields(item.clone(), fields))
                            .collect();

                        // Reconstruct wrapper with filtered items + metadata
                        let mut result = serde_json::Map::new();
                        result.insert(wrapper_key.to_string(), serde_json::Value::Array(filtered_items));

                        // Preserve metadata keys (nextPageToken, resultSizeEstimate, etc.)
                        for (key, val) in map.iter() {
                            if key != *wrapper_key {
                                result.insert(key.clone(), val.clone());
                            }
                        }
                        return serde_json::Value::Object(result);
                    }
                }

                // Not a wrapper - filter as single object
                self.filter_item_fields(serde_json::Value::Object(map), fields)
            }
            serde_json::Value::Array(arr) => {
                // Filter each item in the array
                serde_json::Value::Array(
                    arr.into_iter()
                        .map(|item| self.filter_item_fields(item, fields))
                        .collect()
                )
            }
            _ => value,
        }
    }

    /// Filter fields from an individual item (not a wrapper)
    fn filter_item_fields(&self, value: serde_json::Value, fields: &[String]) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut filtered = serde_json::Map::new();
                for field in fields {
                    // Handle nested fields like "payload.headers"
                    let parts: Vec<&str> = field.split('.').collect();
                    if let Some(val) = Self::get_nested_value(&serde_json::Value::Object(map.clone()), &parts) {
                        if parts.len() == 1 {
                            filtered.insert(parts[0].to_string(), val);
                        } else {
                            // For nested fields, reconstruct the path
                            Self::set_nested_value(&mut filtered, &parts, val);
                        }
                    }
                }
                serde_json::Value::Object(filtered)
            }
            _ => value,
        }
    }

    /// Get a nested value from a JSON object
    fn get_nested_value(value: &serde_json::Value, parts: &[&str]) -> Option<serde_json::Value> {
        if parts.is_empty() {
            return Some(value.clone());
        }

        match value {
            serde_json::Value::Object(map) => {
                map.get(parts[0]).and_then(|v| {
                    if parts.len() == 1 {
                        Some(v.clone())
                    } else {
                        Self::get_nested_value(v, &parts[1..])
                    }
                })
            }
            _ => None,
        }
    }

    /// Set a nested value in a JSON map
    fn set_nested_value(map: &mut serde_json::Map<String, serde_json::Value>, parts: &[&str], value: serde_json::Value) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
        } else {
            let child = map.entry(parts[0].to_string())
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            if let serde_json::Value::Object(child_map) = child {
                Self::set_nested_value(child_map, &parts[1..], value);
            }
        }
    }

    /// Write a single item
    pub fn write<T: Serialize>(&mut self, item: &T) -> io::Result<()> {
        // Quiet mode: suppress all output
        if self.quiet {
            return Ok(());
        }

        // Convert to JSON value for field filtering
        let value = serde_json::to_value(item)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let filtered = self.filter_fields(value);

        match self.format {
            OutputFormat::Toon => {
                let deduped = Self::dedup_string_values(filtered);
                let toon = toon_format::encode(&deduped, &toon_format::EncodeOptions::default())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                writeln!(self.writer, "{}", toon)
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&filtered)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::JsonCompact => {
                let json = serde_json::to_string(&filtered)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Jsonl => {
                let json = serde_json::to_string(&filtered)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Csv => {
                self.write_csv_row(&filtered)
            }
        }
    }

    /// Write multiple items as an array (JSON) or stream (JSONL/CSV)
    pub fn write_all<T: Serialize>(&mut self, items: &[T]) -> io::Result<()> {
        // Quiet mode: suppress all output
        if self.quiet {
            return Ok(());
        }

        match self.format {
            OutputFormat::Toon => {
                // Convert to JSON value for field filtering, then encode as TOON
                let value = serde_json::to_value(items)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let filtered = self.filter_fields(value);
                let deduped = Self::dedup_string_values(filtered);
                let toon = toon_format::encode(&deduped, &toon_format::EncodeOptions::default())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                writeln!(self.writer, "{}", toon)
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                // Convert to JSON value for field filtering
                let value = serde_json::to_value(items)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let filtered = self.filter_fields(value);

                let json = if self.format == OutputFormat::Json {
                    serde_json::to_string_pretty(&filtered)
                } else {
                    serde_json::to_string(&filtered)
                }.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Jsonl | OutputFormat::Csv => {
                for item in items {
                    self.write(item)?;
                }
                Ok(())
            }
        }
    }

    /// Start streaming output (for paginated results)
    pub fn start_stream(&mut self) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        match self.format {
            OutputFormat::Json => write!(self.writer, "["),
            OutputFormat::JsonCompact => write!(self.writer, "["),
            OutputFormat::Toon | OutputFormat::Jsonl | OutputFormat::Csv => Ok(()),
        }
    }

    /// Write a single item in stream mode
    pub fn stream_item<T: Serialize>(&mut self, item: &T) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        // Convert to JSON value for field filtering
        let value = serde_json::to_value(item)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let filtered = self.filter_fields(value);

        match self.format {
            OutputFormat::Toon => {
                let deduped = Self::dedup_string_values(filtered);
                let toon = toon_format::encode(&deduped, &toon_format::EncodeOptions::default())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                writeln!(self.writer, "{}", toon)
            }
            OutputFormat::Json | OutputFormat::JsonCompact => {
                if !self.first_item {
                    write!(self.writer, ",")?;
                }
                self.first_item = false;

                let json = if self.format == OutputFormat::Json {
                    // For pretty JSON in streaming mode, add newline before each item
                    let pretty = serde_json::to_string_pretty(&filtered)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    // Indent each line for proper array formatting
                    let indented = pretty.lines()
                        .map(|line| format!("  {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("\n{}", indented)
                } else {
                    serde_json::to_string(&filtered)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                };
                write!(self.writer, "{}", json)
            }
            OutputFormat::Jsonl => {
                let json = serde_json::to_string(&filtered)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Csv => {
                self.write_csv_row(&filtered)
            }
        }
    }

    /// End streaming output
    pub fn end_stream(&mut self) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        match self.format {
            OutputFormat::Json => writeln!(self.writer, "\n]"),
            OutputFormat::JsonCompact => writeln!(self.writer, "]"),
            OutputFormat::Toon | OutputFormat::Jsonl | OutputFormat::Csv => Ok(()),
        }
    }

    fn write_csv_row(&mut self, value: &serde_json::Value) -> io::Result<()> {
        match value {
            serde_json::Value::Object(map) => {
                // Write header if first item and store the header order
                if self.first_item {
                    let headers: Vec<String> = map.keys().map(|s| s.to_string()).collect();
                    writeln!(self.writer, "{}", headers.join(","))?;
                    self.csv_headers = Some(headers);
                    self.first_item = false;
                }

                // Write values in the same order as headers
                if let Some(ref headers) = self.csv_headers {
                    let values: Vec<String> = headers.iter()
                        .map(|key| {
                            map.get(key)
                                .map(|v| self.csv_escape(v))
                                .unwrap_or_default()
                        })
                        .collect();
                    writeln!(self.writer, "{}", values.join(","))
                } else {
                    // Fallback if headers not set (shouldn't happen)
                    let values: Vec<String> = map.values()
                        .map(|v| self.csv_escape(v))
                        .collect();
                    writeln!(self.writer, "{}", values.join(","))
                }
            }
            serde_json::Value::Array(arr) => {
                // Write header if first item
                if self.first_item {
                    let headers: Vec<String> = (0..arr.len()).map(|i| format!("col{}", i)).collect();
                    writeln!(self.writer, "{}", headers.join(","))?;
                    self.first_item = false;
                }
                let values: Vec<String> = arr.iter()
                    .map(|v| self.csv_escape(v))
                    .collect();
                writeln!(self.writer, "{}", values.join(","))
            }
            _ => {
                writeln!(self.writer, "{}", self.csv_escape(value))
            }
        }
    }

    fn csv_escape(&self, value: &serde_json::Value) -> String {
        let s = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => String::new(),
            _ => value.to_string(),
        };
        // Escape quotes and wrap in quotes if contains comma, quote, or newline
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s
        }
    }

    /// Flush the writer
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Convenience function to output a single result
pub fn output_json<T: Serialize>(item: &T) -> io::Result<()> {
    let mut formatter = Formatter::new(OutputFormat::Json);
    formatter.write(item)
}

/// Convenience function to output as JSONL
pub fn output_jsonl<T: Serialize>(item: &T) -> io::Result<()> {
    let mut formatter = Formatter::new(OutputFormat::Jsonl);
    formatter.write(item)
}

/// Convenience function to output as TOON
pub fn output_toon<T: Serialize>(item: &T) -> io::Result<()> {
    let mut formatter = Formatter::new(OutputFormat::Toon);
    formatter.write(item)
}
