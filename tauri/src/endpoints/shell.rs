use crate::app::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  Execute { command: String, args: Vec<String> },
  /// The open URL in browser API
  Open { uri: String },
}

impl Cmd {
  pub async fn run(self) -> crate::Result<InvokeResponse> {
    match self {
      Self::Execute {
        command: _,
        args: _,
      } => {
        #[cfg(shell_execute)]
        {
          //TODO
          Ok(().into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::Open { uri } => {
        #[cfg(shell_open)]
        {
          open_browser(uri);
          Ok(().into())
        }
        #[cfg(not(shell_open))]
        Err(crate::Error::ApiNotAllowlisted("shell > open".to_string()))
      }
    }
  }
}

#[cfg(shell_open)]
pub fn open_browser(uri: String) {
  #[cfg(test)]
  assert!(uri.contains("http://"));

  #[cfg(not(test))]
  webbrowser::open(&uri).expect("Failed to open webbrowser with uri");
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;
  // Test the open func to see if proper uris can be opened by the browser.
  proptest! {
    #[cfg(shell_open)]
    #[test]
    fn check_open(uri in r"(http://)([\\w\\d\\.]+([\\w]{2,6})?)") {
      super::open_browser(uri);
    }
  }
}
