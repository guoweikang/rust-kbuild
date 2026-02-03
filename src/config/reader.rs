use crate::error::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct ConfigReader;

impl ConfigReader {
    pub fn read(path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(path)?;
        let mut config = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Handle "# CONFIG_XXX is not set"
            if line.starts_with("# CONFIG_") && line.ends_with(" is not set") {
                let name = line
                    .trim_start_matches("# ")
                    .trim_end_matches(" is not set");
                config.insert(name.to_string(), "n".to_string());
                continue;
            }

            // Handle "CONFIG_XXX=value"
            if let Some(pos) = line.find('=') {
                let name = line[..pos].trim();
                let value = line[pos + 1..].trim();
                
                // Remove quotes from string values
                let value = value.trim_matches('"');
                
                config.insert(name.to_string(), value.to_string());
            }
        }

        Ok(config)
    }
}
