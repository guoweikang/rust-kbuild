use crate::error::Result;
use crate::kconfig::SymbolTable;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct ConfigGenerator;

impl ConfigGenerator {
    pub fn generate_auto_conf(path: impl AsRef<Path>, symbols: &SymbolTable) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "#")?;
        writeln!(file, "# Automatically generated file; DO NOT EDIT.")?;
        writeln!(file, "#")?;

        for (name, symbol) in symbols.all_symbols() {
            if let Some(value) = &symbol.value {
                if value != "n" {
                    writeln!(file, "{}={}", name, value)?;
                }
            }
        }

        Ok(())
    }

    pub fn generate_autoconf_h(path: impl AsRef<Path>, symbols: &SymbolTable) -> Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "/*")?;
        writeln!(file, " * Automatically generated file; DO NOT EDIT.")?;
        writeln!(file, " */")?;
        writeln!(file)?;

        for (name, symbol) in symbols.all_symbols() {
            if let Some(value) = &symbol.value {
                match value.as_str() {
                    "y" => {
                        writeln!(file, "#define {} 1", name)?;
                    }
                    "m" => {
                        writeln!(file, "#define {}_MODULE 1", name)?;
                    }
                    "n" => {
                        // Don't define anything for disabled options
                    }
                    _ => {
                        writeln!(file, "#define {} \"{}\"", name, value)?;
                    }
                }
            }
        }

        Ok(())
    }
}
