use std::path::PathBuf;

pub fn default_socket_path() -> PathBuf {
  dirs::config_dir()
    .unwrap_or_else(|| PathBuf::from("/tmp"))
    .join("EasyKpf")
    .join("kpfctl.sock")
}
