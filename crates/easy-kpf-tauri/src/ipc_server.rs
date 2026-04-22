use easy_kpf_core::ipc::protocol::{Request, Response, ResponseData, ServiceStatus};
use easy_kpf_core::ipc::socket_path::default_socket_path;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::services::{KubectlService, PortForwardService};

pub async fn spawn(app_handle: tauri::AppHandle) {
  let socket_path = default_socket_path();

  // Remove stale socket: try connecting; if refused/absent, unlink and rebind
  if socket_path.exists() {
    match tokio::time::timeout(
      std::time::Duration::from_millis(200),
      UnixStream::connect(&socket_path),
    )
    .await
    {
      Ok(Ok(_)) => {
        log::warn!("Another EasyKpf instance is already listening; skipping IPC server");
        return;
      }
      _ => {
        let _ = std::fs::remove_file(&socket_path);
      }
    }
  }

  let listener = match UnixListener::bind(&socket_path) {
    Ok(l) => l,
    Err(e) => {
      log::error!("Failed to bind IPC socket at {:?}: {}", socket_path, e);
      return;
    }
  };

  log::info!("IPC server listening at {:?}", socket_path);

  loop {
    match listener.accept().await {
      Ok((stream, _)) => {
        let handle = app_handle.clone();
        tokio::spawn(async move {
          if let Err(e) = handle_connection(stream, handle).await {
            log::warn!("IPC connection error: {}", e);
          }
        });
      }
      Err(e) => {
        log::error!("IPC accept error: {}", e);
        break;
      }
    }
  }
}

async fn handle_connection(
  stream: UnixStream,
  app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let (read_half, mut write_half) = stream.into_split();
  let mut reader = BufReader::new(read_half);
  let mut line = String::new();

  reader.read_line(&mut line).await?;
  let line = line.trim();
  if line.is_empty() {
    return Ok(());
  }

  let request: Request = serde_json::from_str(line)?;
  let response = dispatch(request, &app_handle).await;

  let mut json = serde_json::to_string(&response)?;
  json.push('\n');
  write_half.write_all(json.as_bytes()).await?;

  Ok(())
}

async fn dispatch(request: Request, app_handle: &tauri::AppHandle) -> Response {
  let pf = app_handle.state::<PortForwardService>();
  let kc = app_handle.state::<KubectlService>();

  match request {
    Request::List | Request::Status => {
      let configs = match pf.get_configs() {
        Ok(c) => c,
        Err(e) => return Response::Err { message: e.to_string() },
      };
      let running = match pf.get_running_services() {
        Ok(r) => r,
        Err(e) => return Response::Err { message: e.to_string() },
      };
      let statuses = configs
        .into_iter()
        .map(|c| {
          let running = running.contains(&c.name);
          ServiceStatus { name: c.name, running }
        })
        .collect();
      Response::Ok { data: ResponseData::Services(statuses) }
    }

    Request::Start { name } => {
      match pf.start_port_forward_by_key(kc.inner(), &name).await {
        Ok(msg) => Response::Ok { data: ResponseData::Text(msg) },
        Err(e) => Response::Err { message: e.to_string() },
      }
    }

    Request::Stop { name } => match pf.stop_port_forward(&name) {
      Ok(msg) => Response::Ok { data: ResponseData::Text(msg) },
      Err(e) => Response::Err { message: e.to_string() },
    },

    Request::ReconnectAll => {
      match crate::reconnect::reconnect_all(pf.inner(), kc.inner()).await {
        Ok(names) => Response::Ok { data: ResponseData::Reconnected(names) },
        Err(e) => Response::Err { message: e.to_string() },
      }
    }

    Request::Show => {
      crate::window::activate_and_show_window(app_handle);
      Response::Ok { data: ResponseData::Empty }
    }
  }
}
