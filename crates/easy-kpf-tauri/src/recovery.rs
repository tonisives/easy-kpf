use easy_kpf_core::error::{AppError, Result};
use easy_kpf_core::types::{HookCommand, HookType, ReconnectPolicy};
use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tokio::process::Command;

#[derive(Clone, Default)]
pub struct RecoveryCoordinator {
  state: Arc<Mutex<RecoveryState>>,
}

#[derive(Default)]
struct RecoveryState {
  active: HashSet<String>,
  attempts: HashMap<String, u32>,
  generations: HashMap<String, u64>,
  last_hook_runs: HashMap<String, Instant>,
}

#[derive(Clone)]
pub struct RecoveryTicket {
  service_name: String,
  generation: u64,
}

pub struct RecoveryAttempt {
  pub number: u32,
  pub delay: Duration,
}

pub struct HookContext<'a> {
  pub service_name: &'a str,
  pub error: &'a str,
  pub attempt: u32,
}

#[derive(Clone, Copy)]
pub enum HookEvent {
  Failure,
  BeforeReconnect,
  Recovered,
}

impl HookEvent {
  fn as_str(self) -> &'static str {
    match self {
      Self::Failure => "failure",
      Self::BeforeReconnect => "before_reconnect",
      Self::Recovered => "recovered",
    }
  }
}

impl RecoveryCoordinator {
  pub fn begin(&self, service_name: &str) -> Result<Option<RecoveryTicket>> {
    let mut state = self.lock_state()?;
    if !state.active.insert(service_name.to_string()) {
      return Ok(None);
    }

    let generation = state
      .generations
      .entry(service_name.to_string())
      .and_modify(|value| *value = value.saturating_add(1))
      .or_insert(1);

    Ok(Some(RecoveryTicket {
      service_name: service_name.to_string(),
      generation: *generation,
    }))
  }

  pub fn next_attempt(
    &self,
    ticket: &RecoveryTicket,
    policy: &ReconnectPolicy,
  ) -> Result<Option<RecoveryAttempt>> {
    let mut state = self.lock_state()?;
    if !is_current(&state, ticket) {
      return Ok(None);
    }

    let attempt = state
      .attempts
      .entry(ticket.service_name.clone())
      .and_modify(|value| *value = value.saturating_add(1))
      .or_insert(1);

    if policy.max_attempts > 0 && *attempt > policy.max_attempts {
      state.active.remove(&ticket.service_name);
      return Ok(None);
    }

    Ok(Some(RecoveryAttempt {
      number: *attempt,
      delay: reconnect_delay(policy, *attempt),
    }))
  }

  pub fn finish(&self, ticket: &RecoveryTicket) -> Result<()> {
    let mut state = self.lock_state()?;
    if is_current(&state, ticket) {
      state.active.remove(&ticket.service_name);
    }
    Ok(())
  }

  pub fn is_active(&self, ticket: &RecoveryTicket) -> Result<bool> {
    let state = self.lock_state()?;
    Ok(is_current(&state, ticket))
  }

  pub fn mark_stable(&self, ticket: &RecoveryTicket) -> Result<bool> {
    let mut state = self.lock_state()?;
    if !is_current_generation(&state, ticket) {
      return Ok(false);
    }
    state.attempts.remove(&ticket.service_name);
    Ok(true)
  }

  pub fn cancel(&self, service_name: &str) -> Result<bool> {
    let mut state = self.lock_state()?;
    let recovery_was_active = state.active.remove(service_name);
    state.attempts.remove(service_name);
    state
      .generations
      .entry(service_name.to_string())
      .and_modify(|value| *value = value.saturating_add(1))
      .or_insert(1);
    Ok(recovery_was_active)
  }

  pub fn cancel_all(&self) -> Result<()> {
    let mut state = self.lock_state()?;
    state.active.clear();
    state.attempts.clear();
    for generation in state.generations.values_mut() {
      *generation = generation.saturating_add(1);
    }
    Ok(())
  }

  pub async fn run_hooks(
    &self,
    event: HookEvent,
    hooks: &[HookCommand],
    context: &HookContext<'_>,
  ) {
    for hook in hooks {
      match self.reserve_hook(event, hook) {
        Ok(true) => {
          let _ = run_hook(event, hook, context).await;
        }
        Ok(false) => {
          log::debug!(
            "[{}] Skipping {} hook during cooldown",
            context.service_name,
            event.as_str()
          );
        }
        Err(error) => {
          log::warn!(
            "[{}] Could not schedule {} hook: {}",
            context.service_name,
            event.as_str(),
            error
          );
        }
      }
    }
  }

  fn reserve_hook(&self, event: HookEvent, hook: &HookCommand) -> Result<bool> {
    let key = hook_key(event, hook);
    let now = Instant::now();
    let mut state = self.lock_state()?;
    if state.last_hook_runs.get(&key).is_some_and(|last_run| {
      now.duration_since(*last_run) < Duration::from_secs(hook.cooldown_seconds)
    }) {
      return Ok(false);
    }
    state.last_hook_runs.insert(key, now);
    Ok(true)
  }

  fn lock_state(&self) -> Result<MutexGuard<'_, RecoveryState>> {
    self
      .state
      .lock()
      .map_err(|_| AppError::System("Recovery state lock was poisoned".to_string()))
  }
}

fn is_current(state: &RecoveryState, ticket: &RecoveryTicket) -> bool {
  state.active.contains(&ticket.service_name) && is_current_generation(state, ticket)
}

fn is_current_generation(state: &RecoveryState, ticket: &RecoveryTicket) -> bool {
  state.generations.get(&ticket.service_name) == Some(&ticket.generation)
}

fn reconnect_delay(policy: &ReconnectPolicy, attempt: u32) -> Duration {
  let exponent = attempt.saturating_sub(1).min(31);
  let multiplier = 1_u64 << exponent;
  let delay_ms = policy.initial_delay_ms.saturating_mul(multiplier);
  let maximum_delay_ms = policy.max_delay_ms.max(policy.initial_delay_ms);
  Duration::from_millis(delay_ms.min(maximum_delay_ms))
}

fn hook_key(event: HookEvent, hook: &HookCommand) -> String {
  format!(
    "{}\u{0}{:?}\u{0}{}\u{0}{}\u{0}{}",
    event.as_str(),
    hook.kind,
    hook.ssh_host.as_deref().unwrap_or_default(),
    hook.command,
    hook.args.join("\u{0}")
  )
}

async fn run_hook(event: HookEvent, hook: &HookCommand, context: &HookContext<'_>) -> bool {
  if hook.command.trim().is_empty() {
    log::warn!(
      "[{}] Ignoring empty {} hook command",
      context.service_name,
      event.as_str()
    );
    return false;
  }

  log::info!(
    "[{}] Running {} hook: {}",
    context.service_name,
    event.as_str(),
    hook.command
  );

  let Some((program, args)) = hook_invocation(hook) else {
    log::warn!(
      "[{}] Ignoring {} SSH hook without a host",
      context.service_name,
      event.as_str()
    );
    return false;
  };
  let mut command = Command::new(program);
  command
    .args(args)
    .env("EASY_KPF_EVENT", event.as_str())
    .env("EASY_KPF_SERVICE", context.service_name)
    .env("EASY_KPF_ERROR", context.error)
    .env("EASY_KPF_ATTEMPT", context.attempt.to_string())
    .stdin(Stdio::null())
    .kill_on_drop(true);

  match tokio::time::timeout(
    Duration::from_secs(hook.timeout_seconds.max(1)),
    command.output(),
  )
  .await
  {
    Ok(Ok(output)) if output.status.success() => {
      log::info!(
        "[{}] {} hook completed successfully",
        context.service_name,
        event.as_str()
      );
      true
    }
    Ok(Ok(output)) => {
      log::warn!(
        "[{}] {} hook exited with status {}",
        context.service_name,
        event.as_str(),
        output.status
      );
      false
    }
    Ok(Err(error)) => {
      log::warn!(
        "[{}] Failed to run {} hook: {}",
        context.service_name,
        event.as_str(),
        error
      );
      false
    }
    Err(_) => {
      log::warn!(
        "[{}] {} hook timed out after {} seconds",
        context.service_name,
        event.as_str(),
        hook.timeout_seconds
      );
      false
    }
  }
}

fn hook_invocation(hook: &HookCommand) -> Option<(String, Vec<String>)> {
  match hook.kind {
    HookType::Command => Some((hook.command.clone(), hook.args.clone())),
    HookType::Ssh => {
      let ssh_host = hook
        .ssh_host
        .as_deref()
        .filter(|host| !host.trim().is_empty())?;
      let mut args = vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=10".to_string(),
        ssh_host.to_string(),
        "--".to_string(),
        hook.command.clone(),
      ];
      args.extend(hook.args.clone());
      Some(("ssh".to_string(), args))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{hook_invocation, run_hook, HookContext, HookEvent, RecoveryCoordinator};
  use easy_kpf_core::types::{HookCommand, HookType, ReconnectPolicy};
  use std::time::Duration;

  fn policy() -> ReconnectPolicy {
    ReconnectPolicy {
      enabled: true,
      initial_delay_ms: 500,
      max_delay_ms: 5_000,
      stable_after_seconds: 30,
      max_attempts: 0,
    }
  }

  #[test]
  fn reconnect_attempts_back_off_and_cap() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = RecoveryCoordinator::default();
    let ticket = coordinator
      .begin("db gp")?
      .ok_or_else(|| std::io::Error::other("first recovery did not get a ticket"))?;

    assert_eq!(
      coordinator
        .next_attempt(&ticket, &policy())?
        .map(|value| value.delay),
      Some(Duration::from_millis(500))
    );
    assert_eq!(
      coordinator
        .next_attempt(&ticket, &policy())?
        .map(|value| value.delay),
      Some(Duration::from_millis(1_000))
    );
    for _ in 0..5 {
      let _ = coordinator.next_attempt(&ticket, &policy())?;
    }
    assert_eq!(
      coordinator
        .next_attempt(&ticket, &policy())?
        .map(|value| value.delay),
      Some(Duration::from_millis(5_000))
    );
    Ok(())
  }

  #[test]
  fn only_one_recovery_runs_per_service() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = RecoveryCoordinator::default();
    let ticket = coordinator
      .begin("db gp")?
      .ok_or_else(|| std::io::Error::other("first recovery did not get a ticket"))?;

    assert!(coordinator.begin("db gp")?.is_none());
    coordinator.finish(&ticket)?;
    assert!(coordinator.begin("db gp")?.is_some());
    Ok(())
  }

  #[test]
  fn hook_cooldown_deduplicates_shared_recovery_action() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = RecoveryCoordinator::default();
    let hook = HookCommand {
      kind: HookType::Command,
      command: "launchctl".to_string(),
      args: vec!["kickstart".to_string()],
      ssh_host: None,
      timeout_seconds: 30,
      cooldown_seconds: 30,
    };

    assert!(coordinator.reserve_hook(HookEvent::BeforeReconnect, &hook)?);
    assert!(!coordinator.reserve_hook(HookEvent::BeforeReconnect, &hook)?);
    Ok(())
  }

  #[tokio::test]
  async fn hook_receives_recovery_environment() {
    let hook = HookCommand {
      kind: HookType::Command,
      command: "/bin/sh".to_string(),
      args: vec![
        "-c".to_string(),
        concat!(
          "test \"$EASY_KPF_EVENT\" = before_reconnect",
          " && test \"$EASY_KPF_SERVICE\" = \"db gp\"",
          " && test \"$EASY_KPF_ERROR\" = timeout",
          " && test \"$EASY_KPF_ATTEMPT\" = 2"
        )
        .to_string(),
      ],
      ssh_host: None,
      timeout_seconds: 5,
      cooldown_seconds: 0,
    };

    assert!(
      run_hook(
        HookEvent::BeforeReconnect,
        &hook,
        &HookContext {
          service_name: "db gp",
          error: "timeout",
          attempt: 2,
        },
      )
      .await
    );
  }

  #[test]
  fn ssh_hook_targets_its_configured_host_and_remote_job() {
    let hook = HookCommand {
      kind: HookType::Ssh,
      command: "sudo systemctl restart demand-map-worker".to_string(),
      args: Vec::new(),
      ssh_host: Some("k3s-host".to_string()),
      timeout_seconds: 30,
      cooldown_seconds: 30,
    };

    assert_eq!(
      hook_invocation(&hook),
      Some((
        "ssh".to_string(),
        vec![
          "-o".to_string(),
          "BatchMode=yes".to_string(),
          "-o".to_string(),
          "ConnectTimeout=10".to_string(),
          "k3s-host".to_string(),
          "--".to_string(),
          "sudo systemctl restart demand-map-worker".to_string(),
        ],
      ))
    );
  }
}
