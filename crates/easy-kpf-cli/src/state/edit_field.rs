use easy_kpf_core::{ForwardType, PortForwardConfig};

/// Represents a field in the edit form with type-safe access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
  Name,
  Context,
  Namespace,
  Service,
  Ports,
  LocalInterface,
  ForwardType,
}

impl EditField {
  /// Get all fields in order
  pub const ALL: [EditField; 7] = [
    EditField::Name,
    EditField::Context,
    EditField::Namespace,
    EditField::Service,
    EditField::Ports,
    EditField::LocalInterface,
    EditField::ForwardType,
  ];

  /// Get field from index
  pub fn from_index(index: usize) -> Option<Self> {
    Self::ALL.get(index).copied()
  }

  /// Get index of this field
  #[allow(dead_code)]
  pub fn index(&self) -> usize {
    match self {
      EditField::Name => 0,
      EditField::Context => 1,
      EditField::Namespace => 2,
      EditField::Service => 3,
      EditField::Ports => 4,
      EditField::LocalInterface => 5,
      EditField::ForwardType => 6,
    }
  }

  /// Get the display name of the field
  pub fn name(&self) -> &'static str {
    match self {
      EditField::Name => "Name",
      EditField::Context => "Context",
      EditField::Namespace => "Namespace",
      EditField::Service => "Service",
      EditField::Ports => "Ports",
      EditField::LocalInterface => "Local Interface",
      EditField::ForwardType => "Type (kubectl/ssh)",
    }
  }

  /// Get the description/help text for the field
  pub fn description(&self) -> &'static str {
    match self {
      EditField::LocalInterface => "Optional, e.g. 127.0.0.2 to avoid port conflicts",
      _ => "",
    }
  }

  /// Check if this field supports autocomplete
  pub fn supports_autocomplete(&self) -> bool {
    matches!(
      self,
      EditField::Context
        | EditField::Namespace
        | EditField::Service
        | EditField::Ports
        | EditField::ForwardType
    )
  }

  /// Get the value from a config
  pub fn get_value(&self, config: &PortForwardConfig) -> String {
    match self {
      EditField::Name => config.name.clone(),
      EditField::Context => config.context.clone(),
      EditField::Namespace => config.namespace.clone(),
      EditField::Service => config.service.clone(),
      EditField::Ports => config.ports.join(", "),
      EditField::LocalInterface => config.local_interface.clone().unwrap_or_default(),
      EditField::ForwardType => match config.forward_type {
        ForwardType::Kubectl => "kubectl".to_string(),
        ForwardType::Ssh => "ssh".to_string(),
      },
    }
  }

  /// Set the value on a config
  pub fn set_value(&self, config: &mut PortForwardConfig, value: String) {
    match self {
      EditField::Name => config.name = value,
      EditField::Context => config.context = value,
      EditField::Namespace => config.namespace = value,
      EditField::Service => config.service = value,
      EditField::Ports => {
        config.ports = value
          .split(',')
          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty())
          .collect()
      }
      EditField::LocalInterface => {
        config.local_interface = if value.is_empty() { None } else { Some(value) }
      }
      EditField::ForwardType => {
        config.forward_type = if value.to_lowercase() == "ssh" {
          ForwardType::Ssh
        } else {
          ForwardType::Kubectl
        }
      }
    }
  }

  /// Check if changing this field should trigger name auto-generation
  pub fn triggers_name_generation(&self) -> bool {
    matches!(
      self,
      EditField::Service | EditField::Ports | EditField::ForwardType
    )
  }

  /// Total number of fields
  pub fn count() -> usize {
    Self::ALL.len()
  }
}
