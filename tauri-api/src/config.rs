use serde::Deserialize;

use std::collections::HashMap;
use std::{fs, path};

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "window", rename_all = "camelCase")]
pub struct WindowConfig {
  #[serde(default = "default_width")]
  pub width: i32,
  #[serde(default = "default_height")]
  pub height: i32,
  #[serde(default = "default_resizable")]
  pub resizable: bool,
  #[serde(default = "default_title")]
  pub title: String,
  #[serde(default)]
  pub fullscreen: bool,
}

fn default_width() -> i32 {
  800
}

fn default_height() -> i32 {
  600
}

fn default_resizable() -> bool {
  true
}

fn default_title() -> String {
  "Tauri App".to_string()
}

fn default_window() -> WindowConfig {
  WindowConfig {
    width: default_width(),
    height: default_height(),
    resizable: default_resizable(),
    title: default_title(),
    fullscreen: false,
  }
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "embeddedServer", rename_all = "camelCase")]
pub struct EmbeddedServerConfig {
  #[serde(default = "default_host")]
  pub host: String,
  #[serde(default = "default_port")]
  pub port: String,
}

fn default_host() -> String {
  "http://127.0.0.1".to_string()
}

fn default_port() -> String {
  "random".to_string()
}

fn default_embedded_server() -> EmbeddedServerConfig {
  EmbeddedServerConfig {
    host: default_host(),
    port: default_port(),
  }
}

#[derive(PartialEq, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CliArg {
  pub short: Option<char>,
  pub name: String,
  pub description: Option<String>,
  pub long_description: Option<String>,
  pub takes_value: Option<bool>,
  pub multiple: Option<bool>,
  pub multiple_occurrences: Option<bool>,
  pub number_of_values: Option<u64>,
  pub possible_values: Option<Vec<String>>,
  pub min_values: Option<u64>,
  pub max_values: Option<u64>,
  pub required: Option<bool>,
  pub required_unless: Option<String>,
  pub required_unless_all: Option<Vec<String>>,
  pub required_unless_one: Option<Vec<String>>,
  pub conflicts_with: Option<String>,
  pub conflicts_with_all: Option<Vec<String>>,
  pub requires: Option<String>,
  pub requires_all: Option<Vec<String>>,
  pub requires_if: Option<Vec<String>>,
  pub required_if: Option<Vec<String>>,
  pub require_equals: Option<bool>,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CliSubcommand {
  description: Option<String>,
  long_description: Option<String>,
  before_help: Option<String>,
  after_help: Option<String>,
  args: Option<Vec<CliArg>>,
  subcommands: Option<HashMap<String, CliSubcommand>>,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "cli", rename_all = "camelCase")]
pub struct CliConfig {
  description: Option<String>,
  long_description: Option<String>,
  before_help: Option<String>,
  after_help: Option<String>,
  args: Option<Vec<CliArg>>,
  subcommands: Option<HashMap<String, CliSubcommand>>,
}

pub trait Cli {
  fn args(&self) -> Option<&Vec<CliArg>>;
  fn subcommands(&self) -> Option<&HashMap<String, CliSubcommand>>;
  fn description(&self) -> Option<&String>;
  fn long_description(&self) -> Option<&String>;
  fn before_help(&self) -> Option<&String>;
  fn after_help(&self) -> Option<&String>;
}

macro_rules! impl_cli {
  ( $($field_name:ident),+ $(,)?) => {
    $(
      impl Cli for $field_name {

        fn args(&self) -> Option<&Vec<CliArg>> {
          self.args.as_ref()
        }

        fn subcommands(&self) -> Option<&HashMap<String, CliSubcommand>> {
          self.subcommands.as_ref()
        }

        fn description(&self) -> Option<&String> {
          self.description.as_ref()
        }

        fn long_description(&self) -> Option<&String> {
          self.description.as_ref()
        }

        fn before_help(&self) -> Option<&String> {
          self.before_help.as_ref()
        }

        fn after_help(&self) -> Option<&String> {
          self.after_help.as_ref()
        }
      }
    )+
  }
}

impl_cli!(CliSubcommand, CliConfig);

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  #[serde(default = "default_window")]
  pub window: WindowConfig,
  #[serde(default = "default_embedded_server")]
  pub embedded_server: EmbeddedServerConfig,
  #[serde(default)]
  pub cli: Option<CliConfig>,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
}

fn default_dev_path() -> String {
  "".to_string()
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default = "default_tauri")]
  pub tauri: TauriConfig,
  #[serde(default = "default_build")]
  pub build: BuildConfig,
}

fn default_tauri() -> TauriConfig {
  TauriConfig {
    window: default_window(),
    embedded_server: default_embedded_server(),
    cli: None,
  }
}

fn default_build() -> BuildConfig {
  BuildConfig {
    dev_path: default_dev_path(),
  }
}

pub fn get() -> crate::Result<Config> {
  match option_env!("TAURI_CONFIG") {
    Some(config) => Ok(serde_json::from_str(config).expect("failed to parse TAURI_CONFIG env")),
    None => {
      let env_var = envmnt::get_or("TAURI_DIR", "../dist");
      let path = path::Path::new(&env_var);
      let contents = fs::read_to_string(path.join("tauri.conf.json"))?;

      Ok(serde_json::from_str(&contents).expect("failed to read tauri.conf.json"))
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  // generate a test_config based on the test fixture
  fn create_test_config() -> Config {
    let mut subcommands = std::collections::HashMap::new();
    subcommands.insert(
      "update".to_string(),
      CliSubcommand {
        description: Some("Updates the app".to_string()),
        long_description: None,
        before_help: None,
        after_help: None,
        args: Some(vec![CliArg {
          short: Some('b'),
          name: "background".to_string(),
          description: Some("Update in background".to_string()),
          ..Default::default()
        }]),
        subcommands: None,
      },
    );
    Config {
      tauri: TauriConfig {
        window: WindowConfig {
          width: 800,
          height: 600,
          resizable: true,
          title: String::from("Tauri API Validation"),
          fullscreen: false,
        },
        embedded_server: EmbeddedServerConfig {
          host: String::from("http://127.0.0.1"),
          port: String::from("random"),
        },
        cli: Some(CliConfig {
          description: Some("Tauri communication example".to_string()),
          long_description: None,
          before_help: None,
          after_help: None,
          args: Some(vec![
            CliArg {
              short: Some('c'),
              name: "config".to_string(),
              takes_value: Some(true),
              description: Some("Config path".to_string()),
              ..Default::default()
            },
            CliArg {
              short: Some('t'),
              name: "theme".to_string(),
              takes_value: Some(true),
              description: Some("App theme".to_string()),
              possible_values: Some(vec![
                "light".to_string(),
                "dark".to_string(),
                "system".to_string(),
              ]),
              ..Default::default()
            },
            CliArg {
              short: Some('v'),
              name: "verbose".to_string(),
              multiple_occurrences: Some(true),
              description: Some("Verbosity level".to_string()),
              ..Default::default()
            },
          ]),
          subcommands: Some(subcommands),
        }),
      },
      build: BuildConfig {
        dev_path: String::from("../dist"),
      },
    }
  }

  #[test]
  // test the get function.  Will only resolve to true if the TAURI_CONFIG variable is set properly to the fixture.
  fn test_get() {
    // get test_config
    let test_config = create_test_config();

    // call get();
    let config = get();

    // check to see if there is an OK or Err, on Err fail test.
    match config {
      // On Ok, check that the config is the same as the test config.
      Ok(c) => {
        println!("{:?}", c);
        assert_eq!(c, test_config)
      }
      Err(_) => assert!(false),
    }
  }

  #[test]
  // test all of the default functions
  fn test_defaults() {
    // get default tauri config
    let t_config = default_tauri();
    // get default build config
    let b_config = default_build();
    // get default dev path
    let d_path = default_dev_path();
    // get default embedded server
    let de_server = default_embedded_server();
    // get default window
    let d_window = default_window();
    // get default title
    let d_title = default_title();

    // create a tauri config.
    let tauri = TauriConfig {
      window: WindowConfig {
        width: 800,
        height: 600,
        resizable: true,
        title: String::from("Tauri App"),
        fullscreen: false,
      },
      embedded_server: EmbeddedServerConfig {
        host: String::from("http://127.0.0.1"),
        port: String::from("random"),
      },
      cli: None,
    };

    // create a build config
    let build = BuildConfig {
      dev_path: String::from(""),
    };

    // test the configs
    assert_eq!(t_config, tauri);
    assert_eq!(b_config, build);
    assert_eq!(de_server, tauri.embedded_server);
    assert_eq!(d_path, String::from(""));
    assert_eq!(d_title, tauri.window.title);
    assert_eq!(d_window, tauri.window);
  }
}
