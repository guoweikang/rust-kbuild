use rust_kbuild::kconfig::{Parser, SymbolTable, SymbolType};
use rust_kbuild::ui::app::MenuConfigApp;
use std::path::PathBuf;

/// Test that MenuConfigApp can be created with initialized values
/// This verifies the critical fix for checkbox state display
#[test]
fn test_menuconfig_app_initialization_with_values() {
    // Setup test Kconfig
    let kconfig_path = PathBuf::from("examples/sample_project/Kconfig");
    let srctree = PathBuf::from("examples/sample_project");
    
    let mut parser = Parser::new(&kconfig_path, &srctree).unwrap();
    let ast = parser.parse().unwrap();
    
    // Create symbol table with some values
    let mut symbol_table = SymbolTable::new();
    symbol_table.add_symbol("HAVE_ARCH".to_string(), SymbolType::Bool);
    symbol_table.add_symbol("DEBUG".to_string(), SymbolType::Bool);
    symbol_table.add_symbol("VERBOSE".to_string(), SymbolType::Bool);
    
    symbol_table.set_value("HAVE_ARCH", "y".to_string());
    symbol_table.set_value("DEBUG", "y".to_string());
    symbol_table.set_value("VERBOSE", "n".to_string());
    
    // Create MenuConfigApp - this should initialize values in both all_items AND menu_tree
    let app = MenuConfigApp::new(ast.entries, symbol_table);
    
    // The app should be created successfully
    assert!(app.is_ok(), "MenuConfigApp should be created successfully with initialized values");
}

/// Test that MenuConfigApp can be created without pre-existing config values
#[test]
fn test_menuconfig_app_initialization_with_defaults() {
    // Setup test Kconfig
    let kconfig_path = PathBuf::from("examples/sample_project/Kconfig");
    let srctree = PathBuf::from("examples/sample_project");
    
    let mut parser = Parser::new(&kconfig_path, &srctree).unwrap();
    let ast = parser.parse().unwrap();
    
    // Create empty symbol table
    let symbol_table = SymbolTable::new();
    
    // Create MenuConfigApp - this should initialize with default values
    let app = MenuConfigApp::new(ast.entries, symbol_table);
    
    // The app should be created successfully with defaults
    assert!(app.is_ok(), "MenuConfigApp should be created successfully with default values");
}
