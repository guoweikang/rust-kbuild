use crate::error::Result;
use crate::kconfig::SymbolTable;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct ConfigWriter;

impl ConfigWriter {
    pub fn write(path: impl AsRef<Path>, symbols: &SymbolTable) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "#")?;
        writeln!(file, "# Automatically generated file; DO NOT EDIT.")?;
        writeln!(file, "#")?;

        for (name, symbol) in symbols.all_symbols() {
            if let Some(value) = &symbol.value {
                match value.as_str() {
                    "y" | "m" => {
                        writeln!(file, "{}={}", name, value)?;
                    }
                    "n" => {
                        writeln!(file, "# {} is not set", name)?;
                    }
                    _ => {
                        writeln!(file, "{}=\"{}\"", name, value)?;
                    }
                }
            } else {
                writeln!(file, "# {} is not set", name)?;
            }
        }

        Ok(())
    }
}
