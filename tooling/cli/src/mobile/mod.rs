// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{app_paths::tauri_dir, config::Config as TauriConfig},
  interface::DevProcess,
};
use anyhow::{bail, Result};
#[cfg(target_os = "macos")]
use cargo_mobile::apple::config::{
  Metadata as AppleMetadata, Platform as ApplePlatform, Raw as RawAppleConfig,
};
use cargo_mobile::{
  android::config::{Metadata as AndroidMetadata, Raw as RawAndroidConfig},
  bossy,
  config::{app::Raw as RawAppConfig, metadata::Metadata, Config, Raw},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsString, fmt::Write, path::PathBuf, process::ExitStatus};

pub mod android;
mod init;
#[cfg(target_os = "macos")]
pub mod ios;

pub struct DevChild(Option<bossy::Handle>);

impl Drop for DevChild {
  fn drop(&mut self) {
    // consume the handle since we're not waiting on it
    // just to prevent a log error
    // note that this doesn't leak any memory
    self.0.take().unwrap().leak();
  }
}

impl DevProcess for DevChild {
  fn kill(&mut self) -> std::io::Result<()> {
    self
      .0
      .as_mut()
      .unwrap()
      .kill()
      .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to kill"))
  }

  fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
    self
      .0
      .as_mut()
      .unwrap()
      .try_wait()
      .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to wait"))
  }
}

#[derive(PartialEq, Eq)]
pub enum Target {
  Android,
  #[cfg(target_os = "macos")]
  Ios,
}

impl Target {
  fn ide_name(&self) -> &'static str {
    match self {
      Self::Android => "Android Studio",
      #[cfg(target_os = "macos")]
      Self::Ios => "Xcode",
    }
  }

  fn command_name(&self) -> &'static str {
    match self {
      Self::Android => "android",
      #[cfg(target_os = "macos")]
      Self::Ios => "ios",
    }
  }

  fn ide_build_script_name(&self) -> &'static str {
    match self {
      Self::Android => "android-studio-script",
      #[cfg(target_os = "macos")]
      Self::Ios => "xcode-script",
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CliOptions {
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub vars: HashMap<String, OsString>,
}

fn options_path(bundle_identifier: &str, target: Target) -> PathBuf {
  let out_dir = dirs_next::cache_dir()
    .or_else(dirs_next::home_dir)
    .unwrap_or_else(std::env::temp_dir);
  let out_dir = out_dir.join(".tauri").join(bundle_identifier);
  let _ = std::fs::create_dir_all(&out_dir);
  out_dir
    .join("cli-options")
    .with_extension(target.command_name())
}

fn env_vars() -> HashMap<String, OsString> {
  let mut vars = HashMap::new();
  for (k, v) in std::env::vars_os() {
    let k = k.to_string_lossy();
    if k.starts_with("TAURI") && k != "TAURI_PRIVATE_KEY" && k != "TAURI_KEY_PASSWORD" {
      vars.insert(k.into_owned(), v);
    }
  }
  vars
}

/// Writes CLI options to be used later on the Xcode and Android Studio build commands
pub fn write_options(
  mut options: CliOptions,
  bundle_identifier: &str,
  target: Target,
) -> crate::Result<()> {
  options.vars.extend(env_vars());
  std::fs::write(
    options_path(bundle_identifier, target),
    &serde_json::to_string(&options)?,
  )?;
  Ok(())
}

fn read_options(config: &TauriConfig, target: Target) -> crate::Result<CliOptions> {
  let data = std::fs::read_to_string(options_path(&config.tauri.bundle.identifier, target))?;
  let options = serde_json::from_str(&data)?;
  Ok(options)
}

fn get_config(config: &TauriConfig) -> (Config, Metadata) {
  let mut s = config.tauri.bundle.identifier.rsplit('.');
  let app_name = s.next().unwrap_or("app").to_string();
  let mut domain = String::new();
  for w in s {
    domain.push_str(w);
    domain.push('.');
  }
  domain.pop();

  #[cfg(target_os = "macos")]
  let ios_options = read_options(config, Target::Ios).unwrap_or_default();
  let android_options = read_options(config, Target::Android).unwrap_or_default();

  let raw = Raw {
    app: RawAppConfig {
      name: app_name,
      stylized_name: config.package.product_name.clone(),
      domain,
      asset_dir: None,
      template_pack: None,
    },
    #[cfg(target_os = "macos")]
    apple: Some(RawAppleConfig {
      development_team: std::env::var("TAURI_APPLE_DEVELOPMENT_TEAM")
        .ok()
        .or_else(|| config.tauri.ios.development_team.clone())
        .expect("you must set `tauri > iOS > developmentTeam` config value or the `TAURI_APPLE_DEVELOPMENT_TEAM` environment variable"),
      project_dir: None,
      ios_no_default_features: None,
      ios_features: ios_options.features.clone(),
      macos_no_default_features: None,
      macos_features: None,
      bundle_version: config.package.version.clone(),
      bundle_version_short: config.package.version.clone(),
      ios_version: None,
      macos_version: None,
      use_legacy_build_system: None,
      plist_pairs: None,
      enable_bitcode: None,
    }),
    android: Some(RawAndroidConfig {
      min_sdk_version: None,
      vulkan_validation: None,
      project_dir: None,
      no_default_features: None,
      features: android_options.features.clone(),
    }),
  };
  let config = Config::from_raw(tauri_dir(), raw).unwrap();

  let metadata = Metadata {
    #[cfg(target_os = "macos")]
    apple: AppleMetadata {
      supported: true,
      ios: ApplePlatform {
        no_default_features: false,
        cargo_args: Some(ios_options.args),
        features: ios_options.features,
        libraries: None,
        frameworks: None,
        valid_archs: None,
        vendor_frameworks: None,
        vendor_sdks: None,
        asset_catalogs: None,
        pods: None,
        pod_options: None,
        additional_targets: None,
        pre_build_scripts: None,
        post_compile_scripts: None,
        post_build_scripts: None,
        command_line_arguments: None,
      },
      macos: Default::default(),
    },
    android: AndroidMetadata {
      supported: true,
      no_default_features: false,
      cargo_args: Some(android_options.args),
      features: android_options.features,
      app_sources: None,
      app_plugins: None,
      project_dependencies: None,
      app_dependencies: None,
      app_dependencies_platform: None,
      asset_packs: None,
      app_activity_name: None,
      app_permissions: None,
      app_theme_parent: None,
    },
  };

  (config, metadata)
}

fn ensure_init(project_dir: PathBuf, target: Target) -> Result<()> {
  if !project_dir.exists() {
    bail!(
      "{} project directory {} doesn't exist. Please run `tauri {} init` and try again.",
      target.ide_name(),
      project_dir.display(),
      target.command_name(),
    )
  } else {
    Ok(())
  }
}

fn log_finished(outputs: Vec<PathBuf>, kind: &str) {
  if !outputs.is_empty() {
    let mut printable_paths = String::new();
    for path in &outputs {
      writeln!(printable_paths, "        {}", path.display()).unwrap();
    }

    log::info!(action = "Finished"; "{} {}{} at:\n{}", outputs.len(), kind, if outputs.len() == 1 { "" } else { "s" }, printable_paths);
  }
}
