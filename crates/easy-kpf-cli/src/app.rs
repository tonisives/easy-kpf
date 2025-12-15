use crate::kubectl::KubectlService;
use crate::state::{AutocompleteResult, AutocompleteState, EditField};
use crate::vim::VimState;
use easy_kpf_core::{
  services::{ConfigService, ProcessManager},
  ForwardType, PortForwardConfig, Result,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc;
use tui_textarea::TextArea;

/// Type alias for grouped configs by context
/// Each entry is (context_name, Vec<(visual_index, config_index, config_ref)>)
pub type ConfigsByContext<'a> = Vec<(String, Vec<(usize, usize, &'a PortForwardConfig)>)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
  Normal,
  Visual,
  Search,
  Help,
  Edit,
  Create,
  Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
  ServiceList,
  Logs,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
  pub line: String,
  pub is_stderr: bool,
}

pub struct App {
  pub mode: Mode,
  pub active_panel: Panel,
  pub configs: Vec<PortForwardConfig>,
  pub running_services: HashMap<String, u32>, // name -> pid
  pub selected_index: usize,                  // Index into visual_order
  pub visual_anchor: Option<usize>,           // Start of visual selection (when in Visual mode)
  pub search_query: String,
  pub visual_order: Vec<usize>, // Config indices in display order (grouped by context)
  pub logs: HashMap<String, Vec<LogEntry>>,
  #[allow(dead_code)]
  pub log_scroll: usize,
  pub status_message: Option<String>,
  pub should_quit: bool,
  pub config_service: ConfigService,
  pub process_manager: ProcessManager,
  pub log_receiver: Option<mpsc::Receiver<(String, LogEntry)>>,
  pub log_sender: mpsc::Sender<(String, LogEntry)>,
  // Edit mode state
  pub edit_config: Option<PortForwardConfig>,
  pub edit_original_config: Option<PortForwardConfig>, // Original config for change detection
  pub edit_field_index: usize,
  pub edit_field_value: String,
  pub edit_cursor_pos: usize, // Cursor position within edit_field_value
  pub name_manually_edited: bool,
  // Confirm mode state
  pub confirm_action: Option<ConfirmAction>,
  // Autocomplete state
  pub autocomplete: AutocompleteState,
  pub kubectl_service: KubectlService,
  pub autocomplete_rx: std_mpsc::Receiver<AutocompleteResult>,
  pub autocomplete_tx: std_mpsc::Sender<AutocompleteResult>,
  // Vim command mode state (for :w, :q commands in edit form)
  pub command_mode: bool,
  pub command_buffer: String,
  // Vim edit mode state (press 'e' to enter vim-like editing for current field)
  pub vim_textarea: Option<TextArea<'static>>,
  pub vim_state: Option<VimState>,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
  Delete(String),
  StartAll,
  StopAll,
  CancelEdit(Mode), // Stores the mode to return to if user says "No"
}

impl App {
  pub fn new() -> Result<Self> {
    let config_service = ConfigService::new()?;
    let state_file = config_service.config_dir().join("process-state.json");
    let process_manager = ProcessManager::with_state_file(state_file);
    let (log_sender, log_receiver) = mpsc::channel(1000);
    let kubectl_service = KubectlService::new(config_service.clone());
    let (autocomplete_tx, autocomplete_rx) = std_mpsc::channel();

    let mut app = Self {
      mode: Mode::Normal,
      active_panel: Panel::ServiceList,
      configs: Vec::new(),
      running_services: HashMap::new(),
      selected_index: 0,
      visual_anchor: None,
      search_query: String::new(),
      visual_order: Vec::new(),
      logs: HashMap::new(),
      log_scroll: 0,
      status_message: None,
      should_quit: false,
      config_service,
      process_manager,
      log_receiver: Some(log_receiver),
      log_sender,
      edit_config: None,
      edit_original_config: None,
      edit_field_index: 0,
      edit_field_value: String::new(),
      edit_cursor_pos: 0,
      name_manually_edited: false,
      confirm_action: None,
      autocomplete: AutocompleteState::default(),
      kubectl_service,
      autocomplete_rx,
      autocomplete_tx,
      command_mode: false,
      command_buffer: String::new(),
      vim_textarea: None,
      vim_state: None,
    };

    app.load_configs()?;
    app.sync_running_services();
    app.update_visual_order();

    Ok(app)
  }

  pub fn load_configs(&mut self) -> Result<()> {
    self.configs = self.config_service.load_port_forwards()?;
    self.update_visual_order();
    Ok(())
  }

  pub fn save_configs(&self) -> Result<()> {
    self.config_service.save_port_forwards(&self.configs)
  }

  pub fn sync_running_services(&mut self) {
    self.running_services.clear();
    if let Ok(services) = self.process_manager.get_running_services_with_pids() {
      for (name, pid) in services {
        if is_process_running(pid) {
          self.running_services.insert(name, pid);
        }
      }
    }
  }

  pub fn update_visual_order(&mut self) {
    // First, filter by search query
    let filtered: Vec<usize> = if self.search_query.is_empty() {
      (0..self.configs.len()).collect()
    } else {
      let query = self.search_query.to_lowercase();
      self
        .configs
        .iter()
        .enumerate()
        .filter(|(_, c)| {
          c.name.to_lowercase().contains(&query)
            || c.service.to_lowercase().contains(&query)
            || c.namespace.to_lowercase().contains(&query)
            || c.context.to_lowercase().contains(&query)
        })
        .map(|(i, _)| i)
        .collect()
    };

    // Group by context and flatten to get visual order
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for idx in filtered {
      if let Some(config) = self.configs.get(idx) {
        groups.entry(config.context.clone()).or_default().push(idx);
      }
    }

    // Sort groups by context name and flatten
    let mut sorted_contexts: Vec<_> = groups.keys().cloned().collect();
    sorted_contexts.sort();

    self.visual_order.clear();
    for context in sorted_contexts {
      if let Some(indices) = groups.get(&context) {
        self.visual_order.extend(indices);
      }
    }

    // Ensure selected index is valid
    if self.selected_index >= self.visual_order.len() && !self.visual_order.is_empty() {
      self.selected_index = self.visual_order.len() - 1;
    }
  }

  pub fn selected_config(&self) -> Option<&PortForwardConfig> {
    self
      .visual_order
      .get(self.selected_index)
      .and_then(|&i| self.configs.get(i))
  }

  #[allow(dead_code)]
  pub fn selected_config_mut(&mut self) -> Option<&mut PortForwardConfig> {
    self
      .visual_order
      .get(self.selected_index)
      .copied()
      .and_then(move |i| self.configs.get_mut(i))
  }

  pub fn selected_name(&self) -> Option<String> {
    self.selected_config().map(|c| c.name.clone())
  }

  #[allow(dead_code)]
  pub fn is_selected_running(&self) -> bool {
    self
      .selected_name()
      .map(|k| self.running_services.contains_key(&k))
      .unwrap_or(false)
  }

  pub fn move_selection(&mut self, delta: i32) {
    if self.visual_order.is_empty() {
      return;
    }

    let len = self.visual_order.len() as i32;
    let new_index = (self.selected_index as i32 + delta).rem_euclid(len);
    self.selected_index = new_index as usize;
  }

  pub fn select_first(&mut self) {
    self.selected_index = 0;
  }

  pub fn select_last(&mut self) {
    if !self.visual_order.is_empty() {
      self.selected_index = self.visual_order.len() - 1;
    }
  }

  pub fn enter_visual_mode(&mut self) {
    self.visual_anchor = Some(self.selected_index);
    self.mode = Mode::Visual;
  }

  pub fn exit_visual_mode(&mut self) {
    self.visual_anchor = None;
    self.mode = Mode::Normal;
  }

  /// Returns the range of visual indices that are selected (inclusive)
  /// Returns (start, end) where start <= end
  pub fn visual_selection_range(&self) -> Option<(usize, usize)> {
    self.visual_anchor.map(|anchor| {
      let start = anchor.min(self.selected_index);
      let end = anchor.max(self.selected_index);
      (start, end)
    })
  }

  /// Check if a visual index is within the visual selection
  pub fn is_in_visual_selection(&self, visual_idx: usize) -> bool {
    if self.mode != Mode::Visual {
      return false;
    }
    self
      .visual_selection_range()
      .map(|(start, end)| visual_idx >= start && visual_idx <= end)
      .unwrap_or(false)
  }

  /// Get configs in the visual selection range
  pub fn get_visual_selection_configs(&self) -> Vec<PortForwardConfig> {
    let Some((start, end)) = self.visual_selection_range() else {
      return vec![];
    };
    (start..=end)
      .filter_map(|visual_idx| {
        self
          .visual_order
          .get(visual_idx)
          .and_then(|&config_idx| self.configs.get(config_idx).cloned())
      })
      .collect()
  }

  pub fn set_status(&mut self, msg: impl Into<String>) {
    self.status_message = Some(msg.into());
  }

  #[allow(dead_code)]
  pub fn clear_status(&mut self) {
    self.status_message = None;
  }

  pub fn get_logs_for_selected(&self) -> &[LogEntry] {
    self
      .selected_name()
      .and_then(|k| self.logs.get(&k))
      .map(std::vec::Vec::as_slice)
      .unwrap_or(&[])
  }

  pub fn append_log(&mut self, name: &str, entry: LogEntry) {
    self.logs.entry(name.to_string()).or_default().push(entry);
  }

  // Group configs by context for display, preserving visual order
  pub fn configs_by_context(&self) -> ConfigsByContext<'_> {
    // Returns: Vec<(context, Vec<(visual_index, config_index, config)>)>
    let mut groups: HashMap<String, Vec<(usize, usize, &PortForwardConfig)>> = HashMap::new();

    for (visual_idx, &config_idx) in self.visual_order.iter().enumerate() {
      if let Some(config) = self.configs.get(config_idx) {
        groups
          .entry(config.context.clone())
          .or_default()
          .push((visual_idx, config_idx, config));
      }
    }

    let mut result: Vec<_> = groups.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
  }

  pub fn enter_edit_mode(&mut self) {
    if let Some(config) = self.selected_config().cloned() {
      self.edit_original_config = Some(config.clone()); // Save original for change detection
      self.edit_config = Some(config);
      self.edit_field_index = 0;
      self.edit_field_value = self.get_edit_field_value(0);
      self.edit_cursor_pos = self.edit_field_value.len();
      self.name_manually_edited = true; // Existing config has user-defined name
      self.autocomplete = AutocompleteState::default();
      self.mode = Mode::Edit;
      // Field 0 (Name) doesn't support autocomplete, so no auto-load
    }
  }

  pub fn enter_create_mode(&mut self) {
    // If there's a selected config, use its context as default
    let (default_context, default_namespace) = self
      .selected_config()
      .map(|c| (c.context.clone(), c.namespace.clone()))
      .unwrap_or_else(|| (String::new(), "default".to_string()));

    let new_config = PortForwardConfig {
      name: String::new(),
      context: default_context,
      namespace: default_namespace,
      service: String::new(),
      ports: vec![],
      local_interface: None,
      forward_type: ForwardType::Kubectl,
    };
    self.edit_original_config = Some(new_config.clone()); // Save original for change detection
    self.edit_config = Some(new_config);
    self.edit_field_index = 0;
    self.edit_field_value = String::new();
    self.edit_cursor_pos = 0;
    self.name_manually_edited = false; // New config starts with auto-generated name
    self.autocomplete = AutocompleteState::default();
    self.mode = Mode::Create;
    // Field 0 (Name) doesn't support autocomplete, so no auto-load
  }

  // Derive a config name from service and port (like the GUI does)
  pub fn derive_config_name(config: &PortForwardConfig) -> String {
    match config.forward_type {
      ForwardType::Ssh => {
        let host = config
          .service
          .split('@')
          .next_back()
          .unwrap_or(&config.service);
        let port = config
          .ports
          .first()
          .and_then(|p| p.split(':').next())
          .unwrap_or("unknown");
        format!("{}-{}", host, port)
      }
      ForwardType::Kubectl => {
        let port = config
          .ports
          .first()
          .and_then(|p| p.split(':').next())
          .unwrap_or("unknown");
        if config.service.is_empty() {
          "new-forward".to_string()
        } else {
          format!("{}-{}", config.service, port)
        }
      }
    }
  }

  // Update the auto-generated name if not manually edited
  pub fn update_auto_generated_name(&mut self) {
    if self.name_manually_edited || self.mode != Mode::Create {
      return;
    }

    if let Some(config) = &self.edit_config {
      let derived = Self::derive_config_name(config);
      if let Some(cfg) = &mut self.edit_config {
        cfg.name = derived.clone();
      }
      // Update the field value if we're currently on the name field
      if self.edit_field_index == 0 {
        self.edit_field_value = derived;
      }
    }
  }

  pub fn get_edit_field_value(&self, field_index: usize) -> String {
    let field = match EditField::from_index(field_index) {
      Some(f) => f,
      None => return String::new(),
    };
    self
      .edit_config
      .as_ref()
      .map(|c| field.get_value(c))
      .unwrap_or_default()
  }

  pub fn set_edit_field_value(&mut self, value: String) {
    let field = match EditField::from_index(self.edit_field_index) {
      Some(f) => f,
      None => return,
    };

    if let Some(config) = &mut self.edit_config {
      field.set_value(config, value);
    }

    // Update auto-generated name when service, ports, or type change
    if field.triggers_name_generation() {
      self.update_auto_generated_name();
    }
  }

  pub fn edit_field_count(&self) -> usize {
    EditField::count()
  }

  pub fn edit_field_name(&self, index: usize) -> &'static str {
    EditField::from_index(index).map(|f| f.name()).unwrap_or("")
  }

  pub fn edit_field_description(&self, index: usize) -> &'static str {
    EditField::from_index(index)
      .map(|f| f.description())
      .unwrap_or("")
  }

  pub fn current_edit_field(&self) -> Option<EditField> {
    EditField::from_index(self.edit_field_index)
  }

  pub fn next_edit_field(&mut self) {
    self.set_edit_field_value(self.edit_field_value.clone());
    self.edit_field_index = (self.edit_field_index + 1) % self.edit_field_count();
    self.edit_field_value = self.get_edit_field_value(self.edit_field_index);
    self.edit_cursor_pos = self.edit_field_value.len();
    self.autocomplete.focused = false;
    self.autocomplete.typing = false;
    // Auto-load suggestions for supported fields
    if let Some(field) = self.current_edit_field() {
      if field.supports_autocomplete() {
        self.load_autocomplete();
        // For static fields (type), sync selection immediately
        if field == EditField::ForwardType {
          self.sync_autocomplete_selection();
        }
      } else {
        self.clear_autocomplete();
      }
    }
  }

  pub fn prev_edit_field(&mut self) {
    self.set_edit_field_value(self.edit_field_value.clone());
    self.edit_field_index = if self.edit_field_index == 0 {
      self.edit_field_count() - 1
    } else {
      self.edit_field_index - 1
    };
    self.edit_field_value = self.get_edit_field_value(self.edit_field_index);
    self.edit_cursor_pos = self.edit_field_value.len();
    self.autocomplete.focused = false;
    self.autocomplete.typing = false;
    // Auto-load suggestions for supported fields
    if let Some(field) = self.current_edit_field() {
      if field.supports_autocomplete() {
        self.load_autocomplete();
        // For static fields (type), sync selection immediately
        if field == EditField::ForwardType {
          self.sync_autocomplete_selection();
        }
      } else {
        self.clear_autocomplete();
      }
    }
  }

  #[allow(dead_code)]
  pub fn focus_suggestions(&mut self) {
    if !self.get_autocomplete_suggestions().is_empty() {
      self.autocomplete.focused = true;
    }
  }

  pub fn unfocus_suggestions(&mut self) {
    self.autocomplete.focused = false;
  }

  pub fn enter_typing_mode(&mut self) {
    self.autocomplete.typing = true;
    self.edit_cursor_pos = self.edit_field_value.len();
  }

  pub fn exit_typing_mode(&mut self) {
    self.autocomplete.typing = false;
  }

  pub fn enter_command_mode(&mut self) {
    self.command_mode = true;
    self.command_buffer.clear();
  }

  pub fn exit_command_mode(&mut self) {
    self.command_mode = false;
    self.command_buffer.clear();
  }

  pub fn enter_vim_edit_mode(&mut self) {
    use crate::vim::VimMode;

    let mut textarea = TextArea::from([self.edit_field_value.as_str()]);
    textarea.set_cursor_style(VimMode::Normal.cursor_style());
    // Move cursor to end
    textarea.move_cursor(tui_textarea::CursorMove::End);

    self.vim_textarea = Some(textarea);
    self.vim_state = Some(VimState::new());
  }

  pub fn exit_vim_edit_mode(&mut self) {
    // Copy the text back from textarea
    if let Some(textarea) = &self.vim_textarea {
      let lines = textarea.lines();
      self.edit_field_value = lines.first().cloned().unwrap_or_default();
      self.edit_cursor_pos = self.edit_field_value.len();
    }
    self.vim_textarea = None;
    self.vim_state = None;
  }

  pub fn is_vim_edit_mode(&self) -> bool {
    self.vim_textarea.is_some()
  }

  // Cursor movement helpers for edit field
  pub fn cursor_left(&mut self) {
    if self.edit_cursor_pos > 0 {
      // Move back by one character (handle UTF-8 properly)
      let s = &self.edit_field_value[..self.edit_cursor_pos];
      if let Some(c) = s.chars().last() {
        self.edit_cursor_pos -= c.len_utf8();
      }
    }
  }

  pub fn cursor_right(&mut self) {
    if self.edit_cursor_pos < self.edit_field_value.len() {
      // Move forward by one character (handle UTF-8 properly)
      let s = &self.edit_field_value[self.edit_cursor_pos..];
      if let Some(c) = s.chars().next() {
        self.edit_cursor_pos += c.len_utf8();
      }
    }
  }

  pub fn insert_char(&mut self, c: char) {
    self.edit_field_value.insert(self.edit_cursor_pos, c);
    self.edit_cursor_pos += c.len_utf8();
  }

  pub fn delete_char_before_cursor(&mut self) {
    if self.edit_cursor_pos > 0 {
      let s = &self.edit_field_value[..self.edit_cursor_pos];
      if let Some(c) = s.chars().last() {
        let char_len = c.len_utf8();
        self.edit_cursor_pos -= char_len;
        self.edit_field_value.remove(self.edit_cursor_pos);
      }
    }
  }

  pub fn save_edit(&mut self) -> Result<()> {
    self.set_edit_field_value(self.edit_field_value.clone());

    if let Some(config) = self.edit_config.take() {
      if self.mode == Mode::Create {
        self.configs.push(config);
      } else if let Some(&idx) = self.visual_order.get(self.selected_index) {
        self.configs[idx] = config;
      }
      self.save_configs()?;
      self.update_visual_order();
    }

    self.mode = Mode::Normal;
    Ok(())
  }

  pub fn cancel_edit(&mut self) {
    self.edit_config = None;
    self.edit_original_config = None;
    self.autocomplete = AutocompleteState::default();
    self.mode = Mode::Normal;
  }

  /// Check if the edit form has unsaved changes compared to the original config
  pub fn has_unsaved_changes(&self) -> bool {
    let (current, original) = match (&self.edit_config, &self.edit_original_config) {
      (Some(c), Some(o)) => (c, o),
      _ => return false,
    };

    // Compare all fields
    current.name != original.name
      || current.context != original.context
      || current.namespace != original.namespace
      || current.service != original.service
      || current.ports != original.ports
      || current.local_interface != original.local_interface
      || current.forward_type != original.forward_type
  }

  pub fn get_config_file_path(&self) -> PathBuf {
    self.config_service.config_dir().join("port-forwards.yaml")
  }

  // Load autocomplete suggestions for the current field (async via background thread)
  pub fn load_autocomplete(&mut self) {
    self.autocomplete.selected_index = 0;

    // First, save the current field value to edit_config
    self.set_edit_field_value(self.edit_field_value.clone());

    let config = match &self.edit_config {
      Some(c) => c.clone(),
      None => return,
    };

    let field_index = self.edit_field_index;
    let tx = self.autocomplete_tx.clone();
    let kubectl = self.kubectl_service.clone();

    // Clear current suggestions and set loading
    match field_index {
      1 => self.autocomplete.contexts.clear(),
      2 => self.autocomplete.namespaces.clear(),
      3 => self.autocomplete.services.clear(),
      4 => self.autocomplete.ports.clear(),
      _ => return, // No async loading needed for other fields
    }
    self.autocomplete.loading = true;

    // Spawn background thread for kubectl calls
    std::thread::spawn(move || {
      let result = match field_index {
        1 => AutocompleteResult::Contexts(kubectl.get_contexts()),
        2 => {
          if !config.context.is_empty() {
            AutocompleteResult::Namespaces(kubectl.get_namespaces(&config.context))
          } else {
            AutocompleteResult::Namespaces(vec![])
          }
        }
        3 => {
          if !config.context.is_empty() && !config.namespace.is_empty() {
            AutocompleteResult::Services(kubectl.get_services(&config.context, &config.namespace))
          } else {
            AutocompleteResult::Services(vec![])
          }
        }
        4 => {
          if !config.context.is_empty()
            && !config.namespace.is_empty()
            && !config.service.is_empty()
          {
            AutocompleteResult::Ports(kubectl.get_service_ports(
              &config.context,
              &config.namespace,
              &config.service,
            ))
          } else {
            AutocompleteResult::Ports(vec![])
          }
        }
        _ => return,
      };
      let _ = tx.send(result);
    });
  }

  // Poll for autocomplete results (call this in the main loop)
  pub fn poll_autocomplete(&mut self) {
    while let Ok(result) = self.autocomplete_rx.try_recv() {
      self.autocomplete.loading = false;
      match result {
        AutocompleteResult::Contexts(v) => self.autocomplete.contexts = v,
        AutocompleteResult::Namespaces(v) => self.autocomplete.namespaces = v,
        AutocompleteResult::Services(v) => self.autocomplete.services = v,
        AutocompleteResult::Ports(v) => self.autocomplete.ports = v,
      }
      // After loading, try to select the current field value in suggestions
      self.sync_autocomplete_selection();
    }
  }

  // Get autocomplete suggestions for the current field
  pub fn get_autocomplete_suggestions(&self) -> &[String] {
    match self.edit_field_index {
      1 => &self.autocomplete.contexts,
      2 => &self.autocomplete.namespaces,
      3 => &self.autocomplete.services,
      4 => &self.autocomplete.ports,
      6 => &self.autocomplete.types,
      _ => &[],
    }
  }

  // Navigate autocomplete selection
  pub fn autocomplete_next(&mut self) {
    let suggestions = self.get_autocomplete_suggestions();
    if !suggestions.is_empty() {
      self.autocomplete.selected_index = (self.autocomplete.selected_index + 1) % suggestions.len();
    }
  }

  pub fn autocomplete_prev(&mut self) {
    let suggestions = self.get_autocomplete_suggestions();
    if !suggestions.is_empty() {
      self.autocomplete.selected_index = if self.autocomplete.selected_index == 0 {
        suggestions.len() - 1
      } else {
        self.autocomplete.selected_index - 1
      };
    }
  }

  // Accept the current autocomplete selection
  pub fn accept_autocomplete(&mut self) {
    let suggestions = self.get_autocomplete_suggestions();
    if let Some(value) = suggestions.get(self.autocomplete.selected_index) {
      self.edit_field_value = value.clone();
      self.edit_cursor_pos = self.edit_field_value.len();
    }
  }

  // Clear autocomplete state
  pub fn clear_autocomplete(&mut self) {
    self.autocomplete = AutocompleteState::default();
  }

  // Sync autocomplete selection to match current field value
  pub fn sync_autocomplete_selection(&mut self) {
    let current_value = &self.edit_field_value;
    let suggestions = self.get_autocomplete_suggestions();

    // Find the index of current value in suggestions
    if let Some(idx) = suggestions.iter().position(|s| s == current_value) {
      self.autocomplete.selected_index = idx;
    } else {
      // Value not in suggestions, keep at 0
      self.autocomplete.selected_index = 0;
    }
  }
}

fn is_process_running(pid: u32) -> bool {
  use std::process::Command;
  Command::new("kill")
    .args(["-0", &pid.to_string()])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}
