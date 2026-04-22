use easy_kpf_core::ipc::protocol::{Request, Response, ResponseData};
use easy_kpf_core::ipc::socket_path::default_socket_path;
use std::process;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

pub async fn send(request: Request) {
  let socket_path = default_socket_path();

  let stream = match timeout(Duration::from_secs(2), UnixStream::connect(&socket_path)).await {
    Ok(Ok(s)) => s,
    Ok(Err(e)) => {
      eprintln!(
        "error: EasyKpf desktop app is not running (socket unavailable at {:?}: {})",
        socket_path, e
      );
      process::exit(2);
    }
    Err(_) => {
      eprintln!(
        "error: EasyKpf desktop app is not running (connection timed out at {:?})",
        socket_path
      );
      process::exit(2);
    }
  };

  let (read_half, mut write_half) = stream.into_split();
  let mut reader = BufReader::new(read_half);

  let mut line = serde_json::to_string(&request).unwrap_or_default();
  line.push('\n');

  if let Err(e) = write_half.write_all(line.as_bytes()).await {
    eprintln!("error: failed to send command: {}", e);
    process::exit(3);
  }

  let mut response_line = String::new();
  if let Err(e) = reader.read_line(&mut response_line).await {
    eprintln!("error: failed to read response: {}", e);
    process::exit(3);
  }

  let response: Response = match serde_json::from_str(response_line.trim()) {
    Ok(r) => r,
    Err(e) => {
      eprintln!("error: invalid response from server: {}", e);
      process::exit(3);
    }
  };

  match response {
    Response::Ok { data } => {
      print_data(data);
    }
    Response::Err { message } => {
      eprintln!("error: {}", message);
      process::exit(1);
    }
  }
}

fn print_data(data: ResponseData) {
  match data {
    ResponseData::Services(services) => {
      for s in services {
        let state = if s.running { "running" } else { "stopped" };
        println!("{:30} {}", s.name, state);
      }
    }
    ResponseData::Reconnected(names) => {
      if names.is_empty() {
        println!("nothing to reconnect");
      } else {
        for name in names {
          println!("reconnected: {}", name);
        }
      }
    }
    ResponseData::Text(msg) => println!("{}", msg),
    ResponseData::Empty => {}
  }
}
