pub mod json;
pub mod table;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Table,
}

pub fn print_output(value: &serde_json::Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => json::print_json(value),
        OutputFormat::Table => table::print_table(value),
    }
}
