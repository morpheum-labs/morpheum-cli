use crate::config::OutputFormat;
use crate::error::CliError;
use colored::Colorize;
use serde::Serialize;
use std::io;
use tabled::{settings::Style, Table, Tabled};

/// Centralized output handler for the entire Morpheum CLI.
///
/// **Single responsibility**: format and render data consistently according to the
/// configured `OutputFormat` (Table for humans, JSON for machines).
#[derive(Debug, Clone)]
pub struct Output {
    format: OutputFormat,
}

#[allow(clippy::unused_self)]
impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Prints a list of items as a rich table (human mode) or pretty JSON (machine mode).
    pub fn print_list<T>(&self, items: &[T]) -> io::Result<()>
    where
        T: Tabled + Serialize,
    {
        match self.format {
            OutputFormat::Table => self.print_table(items),
            OutputFormat::Json => self.print_json(items),
        }
    }

    /// Prints a single item as a table row or JSON object.
    #[allow(dead_code)]
    pub fn print_item<T>(&self, item: &T) -> io::Result<()>
    where
        T: Tabled + Serialize,
    {
        match self.format {
            OutputFormat::Table => {
                let table = Table::new(vec![item])
                    .with(Style::modern())
                    .to_string();
                println!("{table}");
                Ok(())
            }
            OutputFormat::Json => self.print_json(item),
        }
    }

    fn print_json<T: Serialize + ?Sized>(&self, value: &T) -> io::Result<()> {
        let json = serde_json::to_string_pretty(value)
            .map_err(io::Error::other)?;
        println!("{json}");
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn print_table<T: Tabled>(&self, items: &[T]) -> io::Result<()> {
        if items.is_empty() {
            println!("(no results)");
            return Ok(());
        }

        let table = Table::new(items)
            .with(Style::modern())
            .to_string();

        println!("{table}");
        Ok(())
    }

    pub fn success(&self, message: impl AsRef<str>) {
        println!("{} {}", "✓".green().bold(), message.as_ref().green());
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        eprintln!("{} {}", "⚠".yellow().bold(), message.as_ref().yellow());
    }

    pub fn info(&self, message: impl AsRef<str>) {
        println!("{} {}", "ℹ".blue().bold(), message.as_ref());
    }

    /// Renders an error using rich miette diagnostics.
    #[allow(dead_code)]
    pub fn error(&self, error: &CliError) {
        eprintln!("{error}");
    }
}
