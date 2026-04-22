mod client;

use clap::{Parser, Subcommand};
use easy_kpf_core::ipc::protocol::Request;

#[derive(Parser)]
#[command(name = "kpfctl", about = "Control the EasyKpf desktop app")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  #[command(about = "Start all stopped port forwards")]
  ReconnectAll,
  #[command(about = "Start a port forward by config name")]
  Start {
    name: String,
  },
  #[command(about = "Stop a port forward by config name")]
  Stop {
    name: String,
  },
  #[command(about = "List all configured port forwards and their state")]
  List,
  #[command(about = "Show status of all port forwards")]
  Status,
  #[command(about = "Bring the EasyKpf window to focus")]
  Show,
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  let request = match cli.command {
    Command::ReconnectAll => Request::ReconnectAll,
    Command::Start { name } => Request::Start { name },
    Command::Stop { name } => Request::Stop { name },
    Command::List => Request::List,
    Command::Status => Request::Status,
    Command::Show => Request::Show,
  };

  client::send(request).await;
}
