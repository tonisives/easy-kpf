use easy_kpf_core::{
  services::{ConfigService, ProcessManager},
  ForwardType, PortForwardConfig, Result,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
  Normal,
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
  pub edit_field_index: usize,
  pub edit_field_value: String,
  // Confirm mode state
  pub confirm_action: Option<ConfirmAction>,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
  Delete(String),
}

impl App {
  pub fn new() -> Result<Self> {
    let config_service = ConfigService::new()?;
    let state_file = config_service.config_dir().join("process-state.json");
    let process_manager = ProcessManager::with_state_file(state_file);
    let (log_sender, log_receiver) = mpsc::channel(1000);

    let mut app = Self {
      mode: Mode::Normal,
      active_panel: Panel::ServiceList,
      configs: Vec::new(),
      running_services: HashMap::new(),
      selected_index: 0,
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
      edit_field_index: 0,
      edit_field_value: String::new(),
      confirm_action: None,
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
      self.configs
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
      .map(|v| v.as_slice())
      .unwrap_or(&[])
  }

  pub fn append_log(&mut self, name: &str, entry: LogEntry) {
    self.logs.entry(name.to_string()).or_default().push(entry);
  }

  // Group configs by context for display, preserving visual order
  pub fn configs_by_context(&self) -> Vec<(String, Vec<(usize, usize, &PortForwardConfig)>)> {
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
      self.edit_config = Some(config);
      self.edit_field_index = 0;
      self.edit_field_value = self.get_edit_field_value(0);
      self.mode = Mode::Edit;
    }
  }

  pub fn enter_create_mode(&mut self) {
    self.edit_config = Some(PortForwardConfig {
      name: String::new(),
      context: String::new(),
      namespace: "default".to_string(),
      service: String::new(),
      ports: vec![],
      local_interface: None,
      forward_type: ForwardType::Kubectl,
    });
    self.edit_field_index = 0;
    self.edit_field_value = String::new();
    self.mode = Mode::Create;
  }

  pub fn get_edit_field_value(&self, field_index: usize) -> String {
    self
      .edit_config
      .as_ref()
      .map(|c| match field_index {
        0 => c.name.clone(),
        1 => c.context.clone(),
        2 => c.namespace.clone(),
        3 => c.service.clone(),
        4 => c.ports.join(", "),
        5 => c.local_interface.clone().unwrap_or_default(),
        6 => match c.forward_type {
          ForwardType::Kubectl => "kubectl".to_string(),
          ForwardType::Ssh => "ssh".to_string(),
        },
        _ => String::new(),
      })
      .unwrap_or_default()
  }

  pub fn set_edit_field_value(&mut self, value: String) {
    if let Some(config) = &mut self.edit_config {
      match self.edit_field_index {
        0 => config.name = value,
        1 => config.context = value,
        2 => config.namespace = value,
        3 => config.service = value,
        4 => {
          config.ports = value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
        }
        5 => config.local_interface = if value.is_empty() { None } else { Some(value) },
        6 => {
          config.forward_type = if value.to_lowercase() == "ssh" {
            ForwardType::Ssh
          } else {
            ForwardType::Kubectl
          }
        }
        _ => {}
      }
    }
  }

  pub fn edit_field_count(&self) -> usize {
    7
  }

  pub fn edit_field_name(&self, index: usize) -> &'static str {
    match index {
      0 => "Name",
      1 => "Context",
      2 => "Namespace",
      3 => "Service",
      4 => "Ports",
      5 => "Local Interface",
      6 => "Type (kubectl/ssh)",
      _ => "",
    }
  }

  pub fn next_edit_field(&mut self) {
    self.set_edit_field_value(self.edit_field_value.clone());
    self.edit_field_index = (self.edit_field_index + 1) % self.edit_field_count();
    self.edit_field_value = self.get_edit_field_value(self.edit_field_index);
  }

  pub fn prev_edit_field(&mut self) {
    self.set_edit_field_value(self.edit_field_value.clone());
    self.edit_field_index = if self.edit_field_index == 0 {
      self.edit_field_count() - 1
    } else {
      self.edit_field_index - 1
    };
    self.edit_field_value = self.get_edit_field_value(self.edit_field_index);
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
    self.mode = Mode::Normal;
  }

  pub fn get_config_file_path(&self) -> PathBuf {
    self.config_service.config_dir().join("port-forwards.yaml")
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
