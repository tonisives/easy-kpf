use async_trait::async_trait;
use easy_kpf_core::{
  traits::{CommandExecutor, ProcessEvent, ProcessHandle, ProcessOutput},
  AppError, Result,
};
use std::process::Stdio;
use tokio::{
  io::{AsyncBufReadExt, BufReader},
  process::Command,
  sync::mpsc,
};

pub struct TokioCommandExecutor;

impl TokioCommandExecutor {
  pub fn new() -> Self {
    Self
  }
}

impl Default for TokioCommandExecutor {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl CommandExecutor for TokioCommandExecutor {
  async fn execute(
    &self,
    program: &str,
    args: &[String],
    env: &[(String, String)],
  ) -> Result<ProcessOutput> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    for (key, value) in env {
      cmd.env(key, value);
    }

    let output = cmd
      .output()
      .await
      .map_err(|e| AppError::Process(format!("Failed to execute {}: {}", program, e)))?;

    Ok(ProcessOutput {
      stdout: output.stdout,
      stderr: output.stderr,
      success: output.status.success(),
    })
  }

  async fn spawn(
    &self,
    program: &str,
    args: &[String],
    env: &[(String, String)],
  ) -> Result<(ProcessHandle, mpsc::Receiver<ProcessEvent>)> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    for (key, value) in env {
      cmd.env(key, value);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
      .spawn()
      .map_err(|e| AppError::Process(format!("Failed to spawn {}: {}", program, e)))?;

    let pid = child.id().unwrap_or(0);
    let handle = ProcessHandle { pid };

    let (tx, rx) = mpsc::channel(100);

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Spawn task to read stdout
    if let Some(stdout) = stdout {
      let tx = tx.clone();
      tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
          let _ = tx.send(ProcessEvent::Stdout(line.into_bytes())).await;
        }
      });
    }

    // Spawn task to read stderr
    if let Some(stderr) = stderr {
      let tx = tx.clone();
      tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
          let _ = tx.send(ProcessEvent::Stderr(line.into_bytes())).await;
        }
      });
    }

    // Spawn task to wait for process exit
    tokio::spawn(async move {
      let status = child.wait().await;
      let code = status.ok().and_then(|s| s.code());
      let _ = tx.send(ProcessEvent::Terminated { code }).await;
    });

    Ok((handle, rx))
  }

  fn spawn_task<F>(&self, future: F)
  where
    F: std::future::Future<Output = ()> + Send + 'static,
  {
    tokio::spawn(future);
  }
}
