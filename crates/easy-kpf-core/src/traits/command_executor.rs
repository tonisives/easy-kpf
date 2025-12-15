use crate::error::Result;
use async_trait::async_trait;

/// Output from a completed process execution
#[derive(Debug)]
pub struct ProcessOutput {
  pub stdout: Vec<u8>,
  pub stderr: Vec<u8>,
  pub success: bool,
}

/// Handle to a spawned process
#[derive(Debug)]
pub struct ProcessHandle {
  pub pid: u32,
}

/// Events emitted by a running process
#[derive(Debug)]
pub enum ProcessEvent {
  Stdout(Vec<u8>),
  Stderr(Vec<u8>),
  Error(String),
  Terminated { code: Option<i32> },
}

/// Abstraction for spawning and managing external processes.
/// This trait allows the core library to work with different process execution
/// backends (Tauri shell plugin, tokio::process, etc.)
#[async_trait]
pub trait CommandExecutor: Send + Sync {
  /// Execute a command and wait for completion
  async fn execute(
    &self,
    program: &str,
    args: &[String],
    env: &[(String, String)],
  ) -> Result<ProcessOutput>;

  /// Spawn a long-running process and return a handle plus event receiver
  async fn spawn(
    &self,
    program: &str,
    args: &[String],
    env: &[(String, String)],
  ) -> Result<(ProcessHandle, tokio::sync::mpsc::Receiver<ProcessEvent>)>;

  /// Spawn a background task for monitoring
  fn spawn_task<F>(&self, future: F)
  where
    F: std::future::Future<Output = ()> + Send + 'static;
}
