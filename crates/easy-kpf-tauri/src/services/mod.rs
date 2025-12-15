// Tauri-specific services (these depend on tauri::AppHandle)
pub mod kubectl_service;
pub mod port_forward_service;

pub use kubectl_service::{KubectlOperations, KubectlService};
pub use port_forward_service::PortForwardService;

// Re-export core services used by handlers
pub use easy_kpf_core::services::ConfigService;
