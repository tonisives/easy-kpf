#[derive(Debug)]
pub enum AutocompleteResult {
  Contexts(Vec<String>),
  Namespaces(Vec<String>),
  Services(Vec<String>),
  Ports(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct AutocompleteState {
  pub contexts: Vec<String>,
  pub namespaces: Vec<String>,
  pub services: Vec<String>,
  pub ports: Vec<String>,
  pub types: Vec<String>,
  pub selected_index: usize,
  pub loading: bool,
  pub focused: bool, // Whether suggestions panel is focused
  pub typing: bool,  // Whether in typing mode (i to enter, Esc to exit)
}

impl Default for AutocompleteState {
  fn default() -> Self {
    Self {
      contexts: vec![],
      namespaces: vec![],
      services: vec![],
      ports: vec![],
      types: vec!["kubectl".to_string(), "ssh".to_string()],
      selected_index: 0,
      loading: false,
      focused: false,
      typing: false,
    }
  }
}
