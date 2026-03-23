pub mod json;
pub mod table;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Json
    }
}

pub fn print_output(value: &serde_json::Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => json::print_json(value),
        OutputFormat::Table => table::print_table(value),
    }
}
