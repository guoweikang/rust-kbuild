use crate::error::Result;
use crate::kconfig::{SymbolTable, SymbolType};
use crate::ui::events::EventResult;
use crate::ui::rendering::Theme;
use crate::ui::state::{ConfigState, ConfigValue, MenuItem, MenuItemKind, NavigationState, TristateValue};
use crate::ui::utils::FuzzySearcher;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    MenuTree,
    SearchBar,
    Dialog,
}

pub struct MenuConfigApp {
    config_state: ConfigState,
    symbol_table: SymbolTable,
    navigation: NavigationState,
    
    // Search state
    search_active: bool,
    search_query: String,
    
    // UI state
    focus: PanelFocus,
    show_help_modal: bool,
    show_save_dialog: bool,
    
    // Theme
    theme: Theme,
    
    // Status message
    status_message: Option<String>,
}

impl MenuConfigApp {
    pub fn new(entries: Vec<crate::kconfig::ast::Entry>, symbol_table: SymbolTable) -> Result<Self> {
        let mut config_state = ConfigState::build_from_entries(&entries);
        
        // Initialize values from symbol table
        for item in &mut config_state.all_items {
            if let MenuItemKind::Config { symbol_type } | MenuItemKind::MenuConfig { symbol_type } = &item.kind {
                if let Some(value) = symbol_table.get_value(&item.id) {
                    item.value = Some(Self::parse_value(&value, symbol_type));
                    config_state.original_values.insert(item.id.clone(), value.clone());
                } else {
                    // Set default value based on type
                    let default_val = match symbol_type {
                        SymbolType::Bool => ConfigValue::Bool(false),
                        SymbolType::Tristate => ConfigValue::Tristate(TristateValue::No),
                        SymbolType::String => ConfigValue::String(String::new()),
                        SymbolType::Int => ConfigValue::Int(0),
                        SymbolType::Hex => ConfigValue::Hex("0x0".to_string()),
                    };
                    item.value = Some(default_val);
                }
            }
        }
        
        Ok(Self {
            config_state,
            symbol_table,
            navigation: NavigationState::new(),
            search_active: false,
            search_query: String::new(),
            focus: PanelFocus::MenuTree,
            show_help_modal: false,
            show_save_dialog: false,
            theme: Theme::default(),
            status_message: None,
        })
    }
    
    fn parse_value(value: &str, symbol_type: &SymbolType) -> ConfigValue {
        match symbol_type {
            SymbolType::Bool => ConfigValue::Bool(value == "y"),
            SymbolType::Tristate => match value {
                "y" => ConfigValue::Tristate(TristateValue::Yes),
                "m" => ConfigValue::Tristate(TristateValue::Module),
                _ => ConfigValue::Tristate(TristateValue::No),
            },
            SymbolType::String => ConfigValue::String(value.trim_matches('"').to_string()),
            SymbolType::Int => ConfigValue::Int(value.parse().unwrap_or(0)),
            SymbolType::Hex => ConfigValue::Hex(value.to_string()),
        }
    }
    
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.handle_key(key)? {
                        EventResult::Quit => break,
                        EventResult::Continue => {}
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(3),  // Search bar
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Status bar
            ])
            .split(frame.size());
        
        self.render_header(frame, chunks[0]);
        self.render_search_bar(frame, chunks[1]);
        self.render_main_content(frame, chunks[2]);
        self.render_status_bar(frame, chunks[3]);
        
        if self.show_help_modal {
            self.render_help_modal(frame);
        }
        
        if self.show_save_dialog {
            self.render_save_dialog(frame);
        }
    }
    
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let modified_count = self.config_state.modified_symbols.len();
        let title = format!(
            " 🔧 Rust Kbuild Configuration{}{}",
            if modified_count > 0 {
                format!("  Changed: {}", modified_count)
            } else {
                String::new()
            },
            "  [S]ave [Q]uit "
        );
        
        let header = Paragraph::new(title)
            .style(self.theme.get_info_style().add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(header, area);
    }
    
    fn render_search_bar(&self, frame: &mut Frame, area: Rect) {
        let search_text = if self.search_active {
            format!(" 🔍 Search: {}_", self.search_query)
        } else {
            " 🔍 Press / to search".to_string()
        };
        
        let style = if self.search_active {
            self.theme.get_selected_style()
        } else {
            Style::default()
        };
        
        let search = Paragraph::new(search_text)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(search, area);
    }
    
    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);
        
        self.render_menu_tree(frame, chunks[0]);
        self.render_detail_panel(frame, chunks[1]);
    }
    
    fn render_menu_tree(&mut self, frame: &mut Frame, area: Rect) {
        let items = if self.search_active && !self.search_query.is_empty() {
            let searcher = FuzzySearcher::new(self.search_query.clone());
            let results = searcher.search(&self.config_state.all_items);
            results.into_iter().map(|r| r.item).collect()
        } else {
            self.config_state.get_items_for_path(&self.navigation.current_path)
        };
        
        if items.is_empty() {
            let empty = Paragraph::new("No items found")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Configuration Menu "));
            frame.render_widget(empty, area);
            return;
        }
        
        // Ensure selected index is valid
        if self.navigation.selected_index >= items.len() {
            self.navigation.selected_index = items.len().saturating_sub(1);
        }
        
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = idx == self.navigation.selected_index;
                self.create_list_item(item, is_selected)
            })
            .collect();
        
        let list = List::new(list_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Configuration Menu ")
                .border_style(if self.focus == PanelFocus::MenuTree {
                    self.theme.get_selected_style()
                } else {
                    self.theme.get_border_style()
                }));
        
        frame.render_widget(list, area);
    }
    
    fn create_list_item(&self, item: &MenuItem, is_selected: bool) -> ListItem<'_> {
        let indent = "  ".repeat(item.depth);
        let icon = self.get_item_icon(item);
        let checkbox = self.get_checkbox_symbol(item);
        let label = &item.label;
        let value_display = self.format_value_display(item);
        
        let style = if is_selected {
            self.theme.get_selected_style()
        } else if !item.is_enabled {
            self.theme.get_disabled_style()
        } else {
            Style::default()
        };
        
        let text = format!("{}{} {} {} {}", indent, icon, checkbox, label, value_display);
        ListItem::new(text).style(style)
    }
    
    fn get_item_icon(&self, item: &MenuItem) -> &str {
        match &item.kind {
            MenuItemKind::Menu { .. } => {
                if item.has_children { "📁" } else { "📂" }
            }
            MenuItemKind::Config { .. } | MenuItemKind::MenuConfig { .. } => "⚙️ ",
            MenuItemKind::Choice { .. } => "◉",
            MenuItemKind::Comment { .. } => "💬",
        }
    }
    
    fn get_checkbox_symbol(&self, item: &MenuItem) -> &str {
        match &item.value {
            Some(ConfigValue::Bool(true)) => "[✓]",
            Some(ConfigValue::Bool(false)) => "[ ]",
            Some(ConfigValue::Tristate(TristateValue::Yes)) => "[✓]",
            Some(ConfigValue::Tristate(TristateValue::No)) => "[ ]",
            Some(ConfigValue::Tristate(TristateValue::Module)) => "[M]",
            None if !item.is_enabled => "[✗]",
            _ => "   ",
        }
    }
    
    fn format_value_display(&self, item: &MenuItem) -> String {
        match &item.value {
            Some(ConfigValue::String(s)) if !s.is_empty() => format!("= \"{}\"", s),
            Some(ConfigValue::Int(i)) => format!("= {}", i),
            Some(ConfigValue::Hex(h)) => format!("= {}", h),
            _ => String::new(),
        }
    }
    
    fn render_detail_panel(&self, frame: &mut Frame, area: Rect) {
        let items = if self.search_active && !self.search_query.is_empty() {
            let searcher = FuzzySearcher::new(self.search_query.clone());
            let results = searcher.search(&self.config_state.all_items);
            results.into_iter().map(|r| r.item).collect()
        } else {
            self.config_state.get_items_for_path(&self.navigation.current_path)
        };
        
        if items.is_empty() || self.navigation.selected_index >= items.len() {
            let empty = Paragraph::new("No item selected")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" 📖 Help & Details "));
            frame.render_widget(empty, area);
            return;
        }
        
        let item = &items[self.navigation.selected_index];
        
        let mut text_lines = vec![];
        
        // Title
        text_lines.push(Line::from(vec![
            Span::styled("📖 ", self.theme.get_info_style()),
            Span::styled(&item.label, Style::default().add_modifier(Modifier::BOLD)),
        ]));
        text_lines.push(Line::from(""));
        
        // Type and ID
        let type_str = match &item.kind {
            MenuItemKind::Config { symbol_type } | MenuItemKind::MenuConfig { symbol_type } => {
                format!("Type: {:?}", symbol_type)
            }
            MenuItemKind::Menu { .. } => "Type: Menu".to_string(),
            MenuItemKind::Choice { .. } => "Type: Choice".to_string(),
            MenuItemKind::Comment { .. } => "Type: Comment".to_string(),
        };
        text_lines.push(Line::from(type_str));
        text_lines.push(Line::from(format!("ID: {}", item.id)));
        text_lines.push(Line::from(""));
        
        // Current value
        if let Some(value) = &item.value {
            let value_str = match value {
                ConfigValue::Bool(true) => "Status: ✓ Enabled".to_string(),
                ConfigValue::Bool(false) => "Status: Disabled".to_string(),
                ConfigValue::Tristate(TristateValue::Yes) => "Status: ✓ Yes".to_string(),
                ConfigValue::Tristate(TristateValue::No) => "Status: No".to_string(),
                ConfigValue::Tristate(TristateValue::Module) => "Status: Module".to_string(),
                ConfigValue::String(s) => format!("Value: \"{}\"", s),
                ConfigValue::Int(i) => format!("Value: {}", i),
                ConfigValue::Hex(h) => format!("Value: {}", h),
            };
            text_lines.push(Line::from(value_str));
            text_lines.push(Line::from(""));
        }
        
        // Help text
        if let Some(help) = &item.help_text {
            text_lines.push(Line::from("Description:"));
            text_lines.push(Line::from("━━━━━━━━━━━━"));
            // Split help text into lines
            for line in help.lines() {
                text_lines.push(Line::from(line.to_string()));
            }
            text_lines.push(Line::from(""));
        }
        
        // Dependencies
        if !item.selects.is_empty() {
            text_lines.push(Line::from("⚡ Enables:"));
            for select in &item.selects {
                text_lines.push(Line::from(format!("  • {}", select)));
            }
        }
        
        let detail = Paragraph::new(text_lines)
            .wrap(Wrap { trim: true })
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 📖 Help & Details "));
        
        frame.render_widget(detail, area);
    }
    
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status_text = if let Some(msg) = &self.status_message {
            msg.clone()
        } else {
            " ↑↓:Navigate │ Space:Toggle │ Enter:Open │ /:Search │ ?:Help │ ESC:Back".to_string()
        };
        
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(status, area);
    }
    
    fn render_help_modal(&self, frame: &mut Frame) {
        let area = self.centered_rect(60, 70, frame.size());
        
        let help_text = vec![
            "Keyboard Shortcuts",
            "══════════════════",
            "",
            "Navigation:",
            "  ↑/k        - Move up",
            "  ↓/j        - Move down",
            "  ←/h/ESC    - Go back",
            "  →/l/Enter  - Enter submenu",
            "  PageUp     - Page up",
            "  PageDown   - Page down",
            "  Home       - Jump to first",
            "  End        - Jump to last",
            "",
            "Actions:",
            "  Space      - Toggle option",
            "  s/S        - Save configuration",
            "  q/Q        - Quit",
            "  /          - Search",
            "  ?          - Show this help",
            "",
            "Press any key to close",
        ];
        
        let text: Vec<Line> = help_text.into_iter().map(Line::from).collect();
        
        let help = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(self.theme.get_info_style()));
        
        frame.render_widget(help, area);
    }
    
    fn render_save_dialog(&self, frame: &mut Frame) {
        let area = self.centered_rect(50, 30, frame.size());
        
        let text = vec![
            "Save Configuration?",
            "",
            "You have unsaved changes.",
            "",
            "  y - Save and quit",
            "  n - Quit without saving",
            "  ESC - Cancel",
        ];
        
        let lines: Vec<Line> = text.into_iter().map(Line::from).collect();
        
        let dialog = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Confirm ")
                .style(self.theme.get_warning_style()));
        
        frame.render_widget(dialog, area);
    }
    
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);
        
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> Result<EventResult> {
        // Handle modals first
        if self.show_help_modal {
            self.show_help_modal = false;
            return Ok(EventResult::Continue);
        }
        
        if self.show_save_dialog {
            return self.handle_save_dialog_key(key);
        }
        
        // Handle search mode
        if self.search_active {
            return self.handle_search_key(key);
        }
        
        // Main navigation
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if !self.config_state.modified_symbols.is_empty() {
                    self.show_save_dialog = true;
                    Ok(EventResult::Continue)
                } else {
                    Ok(EventResult::Quit)
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.save_config()?;
                Ok(EventResult::Continue)
            }
            KeyCode::Char('?') => {
                self.show_help_modal = true;
                Ok(EventResult::Continue)
            }
            KeyCode::Char('/') => {
                self.search_active = true;
                self.search_query.clear();
                self.focus = PanelFocus::SearchBar;
                Ok(EventResult::Continue)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
                Ok(EventResult::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
                Ok(EventResult::Continue)
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                self.go_back();
                Ok(EventResult::Continue)
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.enter_submenu();
                Ok(EventResult::Continue)
            }
            KeyCode::Char(' ') => {
                self.toggle_current_item()?;
                Ok(EventResult::Continue)
            }
            KeyCode::PageUp => {
                self.page_up();
                Ok(EventResult::Continue)
            }
            KeyCode::PageDown => {
                self.page_down();
                Ok(EventResult::Continue)
            }
            KeyCode::Home => {
                self.jump_to_first();
                Ok(EventResult::Continue)
            }
            KeyCode::End => {
                self.jump_to_last();
                Ok(EventResult::Continue)
            }
            _ => Ok(EventResult::Continue),
        }
    }
    
    fn handle_save_dialog_key(&mut self, key: KeyEvent) -> Result<EventResult> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.save_config()?;
                self.show_save_dialog = false;
                Ok(EventResult::Quit)
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.show_save_dialog = false;
                Ok(EventResult::Quit)
            }
            KeyCode::Esc => {
                self.show_save_dialog = false;
                Ok(EventResult::Continue)
            }
            _ => Ok(EventResult::Continue),
        }
    }
    
    fn handle_search_key(&mut self, key: KeyEvent) -> Result<EventResult> {
        match key.code {
            KeyCode::Esc => {
                self.search_active = false;
                self.search_query.clear();
                self.focus = PanelFocus::MenuTree;
                self.navigation.selected_index = 0;
                Ok(EventResult::Continue)
            }
            KeyCode::Enter => {
                self.search_active = false;
                self.focus = PanelFocus::MenuTree;
                Ok(EventResult::Continue)
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.navigation.selected_index = 0;
                Ok(EventResult::Continue)
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.navigation.selected_index = 0;
                Ok(EventResult::Continue)
            }
            _ => Ok(EventResult::Continue),
        }
    }
    
    fn move_up(&mut self) {
        if self.navigation.selected_index > 0 {
            self.navigation.selected_index -= 1;
        }
    }
    
    fn move_down(&mut self) {
        let items = if self.search_active && !self.search_query.is_empty() {
            let searcher = FuzzySearcher::new(self.search_query.clone());
            let results = searcher.search(&self.config_state.all_items);
            results.into_iter().map(|r| r.item).collect::<Vec<_>>()
        } else {
            self.config_state.get_items_for_path(&self.navigation.current_path)
        };
        
        if !items.is_empty() && self.navigation.selected_index < items.len() - 1 {
            self.navigation.selected_index += 1;
        }
    }
    
    fn enter_submenu(&mut self) {
        let items = self.config_state.get_items_for_path(&self.navigation.current_path);
        if items.is_empty() || self.navigation.selected_index >= items.len() {
            return;
        }
        
        let item = &items[self.navigation.selected_index];
        if item.has_children {
            self.navigation.current_path.push(item.id.clone());
            self.navigation.selected_index = 0;
            self.navigation.scroll_offset = 0;
        }
    }
    
    fn go_back(&mut self) {
        if !self.navigation.current_path.is_empty() {
            self.navigation.current_path.pop();
            self.navigation.selected_index = 0;
            self.navigation.scroll_offset = 0;
        }
    }
    
    fn page_up(&mut self) {
        self.navigation.selected_index = self.navigation.selected_index.saturating_sub(10);
    }
    
    fn page_down(&mut self) {
        let items = if self.search_active && !self.search_query.is_empty() {
            let searcher = FuzzySearcher::new(self.search_query.clone());
            let results = searcher.search(&self.config_state.all_items);
            results.into_iter().map(|r| r.item).collect::<Vec<_>>()
        } else {
            self.config_state.get_items_for_path(&self.navigation.current_path)
        };
        
        if !items.is_empty() {
            self.navigation.selected_index = (self.navigation.selected_index + 10).min(items.len() - 1);
        }
    }
    
    fn jump_to_first(&mut self) {
        self.navigation.selected_index = 0;
    }
    
    fn jump_to_last(&mut self) {
        let items = if self.search_active && !self.search_query.is_empty() {
            let searcher = FuzzySearcher::new(self.search_query.clone());
            let results = searcher.search(&self.config_state.all_items);
            results.into_iter().map(|r| r.item).collect::<Vec<_>>()
        } else {
            self.config_state.get_items_for_path(&self.navigation.current_path)
        };
        
        if !items.is_empty() {
            self.navigation.selected_index = items.len() - 1;
        }
    }
    
    fn toggle_current_item(&mut self) -> Result<()> {
        let items = if self.search_active && !self.search_query.is_empty() {
            let searcher = FuzzySearcher::new(self.search_query.clone());
            let results = searcher.search(&self.config_state.all_items);
            results.into_iter().map(|r| r.item).collect::<Vec<_>>()
        } else {
            self.config_state.get_items_for_path(&self.navigation.current_path)
        };
        
        if items.is_empty() || self.navigation.selected_index >= items.len() {
            return Ok(());
        }
        
        let item = &items[self.navigation.selected_index];
        let item_id = item.id.clone();
        
        // Toggle value
        let new_value = match &item.value {
            Some(ConfigValue::Bool(b)) => Some(ConfigValue::Bool(!b)),
            Some(ConfigValue::Tristate(t)) => Some(ConfigValue::Tristate(match t {
                TristateValue::No => TristateValue::Yes,
                TristateValue::Yes => TristateValue::Module,
                TristateValue::Module => TristateValue::No,
            })),
            _ => None,
        };
        
        if let Some(new_val) = new_value {
            // Update in config state
            for item in &mut self.config_state.all_items {
                if item.id == item_id {
                    item.value = Some(new_val.clone());
                    break;
                }
            }
            
            // Update in menu tree
            for (_key, items) in self.config_state.menu_tree.iter_mut() {
                for item in items {
                    if item.id == item_id {
                        item.value = Some(new_val.clone());
                        break;
                    }
                }
            }
            
            // Update symbol table
            let value_str = match new_val {
                ConfigValue::Bool(true) => "y".to_string(),
                ConfigValue::Bool(false) => "n".to_string(),
                ConfigValue::Tristate(TristateValue::Yes) => "y".to_string(),
                ConfigValue::Tristate(TristateValue::No) => "n".to_string(),
                ConfigValue::Tristate(TristateValue::Module) => "m".to_string(),
                ConfigValue::String(s) => format!("\"{}\"", s),
                ConfigValue::Int(i) => i.to_string(),
                ConfigValue::Hex(h) => h,
            };
            
            self.symbol_table.set_value_tracked(&item_id, value_str.clone());
            
            // Track modification
            let original = self.config_state.original_values.get(&item_id).cloned();
            if original.as_deref() != Some(value_str.as_str()) {
                self.config_state.modified_symbols.insert(item_id.clone(), value_str);
            } else {
                self.config_state.modified_symbols.remove(&item_id);
            }
            
            self.status_message = Some(format!(" {} toggled", item_id));
        }
        
        Ok(())
    }
    
    fn save_config(&mut self) -> Result<()> {
        use crate::config::ConfigWriter;
        use std::path::Path;
        
        ConfigWriter::write(Path::new(".config"), &self.symbol_table)?;
        
        // Clear modified symbols after save
        self.config_state.modified_symbols.clear();
        
        // Update original values
        for (name, symbol) in self.symbol_table.all_symbols() {
            if let Some(value) = &symbol.value {
                self.config_state.original_values.insert(name.clone(), value.clone());
            }
        }
        
        self.status_message = Some(" Configuration saved to .config".to_string());
        Ok(())
    }
}
