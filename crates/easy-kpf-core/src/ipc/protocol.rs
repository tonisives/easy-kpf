use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
  ReconnectAll,
  Start { name: String },
  Stop { name: String },
  List,
  Status,
  Show,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Response {
  Ok { data: ResponseData },
  Err { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseData {
  Services(Vec<ServiceStatus>),
  Reconnected(Vec<String>),
  Text(String),
  Empty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
  pub name: String,
  pub running: bool,
}
