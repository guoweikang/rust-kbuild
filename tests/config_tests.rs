use rust_kbuild::config::{ConfigReader, ConfigWriter};
use rust_kbuild::kconfig::{SymbolTable, SymbolType};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_reader() {
    let config_path = "tests/fixtures/basic/expected.config";
    let config = ConfigReader::read(config_path).unwrap();

    assert_eq!(config.get("CONFIG_TEST_BOOL"), Some(&"y".to_string()));
    assert_eq!(config.get("CONFIG_TEST_STRING"), Some(&"hello".to_string()));
    assert_eq!(config.get("CONFIG_TEST_INT"), Some(&"42".to_string()));
}

#[test]
fn test_config_writer() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.config");

    let mut symbols = SymbolTable::new();
    symbols.add_symbol("CONFIG_TEST1".to_string(), SymbolType::Bool);
    symbols.set_value("CONFIG_TEST1", "y".to_string());
    symbols.add_symbol("CONFIG_TEST2".to_string(), SymbolType::String);
    symbols.set_value("CONFIG_TEST2", "value".to_string());

    ConfigWriter::write(&config_path, &symbols).unwrap();

    // Read back and verify
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("CONFIG_TEST1=y"));
    assert!(content.contains("CONFIG_TEST2=\"value\""));
}

#[test]
fn test_config_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.config");

    // Write config
    let mut symbols = SymbolTable::new();
    symbols.add_symbol("CONFIG_A".to_string(), SymbolType::Bool);
    symbols.set_value("CONFIG_A", "y".to_string());
    symbols.add_symbol("CONFIG_B".to_string(), SymbolType::Bool);
    symbols.set_value("CONFIG_B", "n".to_string());

    ConfigWriter::write(&config_path, &symbols).unwrap();

    // Read back
    let config = ConfigReader::read(&config_path).unwrap();
    assert_eq!(config.get("CONFIG_A"), Some(&"y".to_string()));
    assert_eq!(config.get("CONFIG_B"), Some(&"n".to_string()));
}
