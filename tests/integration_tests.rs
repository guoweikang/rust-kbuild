use rust_kbuild::config::{ConfigGenerator, ConfigReader, ConfigWriter};
use rust_kbuild::kconfig::{Parser, SymbolTable, SymbolType};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_complete_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // 1. Parse Kconfig file
    let kconfig_path = PathBuf::from("tests/fixtures/basic/Kconfig");
    let srctree = PathBuf::from("tests/fixtures/basic");
    
    let mut parser = Parser::new(&kconfig_path, &srctree).unwrap();
    let ast = parser.parse().unwrap();
    
    assert_eq!(ast.entries.len(), 3);
    
    // 2. Build symbol table
    let mut symbols = SymbolTable::new();
    symbols.add_symbol("CONFIG_TEST_BOOL".to_string(), SymbolType::Bool);
    symbols.set_value("CONFIG_TEST_BOOL", "y".to_string());
    symbols.add_symbol("CONFIG_TEST_STRING".to_string(), SymbolType::String);
    symbols.set_value("CONFIG_TEST_STRING", "hello".to_string());
    symbols.add_symbol("CONFIG_TEST_INT".to_string(), SymbolType::Int);
    symbols.set_value("CONFIG_TEST_INT", "42".to_string());
    
    // 3. Write .config
    let config_path = temp_dir.path().join(".config");
    ConfigWriter::write(&config_path, &symbols).unwrap();
    
    // 4. Read .config back
    let config = ConfigReader::read(&config_path).unwrap();
    assert_eq!(config.get("CONFIG_TEST_BOOL"), Some(&"y".to_string()));
    assert_eq!(config.get("CONFIG_TEST_STRING"), Some(&"hello".to_string()));
    assert_eq!(config.get("CONFIG_TEST_INT"), Some(&"42".to_string()));
    
    // 5. Generate auto.conf
    let auto_conf_path = temp_dir.path().join("auto.conf");
    ConfigGenerator::generate_auto_conf(&auto_conf_path, &symbols).unwrap();
    
    let auto_conf = std::fs::read_to_string(&auto_conf_path).unwrap();
    assert!(auto_conf.contains("CONFIG_TEST_BOOL=y"));
    assert!(auto_conf.contains("CONFIG_TEST_STRING=hello"));
    assert!(auto_conf.contains("CONFIG_TEST_INT=42"));
    
    // 6. Generate autoconf.h
    let autoconf_h_path = temp_dir.path().join("autoconf.h");
    ConfigGenerator::generate_autoconf_h(&autoconf_h_path, &symbols).unwrap();
    
    let autoconf_h = std::fs::read_to_string(&autoconf_h_path).unwrap();
    assert!(autoconf_h.contains("#define CONFIG_TEST_BOOL 1"));
    assert!(autoconf_h.contains("#define CONFIG_TEST_STRING \"hello\""));
    assert!(autoconf_h.contains("#define CONFIG_TEST_INT \"42\""));
}

#[test]
fn test_source_recursion_workflow() {
    // Parse a project with nested source directives
    let kconfig_path = PathBuf::from("examples/sample_project/Kconfig");
    let srctree = PathBuf::from("examples/sample_project");
    
    let mut parser = Parser::new(&kconfig_path, &srctree).unwrap();
    let ast = parser.parse().unwrap();
    
    // Should successfully parse all entries from main and sourced files
    assert!(ast.entries.len() >= 6);
    
    // Verify we got entries from sourced files
    let has_x86 = ast.entries.iter().any(|entry| match entry {
        rust_kbuild::kconfig::Entry::Menu(menu) => {
            menu.entries.iter().any(|e| match e {
                rust_kbuild::kconfig::Entry::Config(config) => config.name == "X86",
                _ => false,
            })
        }
        _ => false,
    });
    assert!(has_x86);
}
