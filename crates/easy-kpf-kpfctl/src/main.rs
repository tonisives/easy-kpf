mod client;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use easy_kpf_core::ipc::protocol::Request;
use std::io;

#[derive(Parser)]
#[command(name = "ekpfctl", about = "Control the EasyKpf desktop app")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  #[command(about = "Start services from the last-active set that aren't running")]
  ReconnectAll,
  #[command(about = "Start a port forward by config name")]
  Start { name: String },
  #[command(about = "Stop a port forward by config name")]
  Stop { name: String },
  #[command(about = "List all configured port forwards and their state")]
  List,
  #[command(about = "Show status of all port forwards")]
  Status,
  #[command(about = "Bring the EasyKpf window to focus")]
  Show,
  #[command(about = "Print shell completion script to stdout")]
  Completions {
    #[arg(value_enum)]
    shell: Shell,
  },
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
    Command::Completions { shell } => {
      let mut cmd = Cli::command();
      let bin_name = cmd.get_name().to_string();
      generate(shell, &mut cmd, bin_name, &mut io::stdout());
      return;
    }
  };

  client::send(request).await;
}
