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
  #[command(about = "Restart every service in the last-active set")]
  ReconnectAll,
  #[command(about = "Control one port forward")]
  Pf {
    name: String,
    #[command(subcommand)]
    command: PortForwardCommand,
  },
  #[command(about = "List all configured port forwards and their state")]
  List,
  #[command(about = "Show status of all port forwards")]
  Status,
  #[command(about = "Bring the EasyKpf window to focus")]
  Show,
  #[command(about = "Print the ekpfctl version")]
  Version,
  #[command(about = "Print shell completion script to stdout")]
  Completions {
    #[arg(value_enum)]
    shell: Shell,
  },
}

#[derive(Subcommand)]
enum PortForwardCommand {
  #[command(about = "Start the port forward")]
  Start,
  #[command(about = "Stop the port forward")]
  Stop,
  #[command(about = "Reconnect the port forward")]
  Reconnect,
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  let request = match cli.command {
    Command::ReconnectAll => Request::ReconnectAll,
    Command::Pf { name, command } => match command {
      PortForwardCommand::Start => Request::Start { name },
      PortForwardCommand::Stop => Request::Stop { name },
      PortForwardCommand::Reconnect => Request::Reconnect { name },
    },
    Command::List => Request::List,
    Command::Status => Request::Status,
    Command::Show => Request::Show,
    Command::Version => {
      println!("{}", env!("CARGO_PKG_VERSION"));
      return;
    }
    Command::Completions { shell } => {
      let mut cmd = Cli::command();
      let bin_name = cmd.get_name().to_string();
      generate(shell, &mut cmd, bin_name, &mut io::stdout());
      return;
    }
  };

  client::send(request).await;
}

#[cfg(test)]
mod tests {
  use super::{Cli, Command, PortForwardCommand};
  use clap::Parser;

  #[test]
  fn parses_single_port_forward_reconnect_command() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["ekpfctl", "pf", "db gp", "reconnect"])?;

    assert!(matches!(
      cli.command,
      Command::Pf {
        name,
        command: PortForwardCommand::Reconnect
      } if name == "db gp"
    ));
    Ok(())
  }

  #[test]
  fn parses_version_command() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["ekpfctl", "version"])?;

    assert!(matches!(cli.command, Command::Version));
    Ok(())
  }
}
