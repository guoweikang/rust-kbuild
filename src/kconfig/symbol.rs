use crate::kconfig::ast::SymbolType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub value: Option<String>,
    pub is_choice: bool,
}

pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, name: String, symbol_type: SymbolType) {
        self.symbols.entry(name.clone()).or_insert(Symbol {
            name,
            symbol_type,
            value: None,
            is_choice: false,
        });
    }

    pub fn set_value(&mut self, name: &str, value: String) {
        if let Some(symbol) = self.symbols.get_mut(name) {
            symbol.value = Some(value);
        }
    }

    pub fn get_value(&self, name: &str) -> Option<String> {
        self.symbols.get(name).and_then(|s| s.value.clone())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.symbols
            .get(name)
            .and_then(|s| s.value.as_ref())
            .map(|v| v == "y" || v == "m")
            .unwrap_or(false)
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = (&String, &Symbol)> {
        self.symbols.iter()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
