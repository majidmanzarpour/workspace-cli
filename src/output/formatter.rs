use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Json,
    JsonCompact,
    Jsonl,
    Csv,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
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
}

impl Formatter {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            writer: Box::new(io::stdout()),
            first_item: true,
            csv_headers: None,
        }
    }

    pub fn with_writer<W: Write + 'static>(mut self, writer: W) -> Self {
        self.writer = Box::new(writer);
        self
    }

    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Write a single item
    pub fn write<T: Serialize>(&mut self, item: &T) -> io::Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(item)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::JsonCompact => {
                let json = serde_json::to_string(item)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Jsonl => {
                let json = serde_json::to_string(item)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Csv => {
                // CSV requires special handling - serialize as single row
                let json = serde_json::to_value(item)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                self.write_csv_row(&json)
            }
        }
    }

    /// Write multiple items as an array (JSON) or stream (JSONL/CSV)
    pub fn write_all<T: Serialize>(&mut self, items: &[T]) -> io::Result<()> {
        match self.format {
            OutputFormat::Json | OutputFormat::JsonCompact => {
                let json = if self.format == OutputFormat::Json {
                    serde_json::to_string_pretty(items)
                } else {
                    serde_json::to_string(items)
                }.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Jsonl => {
                for item in items {
                    self.write(item)?;
                }
                Ok(())
            }
            OutputFormat::Csv => {
                for item in items {
                    self.write(item)?;
                }
                Ok(())
            }
        }
    }

    /// Start streaming output (for paginated results)
    pub fn start_stream(&mut self) -> io::Result<()> {
        match self.format {
            OutputFormat::Json => write!(self.writer, "["),
            OutputFormat::JsonCompact => write!(self.writer, "["),
            _ => Ok(()),
        }
    }

    /// Write a single item in stream mode
    pub fn stream_item<T: Serialize>(&mut self, item: &T) -> io::Result<()> {
        match self.format {
            OutputFormat::Json | OutputFormat::JsonCompact => {
                if !self.first_item {
                    write!(self.writer, ",")?;
                }
                self.first_item = false;

                let json = if self.format == OutputFormat::Json {
                    // For pretty JSON in streaming mode, add newline before each item
                    let pretty = serde_json::to_string_pretty(item)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    // Indent each line for proper array formatting
                    let indented = pretty.lines()
                        .map(|line| format!("  {}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("\n{}", indented)
                } else {
                    serde_json::to_string(item)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                };
                write!(self.writer, "{}", json)
            }
            OutputFormat::Jsonl => {
                let json = serde_json::to_string(item)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(self.writer, "{}", json)
            }
            OutputFormat::Csv => {
                let json = serde_json::to_value(item)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                self.write_csv_row(&json)
            }
        }
    }

    /// End streaming output
    pub fn end_stream(&mut self) -> io::Result<()> {
        match self.format {
            OutputFormat::Json => writeln!(self.writer, "\n]"),
            OutputFormat::JsonCompact => writeln!(self.writer, "]"),
            _ => Ok(()),
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
