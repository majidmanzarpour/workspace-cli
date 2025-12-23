use std::fs::File;
use std::io::{self, BufWriter};

use crate::output::{Formatter, OutputFormat};
use crate::error::CliError;

/// CLI execution context
pub struct CliContext {
    pub format: OutputFormat,
    pub output_file: Option<String>,
    pub fields: Option<Vec<String>>,
    pub quiet: bool,
}

impl CliContext {
    pub fn new(format: &str, output: Option<String>, fields: Option<String>, quiet: bool) -> Self {
        Self {
            format: OutputFormat::from_str(format).unwrap_or(OutputFormat::Json),
            output_file: output,
            fields: fields.map(|f| f.split(',').map(|s| s.trim().to_string()).collect()),
            quiet,
        }
    }

    /// Create a formatter for this context
    pub fn formatter(&self) -> io::Result<Formatter> {
        let mut formatter = Formatter::new(self.format);

        if let Some(ref path) = self.output_file {
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            formatter = formatter.with_writer(writer);
        }

        Ok(formatter)
    }

    /// Output a result, handling file output if specified
    pub fn output<T: serde::Serialize>(&self, value: &T) -> io::Result<()> {
        let mut formatter = self.formatter()?;
        formatter.write(value)?;
        formatter.flush()
    }

    /// Output an error in structured JSON format
    pub fn output_error(&self, error: &CliError) {
        if self.quiet {
            return;
        }
        eprintln!("{}", error.to_json());
    }

    /// Output a success message
    pub fn output_success(&self, message: &str) {
        if self.quiet {
            return;
        }
        let success = serde_json::json!({
            "status": "success",
            "message": message
        });
        println!("{}", serde_json::to_string(&success).unwrap());
    }

    /// Print info message (only if not quiet)
    pub fn info(&self, message: &str) {
        if !self.quiet {
            eprintln!("{}", message);
        }
    }
}

/// Result wrapper for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// Helper macro for handling command results
#[macro_export]
macro_rules! handle_result {
    ($ctx:expr, $result:expr) => {
        match $result {
            Ok(value) => {
                $ctx.output(&value).unwrap_or_else(|e| {
                    eprintln!("Output error: {}", e);
                });
            }
            Err(e) => {
                let cli_err = $crate::error::CliError::from(&e);
                $ctx.output_error(&cli_err);
                std::process::exit(1);
            }
        }
    };
}
