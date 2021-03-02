use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};
pub use tauri_api::config::Config;
use thiserror::Error;

pub mod context;
pub mod embedded_assets;

/// Represents all the errors that can happen while reading the config.
#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("unable to access current working directory: {0}")]
  CurrentDir(std::io::Error),

  // this error should be "impossible" because we use std::env::current_dir() - cover it anyways
  #[error("config file has no parent, this shouldn't be possible. file an issue on https://github.com/tauri-apps/tauri - target {0}")]
  Parent(PathBuf),

  #[error("unable to parse inline TAURI_CONFIG env var: {0}")]
  FormatInline(serde_json::Error),

  #[error("unable to parse config file at {path} because {error}")]
  Format {
    path: PathBuf,
    error: serde_json::Error,
  },

  #[error("unable to read config file at {path} because {error}")]
  Io {
    path: PathBuf,
    error: std::io::Error,
  },
}

/// Get the [`Config`] from the `TAURI_CONFIG` environmental variable, or read from the passed path.
///
/// The passed path should be relative to the building crate, as it is appended onto the current
/// working directory.
pub fn get_config(path: &Path) -> Result<(Config, PathBuf), ConfigError> {
  let cwd = std::env::current_dir().map_err(ConfigError::CurrentDir)?;
  let path = cwd.join(path);

  // in the future we may want to find a way to not need the TAURI_CONFIG env var so that
  // it is impossible for the content of two separate configs to get mixed up. The chances are
  // already unlikely unless the developer goes out of their way to run the cli on a different
  // project than the target crate.
  let config = if let Ok(env) = std::env::var("TAURI_CONFIG") {
    serde_json::from_str(&env).map_err(ConfigError::FormatInline)?
  } else {
    File::open(&path)
      .map_err(|error| ConfigError::Io {
        path: path.to_owned(),
        error,
      })
      .map(BufReader::new)
      .and_then(|file| {
        serde_json::from_reader(file).map_err(|error| ConfigError::Format {
          path: path.to_owned(),
          error,
        })
      })?
  };

  // this should be impossible because of the use of `current_dir()` above, but handle it anyways
  let parent = path
    .parent()
    .map(ToOwned::to_owned)
    .ok_or_else(|| ConfigError::Parent(path.to_owned()))?;

  Ok((config, parent))
}
